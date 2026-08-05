use defmt::{debug, error, info, warn};
use embassy_time::{Duration, Instant, Timer};
use lora_modulation::BaseBandModulationParams;
use lora_phy::mod_traits::RadioKind;
use lora_phy::{DelayNs, LoRa, RxMode};

use super::Radio;
use crate::data_rate::DataRate;
use crate::errors::Error;

impl<RK, DLY> Radio for LoRa<RK, DLY>
where
    RK: RadioKind,
    DLY: DelayNs,
{
    // TODO: this needs to be adjusted
    const PRE_RX_MARGIN_MS: Duration = Duration::from_millis(100);
    const CLOCK_DRIFT_PPM: u64 = 50;

    async fn send(&mut self, freq: u32, dr: &DataRate, buf: &[u8]) -> Result<(), Error> {
        let mod_params = match dr {
            DataRate::Lora(v) => self
                .create_modulation_params(v.spreading_factor, v.bandwidth, v.coding_rate, freq)
                .map_err(|e| {
                    error!("create_modulation_params error: {}", e);
                    Error::RadioError
                })?,
            DataRate::Fsk => return Err(Error::UnsupportedModulation),
        };

        let mut tx_params = self
            .create_tx_packet_params(8, false, true, false, &mod_params)
            .map_err(|e| {
                error!("create_tx_packet_params error: {}", e);
                Error::RadioError
            })?;
        debug!("create_tx_packet_params ok");

        self.prepare_for_tx(&mod_params, &mut tx_params, 14, buf)
            .await
            .map_err(|e| {
                error!("prepare_for_tx error: {}", e);
                Error::RadioError
            })?;
        debug!("prepare_for_tx ok");

        self.tx().await.map_err(|e| {
            error!("tx error: {}", e);
            Error::RadioError
        })?;
        debug!("tx ok");

        Ok(())
    }

    async fn send_sleep_receive(
        &mut self,
        tx_freq: u32,
        rx_freq: u32,
        tx_dr: &DataRate,
        rx_dr: &DataRate,
        tx_buf: &[u8],
        rx_buf: &mut [u8],
        delay: Duration,
    ) -> Result<u8, Error> {
        let tx_mod_params = match tx_dr {
            DataRate::Lora(v) => self
                .create_modulation_params(v.spreading_factor, v.bandwidth, v.coding_rate, tx_freq)
                .map_err(|e| {
                    error!("create_modulation_params error: {}", e);
                    Error::RadioError
                })?,
            DataRate::Fsk => return Err(Error::UnsupportedModulation),
        };
        let mut tx_params = self
            .create_tx_packet_params(8, false, true, false, &tx_mod_params)
            .map_err(|e| {
                error!("create_tx_packet_params error: {}", e);
                Error::RadioError
            })?;
        debug!("create_tx_packet_params ok");

        let rx_mod_params = match rx_dr {
            DataRate::Lora(v) => self
                .create_modulation_params(v.spreading_factor, v.bandwidth, v.coding_rate, rx_freq)
                .map_err(|e| {
                    error!("create_modulation_params error: {}", e);
                    Error::RadioError
                })?,
            DataRate::Fsk => return Err(Error::UnsupportedModulation),
        };
        let rx_params = self
            .create_rx_packet_params(8, false, rx_buf.len() as u8, true, true, &rx_mod_params)
            .map_err(|e| {
                error!("create_rx_packet_params error: {}", e);
                Error::RadioError
            })?;
        let rx_bb_params = match rx_dr {
            DataRate::Lora(v) => {
                BaseBandModulationParams::new(v.spreading_factor, v.bandwidth, v.coding_rate)
            }
            DataRate::Fsk => return Err(Error::UnsupportedModulation),
        };

        let clock_drift = self.get_clock_drift(delay);
        let sleep_duration = delay - Self::PRE_RX_MARGIN_MS - clock_drift;
        let symbol_margin = Self::PRE_RX_MARGIN_MS + clock_drift + clock_drift;
        let symbols_margin = rx_bb_params.delay_in_symbols(symbol_margin.as_millis() as u32);

        self.prepare_for_tx(&tx_mod_params, &mut tx_params, 14, tx_buf)
            .await
            .map_err(|e| {
                error!("prepare_for_tx error: {}", e);
                Error::RadioError
            })?;
        debug!("prepare_for_tx ok");

        self.tx().await.map_err(|e| {
            error!("tx error: {}", e);
            Error::RadioError
        })?;

        let tx_ts = Instant::now();
        info!("tx ok, tx_ts: {}", tx_ts);

        Timer::at(tx_ts + sleep_duration).await;
        self.prepare_for_rx(
            RxMode::Single(6 + symbols_margin),
            &rx_mod_params,
            &rx_params,
        )
        .await
        .map_err(|e| {
            error!("prepare_for_rx error: {}", e);
            Error::RadioError
        })?;

        match self.rx(&rx_params, rx_buf).await {
            Ok((rx_len, _rx_status)) => {
                info!("rx ok");
                Ok(rx_len)
            }
            Err(e) => {
                warn!("rx error: {}", e);
                Ok(0)
            }
        }
    }
}
