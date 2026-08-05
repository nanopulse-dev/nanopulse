use std::collections::HashMap;

use anyhow::{Result, anyhow};

use nanopulse_structs::gateway::{ChannelConfiguration, DataRateConfiguration};

const DEFAULT_RADIO_BANDWIDTH: u32 = 925000;

pub struct LoraMultiSfChannel {
    pub freq_hz: u32,
    pub bandwidth: u32,
}

pub struct LoraStdChannel {
    pub freq_hz: u32,
    pub bandwidth: u32,
    pub spreading_factor: u8,
}

pub struct FskChannel {
    pub freq_hz: u32,
    pub freq_deviation: u32,
    pub bitrate: u32,
    pub bandwidth: u32,
}

#[derive(Clone)]
struct Channel {
    pub freq_hz: u32,
    pub bandwidth: u32,
}

impl Channel {
    fn min_radio_freq(&self) -> u32 {
        let radio_bw = get_radio_bandwidth(self.bandwidth);
        self.freq_hz - (self.bandwidth / 2) + (radio_bw / 2)
    }

    fn min_channel_freq(&self) -> u32 {
        self.freq_hz - (self.bandwidth / 2)
    }

    fn max_channel_freq(&self) -> u32 {
        self.freq_hz + (self.bandwidth / 2)
    }
}

fn get_radio_bandwidth(channel_bw: u32) -> u32 {
    match channel_bw {
        500000 => 1100000,
        250000 => 1000000,
        125000 => 925000,
        _ => DEFAULT_RADIO_BANDWIDTH,
    }
}

pub fn get_radio_frequencies(
    dr_config: &HashMap<u8, DataRateConfiguration>,
    channel_config: &HashMap<u8, ChannelConfiguration>,
) -> Result<[u32; 2]> {
    let mut radios = [0u32; 2];

    let multi_sf_channels = get_lora_multi_fs_channels(dr_config, channel_config)?;
    let lora_std_channel = get_lora_std_channel(dr_config, channel_config);
    let fsk_channel = get_fsk_channel(dr_config, channel_config);

    // combine all channes into one vec
    let mut channels: Vec<Channel> = multi_sf_channels
        .iter()
        .map(|v| Channel {
            freq_hz: v.freq_hz,
            bandwidth: v.bandwidth,
        })
        .collect();
    if let Some(c) = lora_std_channel {
        channels.push(Channel {
            freq_hz: c.freq_hz,
            bandwidth: c.bandwidth,
        });
    }
    if let Some(c) = fsk_channel {
        channels.push(Channel {
            freq_hz: c.freq_hz,
            bandwidth: c.bandwidth,
        });
    }

    // sort vector by min radio freq
    channels.sort_by_key(|c| c.min_radio_freq());

    for c in &channels {
        let channel_max = c.max_channel_freq();
        let radio_bw = get_radio_bandwidth(c.bandwidth);
        let min_radio_center_freq = c.min_channel_freq() + (radio_bw / 2);
        let radio_count = radios.len();

        for (i, radio_freq) in radios.iter_mut().enumerate() {
            // the radio is not defined yet, use it
            if *radio_freq == 0 {
                *radio_freq = min_radio_center_freq;
                break;
            }

            // channel fits within bandwidth of radio
            if channel_max <= *radio_freq + (radio_bw / 2) {
                break;
            }

            if i == radio_count - 1 {
                return Err(anyhow!(
                    "the channels do not fit within the bandwidth of the two radios"
                ));
            }
        }
    }

    Ok(radios)
}

pub fn get_radio_for_channel(radios: &[u32], freq_hz: u32, bandwidth: u32) -> Result<usize> {
    let chan_min = freq_hz - (bandwidth / 2);
    let chan_max = freq_hz + (bandwidth / 2);

    let radio_bandwidth = get_radio_bandwidth(bandwidth);
    for (i, radio_freq) in radios.iter().enumerate() {
        if chan_min >= radio_freq - (radio_bandwidth / 2)
            && chan_max <= radio_freq + (radio_bandwidth / 2)
        {
            return Ok(i);
        }
    }

    Err(anyhow!("channel does not fit in radio bandwidth"))
}

pub fn get_lora_multi_fs_channels(
    dr_config: &HashMap<u8, DataRateConfiguration>,
    channel_config: &HashMap<u8, ChannelConfiguration>,
) -> Result<Vec<LoraMultiSfChannel>> {
    let mut channels = vec![];
    let mut keys: Vec<_> = channel_config.keys().collect();
    keys.sort();

    for key in keys {
        let c = channel_config.get(key).unwrap();
        if c.data_rates.len() > 1 {
            let bandwidth = dr_config
                .get(&c.data_rates[0])
                .map(|v| match v {
                    DataRateConfiguration::Lora(v) => v.bandwidth,
                    _ => 0,
                })
                .unwrap_or_default();
            channels.push(LoraMultiSfChannel {
                freq_hz: c.freq_hz,
                bandwidth,
            });
        }
    }

    if channels.len() > 8 {
        return Err(anyhow!(
            "max lora multi-sf channels is 8, got: {}",
            channels.len()
        ));
    }

    Ok(channels)
}

pub fn get_lora_std_channel(
    dr_config: &HashMap<u8, DataRateConfiguration>,
    channel_config: &HashMap<u8, ChannelConfiguration>,
) -> Option<LoraStdChannel> {
    let mut keys: Vec<_> = channel_config.keys().collect();
    keys.sort();

    for key in keys {
        let c = channel_config.get(key).unwrap();
        if c.data_rates.len() == 1
            && let Some(dr) = dr_config.get(&c.data_rates[0])
                && let DataRateConfiguration::Lora(v) = &dr {
                    return Some(LoraStdChannel {
                        freq_hz: c.freq_hz,
                        bandwidth: v.bandwidth,
                        spreading_factor: v.spreading_factor,
                    });
                }
    }

    None
}

pub fn get_fsk_channel(
    dr_config: &HashMap<u8, DataRateConfiguration>,
    channel_config: &HashMap<u8, ChannelConfiguration>,
) -> Option<FskChannel> {
    let mut keys: Vec<_> = channel_config.keys().collect();
    keys.sort();

    for key in keys {
        let c = channel_config.get(key).unwrap();
        if c.data_rates.len() == 1
            && let Some(dr) = dr_config.get(&c.data_rates[0])
                && let DataRateConfiguration::Fsk(v) = &dr {
                    return Some(FskChannel {
                        freq_hz: c.freq_hz,
                        freq_deviation: v.frequency_deviation,
                        bitrate: v.bitrate,
                        bandwidth: {
                            let bw = 2 * v.frequency_deviation + v.bitrate;
                            if bw == 0 {
                                0
                            } else if bw <= 125000 {
                                125000
                            } else if bw <= 250000 {
                                250000
                            } else if bw <= 500000 {
                                500000
                            } else {
                                0
                            }
                        },
                    });
                }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use nanopulse_structs::common::CodingRate;
    use nanopulse_structs::gateway::{
        ConfigCommand, FskDataRateConfiguration, LoraDataRateConfiguration,
    };

    #[test]
    fn test_get_radio_frequencies() {
        let data_rates: HashMap<u8, DataRateConfiguration> = [
            (
                0,
                DataRateConfiguration::Lora(LoraDataRateConfiguration {
                    bandwidth: 125000,
                    spreading_factor: 12,
                    coding_rate: CodingRate::Cr45,
                }),
            ),
            (
                1,
                DataRateConfiguration::Lora(LoraDataRateConfiguration {
                    bandwidth: 125000,
                    spreading_factor: 11,
                    coding_rate: CodingRate::Cr45,
                }),
            ),
            (
                2,
                DataRateConfiguration::Lora(LoraDataRateConfiguration {
                    bandwidth: 125000,
                    spreading_factor: 10,
                    coding_rate: CodingRate::Cr45,
                }),
            ),
            (
                3,
                DataRateConfiguration::Lora(LoraDataRateConfiguration {
                    bandwidth: 125000,
                    spreading_factor: 9,
                    coding_rate: CodingRate::Cr45,
                }),
            ),
            (
                4,
                DataRateConfiguration::Lora(LoraDataRateConfiguration {
                    bandwidth: 125000,
                    spreading_factor: 8,
                    coding_rate: CodingRate::Cr45,
                }),
            ),
            (
                5,
                DataRateConfiguration::Lora(LoraDataRateConfiguration {
                    bandwidth: 125000,
                    spreading_factor: 7,
                    coding_rate: CodingRate::Cr45,
                }),
            ),
            (
                6,
                DataRateConfiguration::Lora(LoraDataRateConfiguration {
                    bandwidth: 250000,
                    spreading_factor: 7,
                    coding_rate: CodingRate::Cr45,
                }),
            ),
            (
                7,
                DataRateConfiguration::Fsk(FskDataRateConfiguration {
                    bitrate: 50000,
                    frequency_deviation: 25000,
                }),
            ),
        ]
        .iter()
        .cloned()
        .collect();

        let data_rates_us: HashMap<u8, DataRateConfiguration> = [
            (
                0,
                DataRateConfiguration::Lora(LoraDataRateConfiguration {
                    bandwidth: 125000,
                    spreading_factor: 10,
                    coding_rate: CodingRate::Cr45,
                }),
            ),
            (
                1,
                DataRateConfiguration::Lora(LoraDataRateConfiguration {
                    bandwidth: 125000,
                    spreading_factor: 9,
                    coding_rate: CodingRate::Cr45,
                }),
            ),
            (
                2,
                DataRateConfiguration::Lora(LoraDataRateConfiguration {
                    bandwidth: 125000,
                    spreading_factor: 8,
                    coding_rate: CodingRate::Cr45,
                }),
            ),
            (
                3,
                DataRateConfiguration::Lora(LoraDataRateConfiguration {
                    bandwidth: 125000,
                    spreading_factor: 9,
                    coding_rate: CodingRate::Cr45,
                }),
            ),
            (
                4,
                DataRateConfiguration::Lora(LoraDataRateConfiguration {
                    bandwidth: 500000,
                    spreading_factor: 8,
                    coding_rate: CodingRate::Cr45,
                }),
            ),
        ]
        .iter()
        .cloned()
        .collect();

        let tests = vec![
            (
                ConfigCommand {
                    data_rates: data_rates.clone(),
                    uplink_channels: [(
                        0,
                        ChannelConfiguration {
                            freq_hz: 868100000,
                            data_rates: vec![0, 1, 2, 3, 4, 5],
                        },
                    )]
                    .iter()
                    .cloned()
                    .collect(),
                    ..Default::default()
                },
                [868500000, 0],
            ),
            (
                ConfigCommand {
                    data_rates: data_rates.clone(),
                    uplink_channels: [
                        (
                            0,
                            ChannelConfiguration {
                                freq_hz: 868100000,
                                data_rates: vec![0, 1, 2, 3, 4, 5],
                            },
                        ),
                        (
                            1,
                            ChannelConfiguration {
                                freq_hz: 868300000,
                                data_rates: vec![0, 1, 2, 3, 4, 5],
                            },
                        ),
                        (
                            2,
                            ChannelConfiguration {
                                freq_hz: 868500000,
                                data_rates: vec![0, 1, 2, 3, 4, 5],
                            },
                        ),
                    ]
                    .iter()
                    .cloned()
                    .collect(),
                    ..Default::default()
                },
                [868500000, 0],
            ),
            (
                ConfigCommand {
                    data_rates: data_rates.clone(),
                    uplink_channels: [
                        (
                            0,
                            ChannelConfiguration {
                                freq_hz: 868100000,
                                data_rates: vec![0, 1, 2, 3, 4, 5],
                            },
                        ),
                        (
                            1,
                            ChannelConfiguration {
                                freq_hz: 868300000,
                                data_rates: vec![0, 1, 2, 3, 4, 5],
                            },
                        ),
                        (
                            2,
                            ChannelConfiguration {
                                freq_hz: 868500000,
                                data_rates: vec![0, 1, 2, 3, 4, 5],
                            },
                        ),
                        (
                            3,
                            ChannelConfiguration {
                                freq_hz: 867100000,
                                data_rates: vec![0, 1, 2, 3, 4, 5],
                            },
                        ),
                        (
                            4,
                            ChannelConfiguration {
                                freq_hz: 867300000,
                                data_rates: vec![0, 1, 2, 3, 4, 5],
                            },
                        ),
                        (
                            5,
                            ChannelConfiguration {
                                freq_hz: 867500000,
                                data_rates: vec![0, 1, 2, 3, 4, 5],
                            },
                        ),
                        (
                            6,
                            ChannelConfiguration {
                                freq_hz: 867700000,
                                data_rates: vec![0, 1, 2, 3, 4, 5],
                            },
                        ),
                        (
                            7,
                            ChannelConfiguration {
                                freq_hz: 867900000,
                                data_rates: vec![0, 1, 2, 3, 4, 5],
                            },
                        ),
                        (
                            8,
                            ChannelConfiguration {
                                freq_hz: 868300000,
                                data_rates: vec![6],
                            },
                        ),
                        (
                            8,
                            ChannelConfiguration {
                                freq_hz: 868000000,
                                data_rates: vec![7],
                            },
                        ),
                    ]
                    .iter()
                    .cloned()
                    .collect(),
                    ..Default::default()
                },
                [867500000, 868400000],
            ),
            (
                ConfigCommand {
                    data_rates: data_rates_us.clone(),
                    uplink_channels: [
                        (
                            0,
                            ChannelConfiguration {
                                freq_hz: 902300000,
                                data_rates: vec![0, 1, 2, 3],
                            },
                        ),
                        (
                            1,
                            ChannelConfiguration {
                                freq_hz: 902500000,
                                data_rates: vec![0, 1, 2, 3],
                            },
                        ),
                        (
                            2,
                            ChannelConfiguration {
                                freq_hz: 902700000,
                                data_rates: vec![0, 1, 2, 3],
                            },
                        ),
                        (
                            3,
                            ChannelConfiguration {
                                freq_hz: 902900000,
                                data_rates: vec![0, 1, 2, 3],
                            },
                        ),
                        (
                            4,
                            ChannelConfiguration {
                                freq_hz: 903100000,
                                data_rates: vec![0, 1, 2, 3],
                            },
                        ),
                        (
                            5,
                            ChannelConfiguration {
                                freq_hz: 903300000,
                                data_rates: vec![0, 1, 2, 3],
                            },
                        ),
                        (
                            6,
                            ChannelConfiguration {
                                freq_hz: 903500000,
                                data_rates: vec![0, 1, 2, 3],
                            },
                        ),
                        (
                            7,
                            ChannelConfiguration {
                                freq_hz: 903700000,
                                data_rates: vec![0, 1, 2, 3],
                            },
                        ),
                        (
                            8,
                            ChannelConfiguration {
                                freq_hz: 903000000,
                                data_rates: vec![4],
                            },
                        ),
                    ]
                    .iter()
                    .cloned()
                    .collect(),
                    ..Default::default()
                },
                [902700000, 903700000],
            ),
        ];

        for (cmd, expected) in tests {
            assert_eq!(
                expected,
                get_radio_frequencies(&cmd.data_rates, &cmd.uplink_channels).unwrap()
            );
        }
    }

    #[test]
    fn test_get_radio_for_channel() {
        let tests = vec![("Radio 0".to_string(), [868500000, 0], 868100000, 125000, 0)];

        for (_, radios, freq_hz, bandwidth, expected) in tests {
            let radio = super::get_radio_for_channel(&radios, freq_hz, bandwidth).unwrap();
            assert_eq!(radio, expected);
        }
    }
}
