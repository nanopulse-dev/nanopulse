use chacha20::ChaCha20;
use chacha20::cipher::{KeyIvInit, StreamCipher};
use heapless::Vec;

use super::{Buffer, FrameType};
use crate::errors::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivationResponse {
    pub counter: u32,
    pub network_info: NetworkInfo,
}

impl ActivationResponse {
    pub fn encrypt(&mut self, enc_key: &[u8; 32]) -> Result<(), Error> {
        let mut buffer = if let NetworkInfo::Plain(v) = &self.network_info {
            [v.region_id, v.channel_plan_id]
        } else {
            return Err(Error::AlreadyEncrypted);
        };

        let mut iv = [0u8; 12];
        iv[8..12].copy_from_slice(&self.counter.to_le_bytes());

        let mut cipher = ChaCha20::new(enc_key.into(), &iv.into());
        cipher.apply_keystream(&mut buffer);

        self.network_info = NetworkInfo::Encrypted(buffer);

        Ok(())
    }

    pub fn decrypt(&mut self, enc_key: &[u8; 32]) -> Result<(), Error> {
        let mut buffer = if let NetworkInfo::Encrypted(v) = &self.network_info {
            *v
        } else {
            return Err(Error::MustBeEncryptedFirst);
        };

        let mut iv = [0u8; 12];
        iv[8..12].copy_from_slice(&self.counter.to_le_bytes());

        let mut cipher = ChaCha20::new(enc_key.into(), &iv.into());
        cipher.apply_keystream(&mut buffer);

        self.network_info = NetworkInfo::Plain(NetworkInfoPlain {
            region_id: buffer[0],
            channel_plan_id: buffer[1],
        });

        Ok(())
    }
}

impl TryFrom<&[u8]> for ActivationResponse {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 7 {
            return Err(Error::NotEnoughBytes);
        }

        Ok(ActivationResponse {
            counter: u32::from_le_bytes(value[1..5].try_into().unwrap()),
            network_info: NetworkInfo::Encrypted(value[5..7].try_into().unwrap()),
        })
    }
}

impl TryFrom<&ActivationResponse> for Buffer {
    type Error = Error;

    fn try_from(value: &ActivationResponse) -> Result<Self, Self::Error> {
        let mut out = Vec::new();
        out.push(FrameType::ActivationResponse.into()).unwrap();
        out.extend_from_slice(&value.counter.to_le_bytes()).unwrap();

        if let NetworkInfo::Encrypted(v) = &value.network_info {
            out.extend_from_slice(v).unwrap();
        } else {
            return Err(Error::MustBeEncryptedFirst);
        }

        Ok(out)
    }
}

impl TryFrom<&ActivationResponse> for [u8; 7] {
    type Error = Error;

    fn try_from(value: &ActivationResponse) -> Result<Self, Self::Error> {
        let mut out = [0u8; 7];
        out[0] = FrameType::ActivationResponse.into();
        out[1..5].copy_from_slice(&value.counter.to_le_bytes());
        if let NetworkInfo::Encrypted(v) = &value.network_info {
            out[5..7].copy_from_slice(v);
        } else {
            return Err(Error::MustBeEncryptedFirst);
        }

        Ok(out)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkInfo {
    Encrypted([u8; 2]),
    Plain(NetworkInfoPlain),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkInfoPlain {
    pub region_id: u8,
    pub channel_plan_id: u8,
}
