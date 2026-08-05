use rand_core::RngCore;

use nanopulse::keypair;
use nanopulse::x25519_dalek::{PublicKey, StaticSecret};

pub fn new<R>(rng: &mut R) -> (PublicKey, StaticSecret)
where
    R: RngCore,
{
    let mut seed = [0u8; 32];
    rng.fill_bytes(&mut seed);
    keypair::new(seed)
}
