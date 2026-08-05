use core::future::Future;

use embassy_time::Duration;

use crate::data_rate::DataRate;
use crate::errors::Error;

#[cfg(feature = "lora_sx126x")]
pub mod lora_sx126x;

pub trait Radio {
    // PRE_RX_MARGIN_MS defines the margin that is needed to wake up, turn on the radio and
    // configure it in RX mode.
    const PRE_RX_MARGIN_MS: Duration;
    const CLOCK_DRIFT_PPM: u64;

    // Send a single frame.
    fn send(
        &mut self,
        freq: u32,
        dr: &DataRate,
        buf: &[u8],
    ) -> impl Future<Output = Result<(), Error>>;

    // Send a frame, and after the given sleep duration, wake up and try to receive.
    #[allow(clippy::too_many_arguments)]
    fn send_sleep_receive(
        &mut self,
        tx_freq: u32,
        rx_freq: u32,
        tx_dr: &DataRate,
        rx_dr: &DataRate,
        tx_buf: &[u8],
        rx_buf: &mut [u8],
        sleep: Duration,
    ) -> impl Future<Output = Result<u8, Error>>;

    // Return the clock drift for the given interval.
    fn get_clock_drift(&self, interval: Duration) -> Duration {
        Duration::from_micros(interval.as_micros() * Self::CLOCK_DRIFT_PPM / 1_000_000)
    }
}
