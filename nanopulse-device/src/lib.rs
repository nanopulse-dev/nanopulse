#![allow(dead_code)]
#![cfg_attr(not(feature = "std"), no_std)]

pub use embedded_hal_bus;

#[cfg(feature = "lora_sx126x")]
pub use lora_phy;

pub mod context;
pub mod data_rate;
pub mod device;
pub mod errors;
pub mod keypair;
pub mod radio;
pub mod region;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    KeyExchange,
    Activate,
    Idle,
}
