use heapless::Vec;

use super::{Buffer, FrameType};
use crate::errors::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyExchangeRequest {
    pub device_pub_key: [u8; 32],
    pub nonce: u32,
}

impl TryFrom<&[u8]> for KeyExchangeRequest {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 37 {
            return Err(Error::NotEnoughBytes);
        }

        Ok(KeyExchangeRequest {
            device_pub_key: value[1..33].try_into().unwrap(),
            nonce: u32::from_le_bytes(value[33..37].try_into().unwrap()),
        })
    }
}

impl From<&KeyExchangeRequest> for Buffer {
    fn from(value: &KeyExchangeRequest) -> Self {
        let mut out = Vec::new();

        out.push(FrameType::KeyExchangeRequest.into()).unwrap();
        out.extend_from_slice(&value.device_pub_key).unwrap();
        out.extend_from_slice(&value.nonce.to_le_bytes()).unwrap();
        out
    }
}
