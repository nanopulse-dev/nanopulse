use chacha20::ChaCha20;
use chacha20::cipher::{KeyIvInit, StreamCipher};
use heapless::Vec;

use crate::errors::Error;
use crate::frames::{Buffer, FrameType, get_chacha20_iv};

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub is_downlink: bool,
    pub is_resp: bool,
    pub short_id: [u8; 4],
    pub counter: u32,
    pub payload: Payload,
}

impl State {
    pub fn encrypt(&mut self, enc_key: &[u8; 32]) -> Result<(), Error> {
        let mut buffer = Buffer::new();

        match &self.payload {
            Payload::Encrypted(_) => return Err(Error::AlreadyEncrypted),
            Payload::Request(v) => buffer.extend_from_slice(v).unwrap(),
            Payload::Response(v) => {
                buffer
                    .extend_from_slice(&v.request_counter.to_le_bytes())
                    .unwrap();
                buffer.extend_from_slice(&v.payload).unwrap();
            }
        }

        let iv = get_chacha20_iv(FrameType::State, self.is_downlink, self.counter);
        let mut cipher = ChaCha20::new(enc_key.into(), &iv.into());
        cipher.apply_keystream(&mut buffer);

        self.payload = Payload::Encrypted(buffer);

        Ok(())
    }

    pub fn decrypt(&mut self, enc_key: &[u8; 32]) -> Result<(), Error> {
        let mut buffer = if let Payload::Encrypted(v) = &self.payload {
            v.clone()
        } else {
            return Err(Error::MustBeEncryptedFirst);
        };

        let iv = get_chacha20_iv(FrameType::State, self.is_downlink, self.counter);
        let mut cipher = ChaCha20::new(enc_key.into(), &iv.into());
        cipher.apply_keystream(&mut buffer);

        if self.is_resp {
            if buffer.len() < 2 {
                return Err(Error::NotEnoughBytes);
            }

            self.payload = Payload::Response(ResponsePayload {
                request_counter: u16::from_le_bytes([buffer[0], buffer[1]]),
                payload: {
                    buffer.drain(0..2);
                    buffer
                },
            });
        } else {
            self.payload = Payload::Request(buffer);
        }

        Ok(())
    }
}

impl TryFrom<&[u8]> for State {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        // 13 - MIC = 9
        if value.len() < 9 {
            return Err(Error::NotEnoughBytes);
        }

        Ok(State {
            is_downlink: value[0] & 0x01 != 0,
            is_resp: value[0] & 0x02 != 0,
            short_id: value[1..5].try_into().unwrap(),
            counter: u32::from_le_bytes(value[5..9].try_into().unwrap()),
            payload: Payload::Encrypted(value[9..].try_into().unwrap()),
        })
    }
}

impl TryFrom<&State> for Buffer {
    type Error = Error;

    fn try_from(value: &State) -> Result<Self, Self::Error> {
        let mut out = Vec::new();
        out.push(FrameType::State.into()).unwrap();
        if value.is_downlink {
            out[0] |= 0x01;
        }
        if value.is_resp {
            out[0] |= 0x02;
        }
        out.extend_from_slice(&value.short_id).unwrap();
        out.extend_from_slice(&value.counter.to_le_bytes()).unwrap();

        if let Payload::Encrypted(v) = &value.payload {
            out.extend_from_slice(v).unwrap();
        } else {
            return Err(Error::MustBeEncryptedFirst);
        }

        Ok(out)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payload {
    Encrypted(Buffer),
    Request(Buffer),
    Response(ResponsePayload),
}

impl Default for Payload {
    fn default() -> Self {
        Payload::Encrypted(Buffer::new())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct ResponsePayload {
    pub request_counter: u16,
    pub payload: Buffer,
}
