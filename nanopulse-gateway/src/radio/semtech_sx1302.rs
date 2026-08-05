use std::collections::HashMap;
use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Result, anyhow};
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

use nanopulse_hal_sx1302::{
    bindings, board_setconf, get_eui, get_instcnt, receive, rxif_setconf, rxrf_setconf, send,
    start, stop, time_on_air, txgain_setconf,
};
use nanopulse_structs::common::CodingRate;
use nanopulse_structs::gateway::{
    ChannelConfiguration, DataRateConfiguration, FskDataRateConfiguration,
    LoraDataRateConfiguration, RxEvent, Timing, TxAckStatus, TxCommand,
};

use crate::config;
use crate::radio::{
    jitqueue::{self, TxPacket as TxPacketTrait},
    semtech_helpers,
};
use crate::reset::Reset;
use crate::traits::Radio;

static CONCENTRATOR_COUNT: LazyLock<Mutex<(u32, Duration)>> =
    LazyLock::new(|| Mutex::new((0, Duration::ZERO)));

static QUEUE: LazyLock<Mutex<jitqueue::Queue<TxPacket>>> =
    LazyLock::new(|| Mutex::new(jitqueue::Queue::new(128, jitqueue::Config::default())));

struct TxPacket {
    tx_id: u32,
    data_rate: u8,
    channel: u8,
    tx_packet: bindings::lgw_pkt_tx_s,
}

impl jitqueue::TxPacket for TxPacket {
    fn get_id(&self) -> u32 {
        self.tx_id
    }

    fn get_count(&self) -> Duration {
        get_count(self.tx_packet.count_us)
    }

    fn get_time_on_air(&self) -> Result<Duration> {
        Ok(time_on_air(&self.tx_packet))
    }

    fn get_tx_mode(&self) -> jitqueue::TxMode {
        match self.tx_packet.tx_mode as u32 {
            bindings::IMMEDIATE => jitqueue::TxMode::Immediate,
            bindings::TIMESTAMPED => jitqueue::TxMode::Timestamped,
            _ => jitqueue::TxMode::Timestamped,
        }
    }

    fn set_tx_mode(&mut self, tx_mode: jitqueue::TxMode) {
        match tx_mode {
            jitqueue::TxMode::Immediate => self.tx_packet.tx_mode = bindings::IMMEDIATE as u8,
            jitqueue::TxMode::Timestamped => self.tx_packet.tx_mode = bindings::TIMESTAMPED as u8,
        }
    }

    fn set_count(&mut self, count: Duration) {
        self.tx_packet.count_us = (count.as_micros() % (u32::MAX as u128 + 1)) as u32;
    }
}

struct RadioStruct {
    started: AtomicBool,
    reset: Reset,
    board_config: bindings::lgw_conf_board_s,
    tx_gain_config: [bindings::lgw_tx_gain_lut_s; 2],
    rx_rf_config: [bindings::lgw_conf_rxrf_s; 2],
    rx_if_config: [bindings::lgw_conf_rxif_s; 10],
    data_rates: HashMap<u8, DataRateConfiguration>,
    uplink_channels: HashMap<u8, ChannelConfiguration>,
    downlink_channels: HashMap<u8, ChannelConfiguration>,
    jit_join_handle: Option<JoinHandle<()>>,
    antenna_gain_dbi: i8,
}

pub fn new(
    config: &config::semtech_sx1302::Configuration,
    board_mapping: &str,
) -> Result<Box<dyn Radio>> {
    info!(board_mapping = %board_mapping, "Initializing SX1302/3 backend");
    let mapping_conf = config
        .mappings
        .get(board_mapping)
        .cloned()
        .ok_or_else(|| anyhow!("board_mapping '{}' does not exist", board_mapping))?;

    // only configure reset for SPI devices.
    let mut reset = Reset::new();
    if !mapping_conf.com_dev_spi.is_empty() && !mapping_conf.reset_chip.is_empty() {
        reset.add(
            &mapping_conf.reset_chip,
            mapping_conf.reset_pin,
            gpiocdev::line::Value::Active,
            Duration::from_millis(100),
        )?;
    }

    let board_config = bindings::lgw_conf_board_s {
        lorawan_public: true,
        clksrc: config.concentrator.clock_source,
        full_duplex: config.concentrator.full_duplex,
        com_type: if mapping_conf.com_dev_usb.is_empty() {
            bindings::com_type_e_LGW_COM_SPI
        } else {
            bindings::com_type_e_LGW_COM_USB
        },
        com_path: {
            let com_path = if mapping_conf.com_dev_usb.is_empty() {
                mapping_conf.com_dev_spi.clone()
            } else {
                mapping_conf.com_dev_usb.clone()
            };
            let com_path = CString::new(com_path).unwrap();
            let com_path = com_path.as_bytes_with_nul();
            if com_path.len() > 64 {
                return Err(anyhow!("com_dev_spi / _usb max length is 64"));
            }
            let mut com_path_chars = [0; 64];
            for (i, b) in com_path.iter().enumerate() {
                com_path_chars[i] = *b as c_char;
            }

            com_path_chars
        },
    };

    let tx_gain_config = {
        let mut tx_gain_conf = [bindings::lgw_tx_gain_lut_s::default(); 2];

        for (i, config) in config.radios.iter().enumerate() {
            if config.tx_gain_table.len() > 16 {
                return Err(anyhow!(
                    "max size for tx_gain_table is 16, found: {}",
                    config.tx_gain_table.len()
                ));
            }

            let mut lut = [bindings::lgw_tx_gain_s::default(); 16];
            for (i, v) in config.tx_gain_table.iter().enumerate() {
                lut[i] = bindings::lgw_tx_gain_s {
                    rf_power: v.rf_power,
                    dig_gain: v.dig_gain,
                    pa_gain: v.pa_gain,
                    dac_gain: v.dac_gain,
                    mix_gain: v.mix_gain,
                    offset_i: v.offset_i,
                    offset_q: v.offset_q,
                    pwr_idx: v.pwr_idx,
                };
            }

            tx_gain_conf[i] = bindings::lgw_tx_gain_lut_s {
                size: config.tx_gain_table.len() as u8,
                lut,
            };
        }

        tx_gain_conf
    };

    let rx_rf_config = {
        let mut rx_conf = [bindings::lgw_conf_rxrf_s::default(); 2];
        let clock_source = config.concentrator.clock_source;

        for (i, config) in config.radios.iter().enumerate() {
            // https://github.com/Lora-net/sx1302_hal/blob/master/util_chip_id/src/chip_id.c#L173
            rx_conf[i].enable = i == 0 || i == clock_source as usize;
            rx_conf[i].freq_hz = 868500000;
            rx_conf[i].tx_enable = config.tx_enabled;
            rx_conf[i].type_ = match config.radio_type.as_str() {
                "SX1250" => bindings::lgw_radio_type_t_LGW_RADIO_TYPE_SX1250,
                "SX1255" => bindings::lgw_radio_type_t_LGW_RADIO_TYPE_SX1255,
                "SX1257" => bindings::lgw_radio_type_t_LGW_RADIO_TYPE_SX1257,
                "SX1272" => bindings::lgw_radio_type_t_LGW_RADIO_TYPE_SX1272,
                "SX1276" => bindings::lgw_radio_type_t_LGW_RADIO_TYPE_SX1276,
                _ => return Err(anyhow!("unexpected radio_type: {}", config.radio_type)),
            };
            rx_conf[i].rssi_offset = config.rssi_offset;
            rx_conf[i].rssi_tcomp = bindings::lgw_rssi_tcomp_s {
                coeff_a: config.rssi_temp_compensation.coeff_a,
                coeff_b: config.rssi_temp_compensation.coeff_b,
                coeff_c: config.rssi_temp_compensation.coeff_c,
                coeff_d: config.rssi_temp_compensation.coeff_d,
                coeff_e: config.rssi_temp_compensation.coeff_e,
            };
            rx_conf[i].single_input_mode = config.single_input_mode;
        }

        rx_conf
    };

    Ok(Box::new(RadioStruct {
        started: AtomicBool::new(false),
        reset,
        board_config,
        rx_rf_config,
        tx_gain_config,
        rx_if_config: [bindings::lgw_conf_rxif_s::default(); 10],
        data_rates: Default::default(),
        uplink_channels: Default::default(),
        downlink_channels: Default::default(),
        antenna_gain_dbi: config.antenna_gain_dbi,
        jit_join_handle: None,
    }))
}

impl RadioStruct {
    fn get_data_rate(&self, pkt: &bindings::lgw_pkt_rx_s) -> Result<u8> {
        let data_rate = match pkt.modulation as u32 {
            bindings::MOD_LORA => DataRateConfiguration::Lora(LoraDataRateConfiguration {
                bandwidth: match pkt.bandwidth as u32 {
                    bindings::BW_500KHZ => 500000,
                    bindings::BW_250KHZ => 250000,
                    bindings::BW_125KHZ => 125000,
                    _ => return Err(anyhow!("unexpected bandwidth: {}", pkt.bandwidth)),
                },
                spreading_factor: match pkt.datarate {
                    bindings::DR_LORA_SF5 => 5,
                    bindings::DR_LORA_SF6 => 6,
                    bindings::DR_LORA_SF7 => 7,
                    bindings::DR_LORA_SF8 => 8,
                    bindings::DR_LORA_SF9 => 9,
                    bindings::DR_LORA_SF10 => 10,
                    bindings::DR_LORA_SF11 => 11,
                    bindings::DR_LORA_SF12 => 12,
                    _ => {
                        return Err(anyhow!("unexpected spreading-factor: {}", pkt.datarate));
                    }
                },
                coding_rate: match pkt.coderate as u32 {
                    bindings::CR_LORA_4_5 => CodingRate::Cr45,
                    bindings::CR_LORA_4_6 => CodingRate::Cr46,
                    bindings::CR_LORA_4_7 => CodingRate::Cr47,
                    bindings::CR_LORA_4_8 => CodingRate::Cr48,
                    _ => return Err(anyhow!("unexpected coding-rate: {}", pkt.coderate)),
                },
            }),
            bindings::MOD_FSK => DataRateConfiguration::Fsk(FskDataRateConfiguration {
                bitrate: pkt.datarate,
                frequency_deviation: 25_000,
            }),
            _ => return Err(anyhow!("unknown modulation: {}", pkt.modulation)),
        };

        self.data_rates
            .iter()
            .find(|(_, v)| (*v).eq(&data_rate))
            .map(|(k, _)| k)
            .cloned()
            .ok_or_else(|| {
                anyhow!(
                    "data-rate not found for packet with modulation: {:?}",
                    data_rate
                )
            })
    }

    fn get_uplink_channel_index(&self, freq: u32, dr: u8) -> Result<u8> {
        for (i, c) in &self.uplink_channels {
            if c.freq_hz == freq && c.data_rates.contains(&dr) {
                return Ok(*i);
            }
        }

        Err(anyhow!(
            "no uplink channel-index for frequency: {}, data-rate: {}",
            freq,
            dr
        ))
    }
}

impl Radio for RadioStruct {
    fn start(&mut self) -> Result<()> {
        if self.started.load(Ordering::Relaxed) {
            return Err(anyhow!("radio has already been started"));
        }

        self.reset.reset()?;

        debug!(conf = ?self.board_config, "board_setconf");
        board_setconf(&self.board_config)?;
        for (i, config) in self.tx_gain_config.iter().enumerate() {
            debug!(i = i, conf = ?config, "txgain_setconf");
            txgain_setconf(i as u8, config)?;
        }
        for (i, config) in self.rx_rf_config.iter().enumerate() {
            debug!(i = i, conf = ?config, "rxrf_setconf");
            rxrf_setconf(i as u8, config)?;
        }
        for (i, config) in self.rx_if_config.iter().enumerate() {
            debug!(i = i, conf = ?config, "rxif_setconf");
            rxif_setconf(i as u8, config)?;
        }

        info!("Starting radio");
        start()?;

        self.started.store(true, Ordering::Relaxed);

        self.jit_join_handle = Some(tokio::spawn({
            info!("Starting JIT queue loop");

            let antenna_gain_dbi = self.antenna_gain_dbi;
            async move {
                loop {
                    sleep(Duration::from_millis(10)).await;

                    let pkt = QUEUE.lock().unwrap().pop(get_count(get_instcnt().unwrap()));
                    if let Some(mut pkt) = pkt {
                        pkt.tx_packet.rf_power -= antenna_gain_dbi;

                        match send(&mut pkt.tx_packet) {
                            Ok(_) => {
                                info!(
                                    tx_id = pkt.get_id(),
                                    count = ?pkt.get_count(),
                                    dr = pkt.data_rate,
                                    channel = pkt.channel,
                                    "Scheduled packet for tx",
                                );
                            }
                            Err(e) => {
                                error!(error = %e, "Schedule packet for tx error");
                            }
                        }
                    }
                }
            }
        }));

        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        info!("Stopping radio");

        if let Some(handle) = &self.jit_join_handle {
            info!("Aborting JIT queue loop");
            handle.abort();
        }
        self.jit_join_handle = None;

        self.started.store(false, Ordering::Relaxed);
        stop()
    }

    fn get_id(&self) -> Result<[u8; 8]> {
        info!("Reading ID from radio");
        if !self.started.load(Ordering::Relaxed) {
            return Err(anyhow!("radio has not been started"));
        }

        get_eui()
    }

    fn configure_data_rates(
        &mut self,
        data_rates: &HashMap<u8, DataRateConfiguration>,
    ) -> Result<()> {
        info!("Reconfiguring data-rates");
        self.data_rates = data_rates.clone();
        Ok(())
    }

    fn configure_channels(
        &mut self,
        uplink_channels: &HashMap<u8, ChannelConfiguration>,
        downlink_channels: &HashMap<u8, ChannelConfiguration>,
    ) -> Result<()> {
        info!("Reconfiguring channels");
        self.uplink_channels = uplink_channels.clone();
        self.downlink_channels = downlink_channels.clone();

        // configure radios
        let radio_frequencies =
            semtech_helpers::get_radio_frequencies(&self.data_rates, &self.uplink_channels)?;
        for (i, freq) in radio_frequencies.iter().enumerate() {
            let freq = *freq;
            let enable = freq != 0;

            info!(
                radio = i,
                enable = enable,
                freq_hz = freq,
                "Configuring radio"
            );
            self.rx_rf_config[i].freq_hz = freq;
            self.rx_rf_config[i].enable = enable;
        }

        // reset channels
        self.rx_if_config = [bindings::lgw_conf_rxif_s::default(); 10];

        // get channels from config
        let multi_sf_channels =
            semtech_helpers::get_lora_multi_fs_channels(&self.data_rates, &self.uplink_channels)?;
        let lora_std_channel =
            semtech_helpers::get_lora_std_channel(&self.data_rates, &self.uplink_channels);
        let fsk_channel = semtech_helpers::get_fsk_channel(&self.data_rates, &self.uplink_channels);

        // multi SF
        for (i, c) in multi_sf_channels.iter().enumerate() {
            let radio_i =
                semtech_helpers::get_radio_for_channel(&radio_frequencies, c.freq_hz, c.bandwidth)?;
            let radio_freq_offset = c.freq_hz as i32 - radio_frequencies[radio_i] as i32;

            info!(
                channel = i,
                radio = radio_i,
                freq_hz = c.freq_hz,
                bw_hz = c.bandwidth,
                "Configuring multi-SF LoRa channel"
            );
            self.rx_if_config[i].enable = true;
            self.rx_if_config[i].rf_chain = radio_i as u8;
            self.rx_if_config[i].freq_hz = radio_freq_offset;
        }

        // LoRa Std
        if let Some(c) = lora_std_channel {
            let radio_i =
                semtech_helpers::get_radio_for_channel(&radio_frequencies, c.freq_hz, c.bandwidth)?;
            let radio_freq_offset = c.freq_hz as i32 - radio_frequencies[radio_i] as i32;

            info!(
                radio = radio_i,
                freq_hz = c.freq_hz,
                bw_hz = c.bandwidth,
                sf = c.spreading_factor,
                "Configuring LoRa Std channel"
            );

            self.rx_if_config[8].enable = true;
            self.rx_if_config[8].rf_chain = radio_i as u8;
            self.rx_if_config[8].freq_hz = radio_freq_offset;
            self.rx_if_config[8].bandwidth = match c.bandwidth {
                125000 => bindings::BW_125KHZ as u8,
                250000 => bindings::BW_250KHZ as u8,
                500000 => bindings::BW_500KHZ as u8,
                _ => return Err(anyhow!("invalid bandwidth: {}", c.bandwidth)),
            };
            self.rx_if_config[8].datarate = match c.spreading_factor {
                5 => bindings::DR_LORA_SF5,
                6 => bindings::DR_LORA_SF6,
                7 => bindings::DR_LORA_SF7,
                8 => bindings::DR_LORA_SF8,
                9 => bindings::DR_LORA_SF9,
                10 => bindings::DR_LORA_SF10,
                11 => bindings::DR_LORA_SF11,
                12 => bindings::DR_LORA_SF12,
                _ => return Err(anyhow!("invalid spreading-factor: {}", c.spreading_factor)),
            };
        }

        // FSK
        if let Some(c) = fsk_channel {
            let radio_i =
                semtech_helpers::get_radio_for_channel(&radio_frequencies, c.freq_hz, c.bandwidth)?;
            let radio_freq_offset = c.freq_hz as i32 - radio_frequencies[radio_i] as i32;

            // source:
            // https://github.com/Lora-net/packet_forwarder/blob/master/lora_pkt_fwd/src/lora_pkt_fwd.c#L635
            let bw = 2 * c.freq_deviation + c.bitrate;
            let bw = if bw == 0 {
                bindings::BW_UNDEFINED
            } else if bw <= 125000 {
                bindings::BW_125KHZ
            } else if bw <= 250000 {
                bindings::BW_250KHZ
            } else if bw <= 500000 {
                bindings::BW_500KHZ
            } else {
                bindings::BW_UNDEFINED
            };

            info!(
                radio = radio_i,
                freq_hz = c.freq_hz,
                freq_dev = c.freq_deviation,
                "Configuring FSK channel"
            );

            self.rx_if_config[9].enable = true;
            self.rx_if_config[9].rf_chain = radio_i as u8;
            self.rx_if_config[9].freq_hz = radio_freq_offset;
            self.rx_if_config[9].bandwidth = bw as u8;
            self.rx_if_config[9].datarate = c.bitrate;
        }

        Ok(())
    }

    fn send(&mut self, frame: &TxCommand) -> Result<(), TxAckStatus> {
        if !self.started.load(Ordering::Relaxed) {
            return Err(TxAckStatus::InternalError);
        }

        let channel = self
            .downlink_channels
            .get(&frame.channel)
            .ok_or(TxAckStatus::TxChannel)?;

        let data_rate = self
            .data_rates
            .get(&frame.data_rate)
            .ok_or(TxAckStatus::TxDataRate)?;

        let tx_pkt = bindings::lgw_pkt_tx_s {
            freq_hz: channel.freq_hz,
            tx_mode: match &frame.timing {
                Timing::Delay(_) => bindings::TIMESTAMPED,
            } as u8,
            count_us: if let Timing::Delay(v) = &frame.timing {
                let context: [u8; 4] = v.context[0..4]
                    .try_into()
                    .map_err(|_| TxAckStatus::ContextError)?;
                let rx_count = u32::from_be_bytes(context);

                rx_count.wrapping_add(Duration::from_nanos(v.delay_ns).as_micros() as u32)
            } else {
                0
            },
            rf_chain: 0,
            rf_power: frame.tx_power as i8,
            modulation: match &data_rate {
                DataRateConfiguration::Lora(_) => bindings::MOD_LORA,
                DataRateConfiguration::Fsk(_) => bindings::MOD_FSK,
            } as u8,
            bandwidth: if let DataRateConfiguration::Lora(v) = &data_rate {
                match v.bandwidth {
                    125_000 => bindings::BW_125KHZ,
                    250_000 => bindings::BW_250KHZ,
                    500_000 => bindings::BW_500KHZ,
                    _ => return Err(TxAckStatus::InvalidFrame),
                }
            } else {
                0
            } as u8,
            datarate: match &data_rate {
                DataRateConfiguration::Lora(v) => v.spreading_factor as u32,
                DataRateConfiguration::Fsk(v) => v.bitrate,
            },
            coderate: if let DataRateConfiguration::Lora(v) = &data_rate {
                match v.coding_rate {
                    CodingRate::Cr45 => bindings::CR_LORA_4_5,
                    CodingRate::Cr46 => bindings::CR_LORA_4_6,
                    CodingRate::Cr47 => bindings::CR_LORA_4_7,
                    CodingRate::Cr48 => bindings::CR_LORA_4_8,
                }
            } else {
                0
            } as u8,
            invert_pol: matches!(data_rate, DataRateConfiguration::Lora(_)),
            f_dev: if let DataRateConfiguration::Fsk(v) = &data_rate {
                v.frequency_deviation / 1000
            } else {
                0
            } as u8,
            size: frame.payload.len() as u16,
            payload: {
                let mut out: [u8; 256] = [0; 256];
                let mut pl = frame.payload.clone();
                pl.resize(out.len(), 0);
                out.copy_from_slice(&pl);
                out
            },
            ..Default::default()
        };

        QUEUE.lock().unwrap().enqueue(
            get_count(get_instcnt().map_err(|_| TxAckStatus::InternalError)?),
            TxPacket {
                data_rate: frame.data_rate,
                channel: frame.channel,
                tx_id: frame.id,
                tx_packet: tx_pkt,
            },
        )
    }

    fn receive(&self) -> Result<Vec<RxEvent>> {
        // silenty return, as the receive loop might start before the radio has been fully
        // initialized.
        if !self.started.load(Ordering::Relaxed) {
            return Ok(vec![]);
        }

        let res = receive()?;
        Ok(res
            .iter()
            .filter_map(|v| {
                if v.status != bindings::STAT_CRC_OK as u8 {
                    return None;
                }

                let data_rate = match self.get_data_rate(v) {
                    Ok(v) => v,
                    Err(e) => {
                        warn!(error = %e, "Get data-rate error");
                        return None;
                    }
                };

                let channel = match self.get_uplink_channel_index(v.freq_hz, data_rate) {
                    Ok(v) => v,
                    Err(e) => {
                        warn!(error = %e, "Get channel error");
                        return None;
                    }
                };

                let rx_id = getrandom::u32().unwrap_or_default();

                info!(
                    rx_id = rx_id,
                    channel = channel,
                    data_rate = data_rate,
                    rssi = v.rssis,
                    snr = v.snr,
                    "Received frame"
                );

                Some(RxEvent {
                    rx_id,
                    channel,
                    data_rate,
                    payload: v.payload[..v.size as usize].to_vec(),
                    rssi: v.rssis,
                    snr: v.snr,
                    rx_timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos() as u64,
                    context: v.count_us.to_be_bytes().into(),
                })
            })
            .collect())
    }
}

pub fn get_count(concentrator_count: u32) -> Duration {
    let mut mapping = CONCENTRATOR_COUNT.lock().unwrap();

    // If the two values are close together, this can still be a very large diff because of the
    // wrapping_sub. Below we will handle a diff > u32::MAX / 2 as negative.
    let diff_us = concentrator_count.wrapping_sub(mapping.0);

    if !mapping.1.is_zero() && diff_us > u32::MAX / 2 {
        mapping.1 - Duration::from_micros(mapping.0.wrapping_sub(concentrator_count) as u64)
    } else {
        mapping.1 += Duration::from_micros(diff_us as u64);
        mapping.0 = concentrator_count;
        mapping.1
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_count() {
        let count = Duration::from_micros(u32::MAX as u64);
        assert_eq!(count, get_count(u32::MAX));

        // u32 rollover
        let count = Duration::from_micros(u32::MAX as u64 + 1);
        assert_eq!(count, get_count(0));
        assert_eq!(count, get_count(0)); // no increment

        // an increment
        let count = Duration::from_micros(u32::MAX as u64 + 501);
        assert_eq!(count, get_count(500));

        // a slightly lower value, this should not be handled as a rollover
        // as the value is too close to the previous counter value
        let count = Duration::from_micros(u32::MAX as u64 + 101);
        assert_eq!(count, get_count(100));
    }
}
