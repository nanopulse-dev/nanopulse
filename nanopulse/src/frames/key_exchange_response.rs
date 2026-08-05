use heapless::Vec;

use super::{Buffer, FrameType};
use crate::errors::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyExchangeResponse {
    pub server_pub_key: [u8; 32],
    pub counter: u32,
}

impl TryFrom<&[u8]> for KeyExchangeResponse {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 37 {
            return Err(Error::NotEnoughBytes);
        }

        Ok(KeyExchangeResponse {
            server_pub_key: value[1..33].try_into().unwrap(),
            counter: u32::from_le_bytes(value[33..37].try_into().unwrap()),
        })
    }
}

impl From<&KeyExchangeResponse> for Buffer {
    fn from(value: &KeyExchangeResponse) -> Self {
        let mut out = Vec::new();
        out.push(FrameType::KeyExchangeResponse.into()).unwrap();
        out.extend_from_slice(&value.server_pub_key).unwrap();
        out.extend_from_slice(&value.counter.to_le_bytes()).unwrap();

        out
    }
}

impl From<&KeyExchangeResponse> for [u8; 37] {
    fn from(value: &KeyExchangeResponse) -> Self {
        let mut out = [0u8; 37];
        out[0] = FrameType::KeyExchangeResponse.into();
        out[1..33].copy_from_slice(&value.server_pub_key);
        out[33..37].copy_from_slice(&value.counter.to_le_bytes());
        out
    }
}
