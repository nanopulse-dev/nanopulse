use crate::data_rate::{DataRate, lora};
use crate::region::Region;

pub static REGION: Region<8, 8, 6> = Region {
    uplink_channels: [
        868100000, 868300000, 868500000, 867100000, 867300000, 867500000, 867700000, 867900000,
    ],
    downlink_channels: [
        868100000, 868300000, 868500000, 867100000, 867300000, 867500000, 867700000, 867900000,
    ],
    data_rates: [
        DataRate::Lora(lora::Lora {
            spreading_factor: lora::SpreadingFactor::_12,
            coding_rate: lora::CodingRate::_4_5,
            bandwidth: lora::Bandwidth::_125KHz,
        }),
        DataRate::Lora(lora::Lora {
            spreading_factor: lora::SpreadingFactor::_11,
            coding_rate: lora::CodingRate::_4_5,
            bandwidth: lora::Bandwidth::_125KHz,
        }),
        DataRate::Lora(lora::Lora {
            spreading_factor: lora::SpreadingFactor::_10,
            coding_rate: lora::CodingRate::_4_5,
            bandwidth: lora::Bandwidth::_125KHz,
        }),
        DataRate::Lora(lora::Lora {
            spreading_factor: lora::SpreadingFactor::_9,
            coding_rate: lora::CodingRate::_4_5,
            bandwidth: lora::Bandwidth::_125KHz,
        }),
        DataRate::Lora(lora::Lora {
            spreading_factor: lora::SpreadingFactor::_8,
            coding_rate: lora::CodingRate::_4_5,
            bandwidth: lora::Bandwidth::_125KHz,
        }),
        DataRate::Lora(lora::Lora {
            spreading_factor: lora::SpreadingFactor::_7,
            coding_rate: lora::CodingRate::_4_5,
            bandwidth: lora::Bandwidth::_125KHz,
        }),
    ],
    uplink_downlink_channel_mapping: [0, 1, 2, 3, 4, 5, 6, 7],
    uplink_downlink_data_rate_mapping: [0, 1, 2, 3, 4, 5],
    timing_multiplier_ms: 1_000,
    tx_delay_key_exchange_response: 5,
    tx_delay_join_accept: 5,
    tx_delay_data: 1,
};
