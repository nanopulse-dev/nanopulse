use heapless::Vec;
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

use crate::BUFFER_SIZE;
use crate::errors::Error;

pub mod activation_request;
pub mod activation_response;
pub mod key_exchange_request;
pub mod key_exchange_response;
pub mod state;
pub mod telemetry;

pub type Buffer = Vec<u8, BUFFER_SIZE>;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameType {
    KeyExchangeRequest = 0x00,
    KeyExchangeResponse = 0x01,
    ActivationRequest = 0x02,
    ActivationResponse = 0x03,
    Telemetry = 0x06,
    State = 0x07,
}

impl FrameType {
    fn has_mic(&self) -> bool {
        !matches!(self, FrameType::KeyExchangeRequest)
    }
}

impl TryFrom<u8> for FrameType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value >> 4 {
            0x00 => Ok(FrameType::KeyExchangeRequest),
            0x01 => Ok(FrameType::KeyExchangeResponse),
            0x02 => Ok(FrameType::ActivationRequest),
            0x03 => Ok(FrameType::ActivationResponse),
            0x06 => Ok(FrameType::Telemetry),
            0x07 => Ok(FrameType::State),
            _ => Err(Error::InvalidFrameType),
        }
    }
}

impl From<FrameType> for u8 {
    fn from(value: FrameType) -> Self {
        (value as u8) << 4
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payload {
    KeyExchangeRequest(key_exchange_request::KeyExchangeRequest),
    KeyExchangeResponse(key_exchange_response::KeyExchangeResponse),
    ActivationRequest(activation_request::ActivationRequest),
    ActivationResponse(activation_response::ActivationResponse),
    Telemetry(telemetry::Telemetry),
    State(state::State),
}

impl Payload {
    fn encrypt(&mut self, enc_key: &[u8; 32]) -> Result<(), Error> {
        match self {
            Payload::ActivationRequest(v) => v.encrypt(enc_key),
            Payload::ActivationResponse(v) => v.encrypt(enc_key),
            Payload::Telemetry(v) => v.encrypt(enc_key),
            Payload::State(v) => v.encrypt(enc_key),
            _ => Ok(()),
        }
    }

    fn decrypt(&mut self, enc_key: &[u8; 32]) -> Result<(), Error> {
        match self {
            Payload::ActivationRequest(v) => v.decrypt(enc_key),
            Payload::ActivationResponse(v) => v.decrypt(enc_key),
            Payload::Telemetry(v) => v.decrypt(enc_key),
            Payload::State(v) => v.decrypt(enc_key),
            _ => Ok(()),
        }
    }
}

impl TryFrom<&[u8]> for Payload {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(Error::NotEnoughBytes);
        }

        let frame_type = FrameType::try_from(value[0])?;

        Ok(match frame_type {
            FrameType::KeyExchangeRequest => Payload::KeyExchangeRequest(
                key_exchange_request::KeyExchangeRequest::try_from(value)?,
            ),
            FrameType::KeyExchangeResponse => Payload::KeyExchangeResponse(
                key_exchange_response::KeyExchangeResponse::try_from(value)?,
            ),
            FrameType::ActivationRequest => {
                Payload::ActivationRequest(activation_request::ActivationRequest::try_from(value)?)
            }
            FrameType::ActivationResponse => Payload::ActivationResponse(
                activation_response::ActivationResponse::try_from(value)?,
            ),
            FrameType::Telemetry => Payload::Telemetry(telemetry::Telemetry::try_from(value)?),
            FrameType::State => Payload::State(state::State::try_from(value)?),
        })
    }
}

impl TryFrom<&Payload> for Buffer {
    type Error = Error;

    fn try_from(value: &Payload) -> Result<Self, Self::Error> {
        match value {
            Payload::KeyExchangeRequest(v) => Ok(v.into()),
            Payload::KeyExchangeResponse(v) => Ok(v.into()),
            Payload::ActivationRequest(v) => v.try_into(),
            Payload::ActivationResponse(v) => v.try_into(),
            Payload::Telemetry(v) => Ok(v.into()),
            Payload::State(v) => Ok(v.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub payload: Payload,
    pub mic: Option<[u8; 4]>,
}

impl Frame {
    pub fn frame_type(&self) -> FrameType {
        match &self.payload {
            Payload::KeyExchangeRequest(_) => FrameType::KeyExchangeRequest,
            Payload::KeyExchangeResponse(_) => FrameType::KeyExchangeResponse,
            Payload::ActivationRequest(_) => FrameType::ActivationRequest,
            Payload::ActivationResponse(_) => FrameType::ActivationResponse,
            Payload::Telemetry(_) => FrameType::Telemetry,
            Payload::State(_) => FrameType::State,
        }
    }

    pub fn set_mic(&mut self, mic_key: &[u8; 32]) -> Result<(), Error> {
        self.mic = Some(self.calculate_mic(mic_key)?);
        Ok(())
    }

    pub fn validate_mic(&self, mic_key: &[u8; 32]) -> Result<bool, Error> {
        if let Some(mic) = &self.mic {
            let calculated_mic = self.calculate_mic(mic_key)?;
            Ok(mic == &calculated_mic)
        } else {
            Ok(false)
        }
    }

    pub fn encrypt(&mut self, enc_key: &[u8; 32]) -> Result<(), Error> {
        self.payload.encrypt(enc_key)
    }

    pub fn decrypt(&mut self, enc_key: &[u8; 32]) -> Result<(), Error> {
        self.payload.decrypt(enc_key)
    }

    fn calculate_mic(&self, mic_key: &[u8; 32]) -> Result<[u8; 4], Error> {
        let mut mac = Hmac::<Sha256>::new_from_slice(mic_key).map_err(|_| Error::NotEnoughBytes)?;

        let b: &[u8] = match &self.payload {
            Payload::KeyExchangeResponse(pl) => &Into::<[u8; 37]>::into(pl),
            Payload::ActivationRequest(pl) => &TryInto::<[u8; 17]>::try_into(pl)?,
            Payload::ActivationResponse(pl) => &TryInto::<[u8; 7]>::try_into(pl)?,
            Payload::Telemetry(pl) => &Buffer::from(pl),
            Payload::State(pl) => &Buffer::from(pl),
            _ => return Err(Error::InvalidPayload),
        };

        mac.update(b);
        let result = mac.finalize().into_bytes();
        Ok(result[0..4].try_into().unwrap())
    }
}

impl TryFrom<&[u8]> for Frame {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let value_len = value.len();

        if value_len == 0 {
            return Err(Error::NotEnoughBytes);
        }

        let frame_type = FrameType::try_from(value[0])?;
        let has_mic = frame_type.has_mic();
        let mic: Option<[u8; 4]> = if has_mic {
            if value_len < 5 {
                return Err(Error::NotEnoughBytes);
            }
            Some(value[value_len - 4..value_len].try_into().unwrap())
        } else {
            None
        };

        Ok(Frame {
            mic,
            payload: if has_mic {
                Payload::try_from(&value[..value_len - 4])?
            } else {
                Payload::try_from(value)?
            },
        })
    }
}

impl TryFrom<&Frame> for Buffer {
    type Error = Error;

    fn try_from(value: &Frame) -> Result<Self, Self::Error> {
        let mut out = Vec::try_from(&value.payload)?;
        if value.frame_type().has_mic()
            && let Some(mic) = &value.mic
        {
            out.extend_from_slice(mic).unwrap();
        }

        Ok(out)
    }
}

impl TryFrom<Frame> for Buffer {
    type Error = Error;

    fn try_from(value: Frame) -> Result<Self, Self::Error> {
        let mut out = Vec::try_from(&value.payload)?;
        if value.frame_type().has_mic()
            && let Some(mic) = &value.mic
        {
            out.extend_from_slice(mic).unwrap();
        }

        Ok(out)
    }
}

#[cfg(feature = "std")]
impl TryFrom<&Frame> for std::vec::Vec<u8> {
    type Error = Error;

    fn try_from(value: &Frame) -> Result<Self, Self::Error> {
        let out: Buffer = value.try_into()?;
        Ok(out.into_iter().collect())
    }
}

#[cfg(feature = "std")]
impl TryFrom<Frame> for std::vec::Vec<u8> {
    type Error = Error;

    fn try_from(value: Frame) -> Result<Self, Self::Error> {
        let out: Buffer = value.try_into()?;
        Ok(out.into_iter().collect())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::vec::Vec;

    const TEST_KEY: [u8; 32] = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
        26, 27, 28, 29, 30, 31, 32,
    ];

    #[test]
    fn test_key_exchange_request() {
        let pl = Frame {
            payload: Payload::KeyExchangeRequest(key_exchange_request::KeyExchangeRequest {
                device_pub_key: TEST_KEY,
                nonce: 12345,
            }),
            mic: None,
        };

        let b: Vec<_> = (&pl).try_into().unwrap();
        assert_eq!(
            vec![
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
                23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 57, 48, 0, 0
            ],
            b
        );

        let pl_new = Frame::try_from(b.as_ref()).unwrap();
        assert_eq!(pl, pl_new);
    }

    #[test]
    fn test_key_exchange_response() {
        let mut pl = Frame {
            payload: Payload::KeyExchangeResponse(key_exchange_response::KeyExchangeResponse {
                server_pub_key: TEST_KEY,
                counter: 12345,
            }),
            mic: None,
        };
        pl.set_mic(&TEST_KEY).unwrap();

        let b: Vec<_> = (&pl).try_into().unwrap();
        assert_eq!(
            vec![
                16, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
                23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 57, 48, 0, 0, 67, 109, 38, 177
            ],
            b
        );

        let pl_new = Frame::try_from(b.as_ref()).unwrap();
        assert_eq!(pl, pl_new);
    }

    #[test]
    fn test_activation_request() {
        let mut pl = Frame {
            payload: Payload::ActivationRequest(activation_request::ActivationRequest {
                short_id: [1, 2, 3, 4],
                counter: 12345,
                profile_info: activation_request::ProfileInfo::Plain(
                    activation_request::ProfileInfoPlain {
                        vendor_id: [1, 2, 3, 4],
                        profile_id: [1, 2],
                        version_id: [1, 2],
                    },
                ),
            }),
            mic: None,
        };

        pl.encrypt(&TEST_KEY).unwrap();
        assert_eq!(
            Frame {
                payload: Payload::ActivationRequest(activation_request::ActivationRequest {
                    short_id: [1, 2, 3, 4],
                    counter: 12345,
                    profile_info: activation_request::ProfileInfo::Encrypted([
                        188, 95, 115, 168, 116, 168, 85, 88
                    ]),
                }),
                mic: None,
            },
            pl
        );

        pl.set_mic(&TEST_KEY).unwrap();
        assert_eq!(
            Frame {
                payload: Payload::ActivationRequest(activation_request::ActivationRequest {
                    short_id: [1, 2, 3, 4],
                    counter: 12345,
                    profile_info: activation_request::ProfileInfo::Encrypted([
                        188, 95, 115, 168, 116, 168, 85, 88
                    ]),
                }),
                mic: Some([117, 61, 242, 51]),
            },
            pl
        );

        let b: Vec<_> = (&pl).try_into().unwrap();
        assert_eq!(
            vec![
                32, 1, 2, 3, 4, 57, 48, 0, 0, 188, 95, 115, 168, 116, 168, 85, 88, 117, 61, 242, 51
            ],
            b
        );

        let pl_new = Frame::try_from(b.as_ref()).unwrap();
        assert_eq!(pl, pl_new);
    }

    #[test]
    fn test_activation_response() {
        let mut pl = Frame {
            payload: Payload::ActivationResponse(activation_response::ActivationResponse {
                counter: 12345,
                network_info: activation_response::NetworkInfo::Plain(
                    activation_response::NetworkInfoPlain {
                        region_id: 123,
                        channel_plan_id: 223,
                    },
                ),
            }),
            mic: None,
        };

        pl.encrypt(&TEST_KEY).unwrap();
        assert_eq!(
            Frame {
                payload: Payload::ActivationResponse(activation_response::ActivationResponse {
                    counter: 12345,
                    network_info: activation_response::NetworkInfo::Encrypted([198, 130]),
                }),
                mic: None,
            },
            pl
        );

        pl.set_mic(&TEST_KEY).unwrap();
        assert_eq!(
            Frame {
                payload: Payload::ActivationResponse(activation_response::ActivationResponse {
                    counter: 12345,
                    network_info: activation_response::NetworkInfo::Encrypted([198, 130]),
                }),
                mic: Some([241, 80, 153, 85]),
            },
            pl
        );

        let b: Vec<_> = (&pl).try_into().unwrap();
        assert_eq!(vec![48, 57, 48, 0, 0, 198, 130, 241, 80, 153, 85], b);

        let pl_new = Frame::try_from(b.as_ref()).unwrap();
        assert_eq!(pl, pl_new);
    }

    #[test]
    fn test_telemetry() {
        let mut pl = Frame {
            payload: Payload::Telemetry(telemetry::Telemetry {
                short_id: [1, 2, 3, 4],
                counter: 12345,
                payload: [1, 2, 3, 4, 5].try_into().unwrap(),
            }),
            mic: None,
        };

        pl.encrypt(&TEST_KEY).unwrap();
        assert_eq!(
            Frame {
                payload: Payload::Telemetry(telemetry::Telemetry {
                    short_id: [1, 2, 3, 4],
                    counter: 12345,
                    payload: [188, 95, 115, 168, 112].try_into().unwrap(),
                }),
                mic: None,
            },
            pl
        );

        pl.set_mic(&TEST_KEY).unwrap();
        assert_eq!(
            Frame {
                payload: Payload::Telemetry(telemetry::Telemetry {
                    short_id: [1, 2, 3, 4],
                    counter: 12345,
                    payload: [188, 95, 115, 168, 112].try_into().unwrap(),
                }),
                mic: Some([13, 177, 155, 132]),
            },
            pl
        );

        let b: Vec<_> = (&pl).try_into().unwrap();
        assert_eq!(
            vec![
                96, 1, 2, 3, 4, 57, 48, 0, 0, 188, 95, 115, 168, 112, 13, 177, 155, 132
            ],
            b
        );

        let pl_new = Frame::try_from(b.as_ref()).unwrap();
        assert_eq!(pl, pl_new);
    }

    #[test]
    fn test_state() {
        let mut pl = Frame {
            payload: Payload::State(state::State {
                short_id: [1, 2, 3, 4],
                is_downlink: true,
                counter: 12345,
                payload: [1, 2, 3, 4, 5].try_into().unwrap(),
            }),
            mic: None,
        };

        pl.encrypt(&TEST_KEY).unwrap();
        assert_eq!(
            Frame {
                payload: Payload::State(state::State {
                    short_id: [1, 2, 3, 4],
                    is_downlink: true,
                    counter: 12345,
                    payload: [188, 95, 115, 168, 112].try_into().unwrap(),
                }),
                mic: None,
            },
            pl
        );

        pl.set_mic(&TEST_KEY).unwrap();
        assert_eq!(
            Frame {
                payload: Payload::State(state::State {
                    short_id: [1, 2, 3, 4],
                    is_downlink: true,
                    counter: 12345,
                    payload: [188, 95, 115, 168, 112].try_into().unwrap(),
                }),
                mic: Some([188, 224, 52, 72]),
            },
            pl
        );

        let b: Vec<_> = (&pl).try_into().unwrap();
        assert_eq!(
            vec![
                113, 1, 2, 3, 4, 57, 48, 0, 0, 188, 95, 115, 168, 112, 188, 224, 52, 72
            ],
            b
        );

        let pl_new = Frame::try_from(b.as_ref()).unwrap();
        assert_eq!(pl, pl_new);
    }
}
