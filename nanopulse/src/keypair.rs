use sha2::{Digest, Sha256};
use x25519_dalek::{PublicKey, StaticSecret};

pub fn new(seed: [u8; 32]) -> (PublicKey, StaticSecret) {
    let secret = StaticSecret::from(seed);
    let pk = PublicKey::from(&secret);
    (pk, secret)
}

#[cfg(feature = "std")]
pub fn get_seed() -> [u8; 32] {
    let mut seed = [0u8; 32];
    getrandom::fill(&mut seed).unwrap();
    seed
}

#[cfg(feature = "std")]
pub fn get_rand_u32() -> u32 {
    getrandom::u32().unwrap()
}

pub fn get_short_id(public_key: &[u8; 32]) -> [u8; 4] {
    let mut hasher = Sha256::new();
    hasher.update(public_key);
    hasher.finalize()[0..4].try_into().unwrap()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_keypair() {
        let b = [0u8; 32];
        let (pub_key, _secret) = new(b);
        assert_eq!(
            [
                47, 229, 125, 163, 71, 205, 98, 67, 21, 40, 218, 172, 95, 187, 41, 7, 48, 255, 246,
                132, 175, 196, 207, 194, 237, 144, 153, 95, 88, 203, 59, 116
            ],
            pub_key.to_bytes()
        );

        assert_eq!([35, 60, 237, 119], get_short_id(&pub_key.to_bytes()));
    }
}
