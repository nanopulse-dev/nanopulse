use chacha20::ChaCha20;
use chacha20::cipher::{KeyIvInit, StreamCipher};
use heapless::Vec;

use super::{Buffer, FrameType};
use crate::errors::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivationRequest {
    pub short_id: [u8; 4],
    pub counter: u32,
    pub profile_info: ProfileInfo,
}

impl ActivationRequest {
    pub fn encrypt(&mut self, enc_key: &[u8; 32]) -> Result<(), Error> {
        let mut buffer = if let ProfileInfo::Plain(v) = &self.profile_info {
            let mut buffer = [0u8; 8];
            buffer[0..4].copy_from_slice(&v.vendor_id);
            buffer[4..6].copy_from_slice(&v.profile_id);
            buffer[6..8].copy_from_slice(&v.version_id);
            buffer
        } else {
            return Err(Error::AlreadyEncrypted);
        };

        let mut iv = [0u8; 12];
        iv[8..12].copy_from_slice(&self.counter.to_le_bytes());

        let mut cipher = ChaCha20::new(enc_key.into(), &iv.into());
        cipher.apply_keystream(&mut buffer);

        self.profile_info = ProfileInfo::Encrypted(buffer);

        Ok(())
    }

    pub fn decrypt(&mut self, enc_key: &[u8; 32]) -> Result<(), Error> {
        let mut buffer = if let ProfileInfo::Encrypted(v) = &self.profile_info {
            *v
        } else {
            return Err(Error::MustBeEncryptedFirst);
        };

        let mut iv = [0u8; 12];
        iv[8..12].copy_from_slice(&self.counter.to_le_bytes());

        let mut cipher = ChaCha20::new(enc_key.into(), &iv.into());
        cipher.apply_keystream(&mut buffer);

        self.profile_info = ProfileInfo::Plain(ProfileInfoPlain {
            vendor_id: buffer[0..4].try_into().unwrap(),
            profile_id: buffer[4..6].try_into().unwrap(),
            version_id: buffer[6..8].try_into().unwrap(),
        });

        Ok(())
    }
}

impl TryFrom<&[u8]> for ActivationRequest {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 17 {
            return Err(Error::NotEnoughBytes);
        }

        Ok(ActivationRequest {
            short_id: value[1..5].try_into().unwrap(),
            counter: u32::from_le_bytes(value[5..9].try_into().unwrap()),
            profile_info: ProfileInfo::Encrypted(value[9..17].try_into().unwrap()),
        })
    }
}

impl TryFrom<&ActivationRequest> for Buffer {
    type Error = Error;

    fn try_from(value: &ActivationRequest) -> Result<Self, Self::Error> {
        let mut out = Vec::new();
        out.push(FrameType::ActivationRequest.into()).unwrap();
        out.extend_from_slice(&value.short_id).unwrap();
        out.extend_from_slice(&value.counter.to_le_bytes()).unwrap();

        if let ProfileInfo::Encrypted(v) = &value.profile_info {
            out.extend_from_slice(v).unwrap();
        } else {
            return Err(Error::MustBeEncryptedFirst);
        }

        Ok(out)
    }
}

impl TryFrom<&ActivationRequest> for [u8; 17] {
    type Error = Error;
    fn try_from(value: &ActivationRequest) -> Result<Self, Self::Error> {
        let mut out = [0u8; 17];
        out[0] = FrameType::ActivationRequest.into();
        out[1..5].copy_from_slice(&value.short_id);
        out[5..9].copy_from_slice(&value.counter.to_le_bytes());

        if let ProfileInfo::Encrypted(v) = &value.profile_info {
            out[9..17].copy_from_slice(v);
        } else {
            return Err(Error::MustBeEncryptedFirst);
        }

        Ok(out)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileInfo {
    Encrypted([u8; 8]),
    Plain(ProfileInfoPlain),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileInfoPlain {
    pub vendor_id: [u8; 4],
    pub profile_id: [u8; 2],
    pub version_id: [u8; 2],
}
