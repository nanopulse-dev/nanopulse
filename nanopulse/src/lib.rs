#![cfg_attr(not(feature = "std"), no_std)]

pub mod errors;
pub mod frames;
pub mod keypair;
pub mod keys;

pub use x25519_dalek;

pub const BUFFER_SIZE: usize = 255;
