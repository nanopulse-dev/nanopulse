use std::collections::HashMap;

use minicbor::{Decode, Encode};

use crate::common::CodingRate;

#[derive(Default, Debug, Clone, Copy, Encode, Decode)]
#[cbor(index_only)]
pub enum TxAckStatus {
    #[default]
    #[n(0)]
    Ok,
    #[n(1)]
    TooLate,
    #[n(2)]
    TooEarly,
    #[n(3)]
    CollisionPacket,
    #[n(4)]
    TxChannel,
    #[n(5)]
    TxDataRate,
    #[n(6)]
    TxPower,
    #[n(7)]
    QueueFull,
    #[n(8)]
    InternalError,
    #[n(9)]
    ContextError,
    #[n(10)]
    InvalidFrame,
}

#[derive(Default, Debug, Clone, Encode, Decode)]
pub struct RxEvent {
    // Rx ID.
    #[n(0)]
    pub rx_id: u32,

    // Received payload.
    #[cbor(n(1), with = "minicbor::bytes")]
    pub payload: Vec<u8>,

    // Channel number on which the frame was received.
    #[n(2)]
    pub channel: u8,

    // Data-rate on which the frame was received.
    #[n(3)]
    pub data_rate: u8,

    // RSSI.
    #[n(4)]
    pub rssi: f32,

    // SNR (if available).
    #[n(5)]
    pub snr: f32,

    // Rx timestamp.
    #[n(6)]
    pub rx_timestamp: u64,

    // Context blob.
    #[cbor(n(7), with = "minicbor::bytes")]
    pub context: Vec<u8>,
}

#[derive(Default, Debug, Clone, Encode, Decode)]
pub struct HeartbeatEvent {
    // Gateway timestamp.
    #[n(0)]
    pub gw_time: u64,

    // Channel config version.
    #[n(1)]
    pub config_version: String,
}

#[derive(Default, Debug, Clone, Encode, Decode)]
pub struct TxAckEvent {
    // Tx ID.
    #[n(0)]
    pub id: u32,

    #[n(1)]
    pub status: TxAckStatus,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct TxCommand {
    // Tx ID.
    #[n(0)]
    pub id: u32,

    // Payload to send.
    #[cbor(n(1), with = "minicbor::bytes")]
    pub payload: Vec<u8>,

    // Channel number.
    #[n(2)]
    pub channel: u8,

    // Data-rate.
    #[n(3)]
    pub data_rate: u8,

    // Tx power (dBm EIRP).
    #[n(4)]
    pub tx_power: i32,

    // Timing.
    #[n(5)]
    pub timing: Timing,
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum Timing {
    #[n(0)]
    Delay(#[n(0)] TimingDelay),
}

#[derive(Default, Debug, Clone, Encode, Decode)]
pub struct TimingDelay {
    // Delay (ns).
    #[n(0)]
    pub delay_ns: u64,

    // Rx context.
    #[cbor(n(1), with = "minicbor::bytes")]
    pub context: Vec<u8>,
}

#[derive(Default, Debug, Clone, Encode, Decode)]
pub struct ConfigCommand {
    // Configuration version.
    #[n(0)]
    pub version: String,

    // Uplink channels.
    #[n(1)]
    pub uplink_channels: HashMap<u8, ChannelConfiguration>,

    // Downlink channels.
    #[n(2)]
    pub downlink_channels: HashMap<u8, ChannelConfiguration>,

    // Data-rates.
    #[n(3)]
    pub data_rates: HashMap<u8, DataRateConfiguration>,
}

#[derive(Default, Debug, Clone, Encode, Decode)]
pub struct ChannelConfiguration {
    // Frequency Hz.
    #[n(0)]
    pub freq_hz: u32,

    // Data-rates.
    #[n(1)]
    pub data_rates: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
pub enum DataRateConfiguration {
    #[n(0)]
    Lora(#[n(0)] LoraDataRateConfiguration),
    #[n(1)]
    Fsk(#[n(1)] FskDataRateConfiguration),
}

#[derive(Default, Debug, Clone, Encode, Decode, PartialEq, Eq)]
pub struct LoraDataRateConfiguration {
    // Bandwidth (Hz).
    #[n(0)]
    pub bandwidth: u32,

    // Spreading factor.
    #[n(1)]
    pub spreading_factor: u8,

    // Coding-rate.
    #[n(2)]
    pub coding_rate: CodingRate,
}

#[derive(Default, Debug, Clone, Encode, Decode, PartialEq, Eq)]
pub struct FskDataRateConfiguration {
    // Frequency deviation.
    #[n(0)]
    pub frequency_deviation: u32,

    // FSK datarate (bits /sec).
    #[n(1)]
    pub bitrate: u32,
}
