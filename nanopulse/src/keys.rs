use hkdf::Hkdf;
use sha2::Sha256;
use x25519_dalek::{PublicKey, StaticSecret};

use crate::frames::FrameType;

pub enum Error {
    InvalidSharedKey,
}

enum KeyUsage {
    Root,
    Mic,
    Encryption,
}

impl From<KeyUsage> for u8 {
    fn from(val: KeyUsage) -> Self {
        match val {
            KeyUsage::Root => 0x00,
            KeyUsage::Mic => 0x01,
            KeyUsage::Encryption => 0x02,
        }
    }
}

/// Generate shared_secret using our secret and their public key.
pub fn get_shared_secret(secret: &StaticSecret, their_public_key: &PublicKey) -> [u8; 32] {
    secret.diffie_hellman(their_public_key).to_bytes()
}

/// Generate root_key based on shared_secret, device PIN and Key Exchange nonces.
pub fn get_root_key(
    shared_secret: &[u8; 32],
    pin: &[u8; 4],
    key_exchange_req_nonce: u32,
    key_exchange_resp_nonce: u32,
) -> [u8; 32] {
    let mut ikm = [0u8; 36];
    ikm[0..32].copy_from_slice(shared_secret);
    ikm[32..36].copy_from_slice(pin);

    let mut salt = [0u8; 8];
    salt[0..4].copy_from_slice(&key_exchange_req_nonce.to_le_bytes());
    salt[4..8].copy_from_slice(&key_exchange_resp_nonce.to_le_bytes());

    let (root_key, _) = Hkdf::<Sha256>::extract(Some(&salt), &ikm);
    root_key.into()
}

/// Generate session_root_key based on root_key and Activation nonces.
pub fn get_session_root_key(
    root_key: &[u8; 32],
    activation_req_counter: u32,
    activation_resp_counter: u32,
) -> [u8; 32] {
    let mut info = [0u8; 9];
    info[0] = KeyUsage::Root.into();
    info[1..5].copy_from_slice(&activation_req_counter.to_le_bytes());
    info[5..9].copy_from_slice(&activation_resp_counter.to_le_bytes());

    expand_key(root_key, &info)
}

/// This function generates the encryption key for the given parameters.
///
/// key must be set to the session_root_key, except for the Activation Request, in which case no
/// session_root_key has been established yet and the root_key must be used.
pub fn get_encryption_key(key: &[u8; 32], frame_type: FrameType, is_downlink: bool) -> [u8; 32] {
    let mut info = [0u8; 3];
    info[0] = KeyUsage::Encryption.into();
    info[1] = frame_type.into();
    if is_downlink {
        info[2] = 0x01;
    }
    expand_key(key, &info)
}

/// This function generates the MIC key fr the given parameters.
///
/// key must be set to the session_root_key, except for the Activation Request, in which case no
/// session_root_key has been established yet and the root_key must be used.
pub fn get_mic_key(
    key: &[u8; 32],
    frame_type: FrameType,
    tx_channel: u8,
    tx_datarate: u8,
) -> [u8; 32] {
    let mut info = [0u8; 4];
    info[0] = KeyUsage::Mic.into();
    info[1] = frame_type.into();
    info[2] = tx_channel;
    info[3] = tx_datarate;

    expand_key(key, &info)
}

fn expand_key(prk: &[u8], info: &[u8]) -> [u8; 32] {
    let mut okm = [0u8; 32];
    let hk = Hkdf::<Sha256>::from_prk(prk).unwrap();
    hk.expand(info, &mut okm).unwrap();
    okm
}

#[cfg(test)]
mod test {
    use super::*;

    const TEST_KEY: [u8; 32] = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
        26, 27, 28, 29, 30, 31, 32,
    ];

    #[test]
    fn test_get_root_key() {
        let out = get_root_key(&TEST_KEY, &[1, 2, 3, 4], 1, 2);
        assert_eq!(
            [
                91, 62, 84, 152, 70, 217, 161, 254, 245, 254, 4, 87, 65, 16, 75, 169, 33, 34, 43,
                249, 2, 30, 209, 99, 225, 80, 210, 87, 171, 193, 72, 46
            ],
            out
        );
    }

    #[test]
    fn test_get_session_root_key() {
        let out = get_session_root_key(&TEST_KEY, 1, 2);
        assert_eq!(
            [
                223, 32, 154, 167, 203, 181, 116, 116, 21, 234, 235, 237, 130, 200, 222, 43, 50,
                214, 189, 224, 229, 139, 250, 151, 33, 17, 212, 10, 78, 224, 93, 153
            ],
            out
        );
    }

    #[test]
    fn test_get_encryption_key() {
        let out = get_encryption_key(&TEST_KEY, FrameType::Telemetry, false);
        assert_eq!(
            [
                129, 10, 48, 12, 147, 102, 218, 146, 19, 189, 67, 219, 68, 152, 128, 98, 26, 125,
                243, 137, 90, 250, 142, 8, 12, 69, 47, 98, 71, 37, 115, 236
            ],
            out
        );

        assert_ne!(
            out,
            get_encryption_key(&TEST_KEY, FrameType::Telemetry, true)
        );
        assert_ne!(out, get_encryption_key(&TEST_KEY, FrameType::State, false));
    }

    #[test]
    fn test_get_mic_key() {
        let out = get_mic_key(&TEST_KEY, FrameType::Telemetry, 1, 2);
        assert_eq!(
            [
                58, 22, 231, 179, 120, 210, 217, 26, 168, 56, 122, 232, 129, 194, 181, 134, 95,
                231, 12, 145, 8, 79, 21, 79, 68, 247, 29, 111, 175, 53, 164, 83
            ],
            out
        );

        assert_ne!(out, get_mic_key(&TEST_KEY, FrameType::Telemetry, 2, 2));
        assert_ne!(out, get_mic_key(&TEST_KEY, FrameType::Telemetry, 1, 3));
        assert_ne!(out, get_mic_key(&TEST_KEY, FrameType::State, 1, 2));
    }
}
