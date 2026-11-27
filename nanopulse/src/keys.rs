use hkdf::Hkdf;
use sha2::Sha256;
use x25519_dalek::{PublicKey, StaticSecret};

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

/// Get key for encryption use.
///
/// key must be set to the session_root_key, except for the Activation Request, in which case no
/// session_root_key has been established yet and the root_key must be used.
pub fn get_encryption_key(key: &[u8; 32]) -> [u8; 32] {
    expand_key(key, &[KeyUsage::Encryption.into()])
}

/// Get key for mic calculation use.
///
/// key must be set to the session_root_key, except for the Activation Request, in which case no
/// session_root_key has been established yet and the root_key must be used.
pub fn get_mic_key(key: &[u8; 32]) -> [u8; 32] {
    expand_key(key, &[KeyUsage::Mic.into()])
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
        let out = get_encryption_key(&TEST_KEY);
        assert_eq!(
            [
                129, 6, 182, 84, 31, 127, 16, 73, 40, 169, 129, 139, 30, 58, 93, 251, 24, 202, 187,
                223, 52, 237, 81, 68, 171, 49, 69, 144, 143, 148, 154, 62
            ],
            out
        );
    }

    #[test]
    fn test_get_mic_key() {
        let out = get_mic_key(&TEST_KEY);
        assert_eq!(
            [
                167, 81, 153, 67, 37, 57, 146, 66, 144, 42, 245, 24, 65, 152, 254, 187, 116, 19,
                33, 28, 187, 86, 232, 170, 64, 105, 235, 6, 63, 242, 151, 64
            ],
            out
        );
    }
}
