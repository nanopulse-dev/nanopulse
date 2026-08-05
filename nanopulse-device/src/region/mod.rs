use crate::data_rate::DataRate;

#[cfg(feature = "lora_eu868")]
pub mod lora;

#[cfg(feature = "lora_eu868")]
pub use lora::eu868::*;

pub struct Region<
    const UPLINK_CHANNELS: usize,
    const DOWNLINK_CHANNELS: usize,
    const DATA_RATES: usize,
> {
    pub uplink_channels: [u32; UPLINK_CHANNELS],
    pub downlink_channels: [u32; DOWNLINK_CHANNELS],
    pub data_rates: [DataRate; DATA_RATES],
    pub uplink_downlink_channel_mapping: [usize; UPLINK_CHANNELS],
    pub uplink_downlink_data_rate_mapping: [usize; DATA_RATES],
    pub timing_multiplier_ms: u64,
    pub tx_delay_key_exchange_response: u64,
    pub tx_delay_join_accept: u64,
    pub tx_delay_data: u64,
}
