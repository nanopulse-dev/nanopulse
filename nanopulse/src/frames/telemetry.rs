use chacha20::ChaCha20;
use chacha20::cipher::{KeyIvInit, StreamCipher};
use heapless::Vec;

use crate::errors::Error;
use crate::frames::{Buffer, FrameType, get_chacha20_iv};

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Telemetry {
    pub short_id: [u8; 4],
    pub counter: u32,
    pub payload: Buffer,
}

impl Telemetry {
    pub fn encrypt(&mut self, enc_key: &[u8; 32]) -> Result<(), Error> {
        let iv = get_chacha20_iv(FrameType::Telemetry, false, self.counter);
        let mut cipher = ChaCha20::new(enc_key.into(), &iv.into());
        cipher.apply_keystream(&mut self.payload);

        Ok(())
    }

    pub fn decrypt(&mut self, enc_key: &[u8; 32]) -> Result<(), Error> {
        self.encrypt(enc_key)
    }
}

impl TryFrom<&[u8]> for Telemetry {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        // 13 - MIC = 9
        if value.len() < 9 {
            return Err(Error::NotEnoughBytes);
        }

        Ok(Telemetry {
            short_id: value[1..5].try_into().unwrap(),
            counter: u32::from_le_bytes(value[5..9].try_into().unwrap()),
            payload: value[9..].try_into().unwrap(),
        })
    }
}

impl From<&Telemetry> for Buffer {
    fn from(value: &Telemetry) -> Self {
        let mut out = Vec::new();
        out.push(FrameType::Telemetry.into()).unwrap();
        out.extend_from_slice(&value.short_id).unwrap();
        out.extend_from_slice(&value.counter.to_le_bytes()).unwrap();
        out.extend_from_slice(&value.payload).unwrap();
        out
    }
}
