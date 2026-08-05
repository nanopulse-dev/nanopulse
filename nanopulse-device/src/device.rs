use defmt::{debug, error, info};
use embassy_time::{Duration, Timer};
use heapless::Vec;
use rand_core::RngCore;

use nanopulse::frames::{
    Frame, FrameType, Payload, activation_request, key_exchange_request, state, telemetry,
};
use nanopulse::keys;

use crate::State;
use crate::context::{Counters, Flash, RootSecurity, Session, write_counters, write_root_security};
use crate::errors::Error;
use crate::radio::Radio;
use crate::region::Region;

pub struct DeviceConfig {
    pub data_rate: usize,
    pub vendor_id: [u8; 4],
    pub profile_id: [u8; 2],
    pub version_id: [u8; 2],
}

pub struct Device<
    R: Radio,
    RNG: RngCore,
    F: Flash,
    const UPLINK_CHANNELS: usize,
    const DOWNLINK_CHANNELS: usize,
    const DATA_RATES: usize,
> {
    state: State,
    radio: R,
    rng: RNG,
    flash: F,
    session: Session,
    pub root_security: RootSecurity,
    counter_ctx: Counters,
    region: &'static Region<UPLINK_CHANNELS, DOWNLINK_CHANNELS, DATA_RATES>,
    config: DeviceConfig,
    rx_buffer: Vec<u8, { nanopulse::BUFFER_SIZE }>,
    rx_len: u8,
    rx_ch: usize,
    rx_dr: usize,
}

impl<
    R: Radio,
    RNG: RngCore,
    F: Flash,
    const UPLINK_CHANNELS: usize,
    const DOWNLINK_CHANNELS: usize,
    const DATA_RATES: usize,
> Device<R, RNG, F, UPLINK_CHANNELS, DOWNLINK_CHANNELS, DATA_RATES>
{
    pub fn new(
        radio: R,
        rng: RNG,
        flash: F,
        root_security: RootSecurity,
        counter_ctx: Counters,
        region: &'static Region<UPLINK_CHANNELS, DOWNLINK_CHANNELS, DATA_RATES>,
        config: DeviceConfig,
    ) -> Self {
        Device {
            radio,
            rng,
            flash,
            root_security,
            counter_ctx,
            region,
            session: Session::default(),
            state: State::KeyExchange,
            config,
            rx_buffer: Vec::from_array([0u8; nanopulse::BUFFER_SIZE]),
            rx_len: 0,
            rx_ch: 0,
            rx_dr: 0,
        }
    }

    pub async fn next(&mut self) -> Result<Option<Frame>, Error> {
        while self.state == State::KeyExchange {
            info!("starting key-exchange");
            if let Err(e) = self.key_exchange().await {
                error!("key-exchange error: {}", e);
                Timer::after(Duration::from_secs(10)).await;
            }
        }
        info!("key-exchange OK");

        while self.state == State::Activate {
            info!("starting activation");
            if let Err(e) = self.activate().await {
                error!("activation error: {}", e);
                Timer::after(Duration::from_secs(10)).await;
            }
        }
        info!("activation OK");

        if self.rx_len != 0 {
            info!("processing Rx Buffer");
            self.state = State::Idle;
            return self.handle_rx_buffer();
        }

        Ok(None)
    }

    pub fn has_rx_buffer(&self) -> bool {
        self.rx_len != 0
    }

    fn get_random_tx_channel(&mut self) -> usize {
        if self.region.uplink_channels.len() == 1 {
            0
        } else {
            self.rng.next_u32() as usize % self.region.uplink_channels.len()
        }
    }

    pub async fn key_exchange(&mut self) -> Result<(), Error> {
        if self.root_security.has_server_public_key() {
            debug!("skipping key-exchange, device already has server public-key");
            self.state = State::Activate;
            return Ok(());
        }

        info!("Starting Key Exchange");

        let tx_ch = self.get_random_tx_channel();
        self.counter_ctx.last_key_exchange_request_nonce = self.rng.next_u32();

        let pl = Frame {
            payload: Payload::KeyExchangeRequest(key_exchange_request::KeyExchangeRequest {
                device_pub_key: self.root_security.public_key,
                nonce: self.counter_ctx.last_key_exchange_request_nonce,
            }),
            mic: None,
        };

        let pl = Vec::try_from(&pl).map_err(|_| Error::HeaplessVec)?;

        info!("sending Key Exchange Request");
        self.rx_dr = self.region.uplink_downlink_data_rate_mapping[self.config.data_rate];
        self.rx_ch = self.region.uplink_downlink_channel_mapping[tx_ch];

        self.rx_len = self
            .radio
            .send_sleep_receive(
                self.region.uplink_channels[tx_ch],
                self.region.downlink_channels[self.rx_ch],
                &self.region.data_rates[self.config.data_rate],
                &self.region.data_rates[self.rx_dr],
                &pl,
                &mut self.rx_buffer,
                Duration::from_secs(5),
            )
            .await?;
        if self.rx_len == 0 {
            return Err(Error::RadioError);
        }

        let _ = self.handle_rx_buffer()?;
        Ok(())
    }

    pub async fn activate(&mut self) -> Result<(), Error> {
        if self.session.is_activated() {
            debug!("skipping activation, device is already activated");
            self.state = State::Idle;
            return Ok(());
        }

        info!("starting Activation");
        let req_counter = self.counter_ctx.activation_request_counter;
        self.counter_ctx.activation_request_counter += 1;

        let tx_ch = self.get_random_tx_channel();
        let enc_key = keys::get_encryption_key(
            &self.root_security.root_key,
            FrameType::ActivationRequest,
            false,
        );
        let mic_key = keys::get_mic_key(
            &self.root_security.root_key,
            FrameType::ActivationRequest,
            tx_ch as u8,
            self.config.data_rate as u8,
        );

        debug!("root_key: {:02x}", self.root_security.root_key);
        debug!(
            "mic key: {:02x} (tx_ch: {}, tx_dr: {})",
            mic_key, tx_ch, self.config.data_rate
        );

        let mut pl = Frame {
            payload: Payload::ActivationRequest(activation_request::ActivationRequest {
                short_id: self.root_security.short_id,
                counter: req_counter,
                profile_info: activation_request::ProfileInfo::Plain(
                    activation_request::ProfileInfoPlain {
                        vendor_id: self.config.vendor_id,
                        profile_id: self.config.profile_id,
                        version_id: self.config.version_id,
                    },
                ),
            }),
            mic: None,
        };

        pl.encrypt(&enc_key).map_err(|e| {
            error!("encrypt error: {}", e);
            Error::CryptoError
        })?;
        pl.set_mic(&mic_key).map_err(|e| {
            error!("set mic error: {}", e);
            Error::CryptoError
        })?;

        let pl = Vec::try_from(&pl).map_err(|_| Error::HeaplessVec)?;

        info!("sending Activation Request");
        self.rx_dr = self.region.uplink_downlink_data_rate_mapping[self.config.data_rate];
        self.rx_ch = self.region.uplink_downlink_channel_mapping[tx_ch];

        self.rx_len = self
            .radio
            .send_sleep_receive(
                self.region.uplink_channels[tx_ch],
                self.region.downlink_channels[self.rx_ch],
                &self.region.data_rates[self.config.data_rate],
                &self.region.data_rates[self.rx_dr],
                &pl,
                &mut self.rx_buffer,
                Duration::from_secs(5),
            )
            .await?;
        if self.rx_len == 0 {
            return Err(Error::RadioError);
        }

        write_counters(&mut self.flash, &self.counter_ctx);
        let _ = self.handle_rx_buffer()?;

        Ok(())
    }

    pub async fn send_telemetry(&mut self, pl: &[u8]) -> Result<(), Error> {
        if !self.session.is_activated() {
            error!("device must be activated first");
            return Err(Error::NotActivated);
        }

        info!("starting telemetry");
        let telemetry_count = self.session.telemetry_up;
        self.session.telemetry_up += 1;

        let tx_ch = self.get_random_tx_channel();
        let enc_key =
            keys::get_encryption_key(&self.session.session_root_key, FrameType::Telemetry, false);
        let mic_key = keys::get_mic_key(
            &self.session.session_root_key,
            FrameType::Telemetry,
            tx_ch as u8,
            self.config.data_rate as u8,
        );

        debug!("session_root_key: {:02x}", self.session.session_root_key);
        debug!("enc_key: {:02x}", enc_key);
        debug!("mic_key: {:02x}", mic_key);

        let mut pl = Frame {
            payload: Payload::Telemetry(telemetry::Telemetry {
                short_id: self.root_security.short_id,
                counter: telemetry_count,
                payload: Vec::try_from(pl).map_err(|_| Error::HeaplessVec)?,
            }),
            mic: None,
        };
        pl.encrypt(&enc_key).map_err(|e| {
            error!("encrypt error: {}", e);
            Error::CryptoError
        })?;
        pl.set_mic(&mic_key).map_err(|e| {
            error!("set mic error: {}", e);
            Error::CryptoError
        })?;

        let pl = Vec::try_from(&pl).map_err(|_| Error::HeaplessVec)?;
        self.rx_dr = self.region.uplink_downlink_data_rate_mapping[self.config.data_rate];
        self.rx_ch = self.region.uplink_downlink_channel_mapping[tx_ch];
        info!("sending telemetry, {:02x}", pl.as_slice());

        self.rx_len = self
            .radio
            .send_sleep_receive(
                self.region.uplink_channels[tx_ch],
                self.region.downlink_channels[self.rx_ch],
                &self.region.data_rates[self.config.data_rate],
                &self.region.data_rates[self.rx_dr],
                &pl,
                &mut self.rx_buffer,
                Duration::from_secs(1),
            )
            .await?;

        Ok(())
    }

    pub async fn send_state(&mut self, pl: &[u8]) -> Result<(), Error> {
        if !self.session.is_activated() {
            error!("device must be activated first");
            return Err(Error::NotActivated);
        }

        info!("starting state");
        let state_counter = self.session.state_up;
        self.session.state_up += 1;

        let tx_ch = self.get_random_tx_channel();
        let enc_key =
            keys::get_encryption_key(&self.session.session_root_key, FrameType::State, false);
        let mic_key = keys::get_mic_key(
            &self.session.session_root_key,
            FrameType::State,
            tx_ch as u8,
            self.config.data_rate as u8,
        );

        debug!("session_root_key: {:02x}", self.session.session_root_key);
        debug!("enc_key: {:02x}", enc_key);
        debug!("mic_key: {:02x}", mic_key);

        let mut pl = Frame {
            payload: Payload::State(state::State {
                is_downlink: false,
                short_id: self.root_security.short_id,
                counter: state_counter,
                payload: Vec::try_from(pl).map_err(|_| Error::HeaplessVec)?,
            }),
            mic: None,
        };
        pl.encrypt(&enc_key).map_err(|e| {
            error!("encrypt error: {}", e);
            Error::CryptoError
        })?;
        pl.set_mic(&mic_key).map_err(|e| {
            error!("set mic error: {}", e);
            Error::CryptoError
        })?;
        let pl = Vec::try_from(&pl).map_err(|_| Error::HeaplessVec)?;

        self.rx_dr = self.region.uplink_downlink_data_rate_mapping[self.config.data_rate];
        self.rx_ch = self.region.uplink_downlink_channel_mapping[tx_ch];

        info!("sending state: {:02x}", pl.as_slice());
        self.rx_len = self
            .radio
            .send_sleep_receive(
                self.region.uplink_channels[tx_ch],
                self.region.downlink_channels[self.rx_ch],
                &self.region.data_rates[self.config.data_rate],
                &self.region.data_rates[self.rx_dr],
                &pl,
                &mut self.rx_buffer,
                Duration::from_secs(1),
            )
            .await?;

        Ok(())
    }

    fn handle_rx_buffer(&mut self) -> Result<Option<Frame>, Error> {
        let rx_len = self.rx_len as usize;
        self.rx_len = 0;

        info!("decoding rx buffer, len: {}", rx_len);
        debug!("rx data: {:02x}", self.rx_buffer[..rx_len]);

        let mut resp = Frame::try_from(&self.rx_buffer[..rx_len]).map_err(|e| {
            error!("decode frame error: {}", e);
            Error::FrameError
        })?;

        match &resp.payload {
            Payload::KeyExchangeResponse(pl) => {
                info!("received KeyExchangeResponse");

                if self.state != State::KeyExchange {
                    error!("invalid payload for current device state");
                    return Err(Error::FrameError);
                }

                let secret =
                    nanopulse::x25519_dalek::StaticSecret::from(self.root_security.secret_key);
                debug!("secret key: {:02x}", secret.to_bytes());

                let pub_key = nanopulse::x25519_dalek::PublicKey::from(pl.server_pub_key);
                debug!("server public key: {:02x}", pub_key.to_bytes());

                let shared_key = keys::get_shared_secret(&secret, &pub_key);
                debug!("shared key: {:02x}", shared_key);

                let root_key = keys::get_root_key(
                    &shared_key,
                    &self.root_security.pin,
                    self.counter_ctx.last_key_exchange_request_nonce,
                    pl.counter,
                );
                debug!(
                    "root key: {:02x} (pin: {:02x}, req nonce: {}, resp: nonce: {})",
                    root_key,
                    self.root_security.pin,
                    self.counter_ctx.last_key_exchange_request_nonce,
                    pl.counter
                );

                let mic_key = keys::get_mic_key(
                    &root_key,
                    FrameType::KeyExchangeResponse,
                    self.rx_ch as u8,
                    self.rx_dr as u8,
                );
                info!(
                    "mic key: {:02x} (tx_ch: {}, tx_dr: {})",
                    mic_key, self.rx_ch, self.rx_dr
                );

                let mic_ok = resp.validate_mic(&mic_key).map_err(|e| {
                    error!("validate_mic error: {}", e);
                    Error::FrameError
                })?;

                if !mic_ok {
                    error!("invalid mic");
                    return Err(Error::FrameError);
                }

                info!("mic ok");

                self.root_security.server_public_key = pub_key.to_bytes();
                self.root_security.root_key = root_key;
                write_root_security(&mut self.flash, &self.root_security);

                self.state = State::Activate;

                Ok(None)
            }
            Payload::ActivationResponse(pl) => {
                info!("received ActivationResponse");

                if self.state != State::Activate {
                    error!("invalid payload for current device state");
                    return Err(Error::FrameError);
                }

                let sess_root_key = keys::get_session_root_key(
                    &self.root_security.root_key,
                    self.counter_ctx.activation_request_counter - 1,
                    pl.counter,
                );
                let enc_key =
                    keys::get_encryption_key(&sess_root_key, FrameType::ActivationResponse, true);
                let mic_key = keys::get_mic_key(
                    &sess_root_key,
                    FrameType::ActivationResponse,
                    self.rx_ch as u8,
                    self.rx_dr as u8,
                );
                info!(
                    "session root key: {:02x} (req counter: {}, resp counter: {})",
                    sess_root_key,
                    self.counter_ctx.activation_request_counter - 1,
                    pl.counter
                );
                info!("encryption key: {:02x}", enc_key);
                info!(
                    "mic key: {:02x} (tx_ch: {}, tx_dr: {})",
                    mic_key, self.rx_ch, self.rx_dr
                );

                let mic_ok = resp.validate_mic(&mic_key).map_err(|e| {
                    error!("validate mic error: {}", e);
                    Error::FrameError
                })?;
                if !mic_ok {
                    error!("invalid mic");
                    return Err(Error::FrameError);
                }

                info!("mic ok");

                if pl.counter >= self.counter_ctx.activation_response_counter {
                    info!("syncing activation_response_counter");
                    self.counter_ctx.activation_response_counter = pl.counter + 1;
                } else {
                    error!("activation_response_counter has already been used");
                    return Err(Error::CounterError);
                }

                resp.decrypt(&enc_key).map_err(|e| {
                    error!("decrypt error: {}", e);
                    Error::CryptoError
                })?;

                self.session.session_root_key = sess_root_key;
                write_counters(&mut self.flash, &self.counter_ctx);

                self.state = State::Idle;

                Ok(None)
            }
            _ => {
                let enc_key = keys::get_encryption_key(
                    &self.session.session_root_key,
                    resp.frame_type(),
                    true,
                );
                let mic_key = keys::get_mic_key(
                    &self.session.session_root_key,
                    resp.frame_type(),
                    self.rx_ch as u8,
                    self.rx_dr as u8,
                );

                debug!("encryption key: {:02x}", enc_key);
                debug!(
                    "mic key: {:02x} (tx_ch: {}, tx_dr: {})",
                    mic_key, self.rx_ch, self.rx_dr,
                );

                let mic_ok = resp.validate_mic(&mic_key).map_err(|e| {
                    error!("validate mic error: {}", e);
                    Error::FrameError
                })?;

                if !mic_ok {
                    error!("invalid mic");
                    return Err(Error::FrameError);
                }
                info!("mic ok");

                let counter_ok = match &resp.payload {
                    Payload::State(pl) => {
                        if pl.counter >= self.session.state_down {
                            self.session.state_down = pl.counter + 1;
                            true
                        } else {
                            error!("invalid State counter");
                            false
                        }
                    }
                    _ => false,
                };
                if !counter_ok {
                    return Err(Error::CounterError);
                }

                resp.decrypt(&enc_key).map_err(|e| {
                    error!("decrypt error: {}", e);
                    Error::CryptoError
                })?;

                Ok(Some(resp))
            }
        }
    }
}
