use core::convert::{From, TryInto};

use rand_core::RngCore;

use nanopulse::keypair;

#[cfg(feature = "nor_flash")]
mod nor_flash;

#[cfg(feature = "nor_flash")]
pub use nor_flash::*;

unsafe extern "C" {
    static __root_security_start: u32;
    static __root_security_end: u32;
    static __counters_start: u32;
    static __counters_end: u32;
}

#[derive(Default)]
pub struct Session {
    pub session_root_key: [u8; 32],
    pub telemetry_up: u32,
    pub state_up: u32,
    pub state_down: u32,
}

impl Session {
    pub fn is_activated(&self) -> bool {
        defmt::info!("check has session_root_key: {:02x}", self.session_root_key);
        self.session_root_key != [0u8; 32]
    }
}

pub struct RootSecurity {
    pub public_key: [u8; 32],
    pub secret_key: [u8; 32],
    pub short_id: [u8; 4],
    pub pin: [u8; 4],
    pub server_public_key: [u8; 32],
    pub root_key: [u8; 32],
}

impl RootSecurity {
    pub const LENGTH: usize = 136;

    pub fn is_empty(&self) -> bool {
        self.public_key == [255u8; 32]
    }

    pub fn has_server_public_key(&self) -> bool {
        defmt::info!(
            "check has_server_public_key: {:02x}",
            self.server_public_key
        );
        self.server_public_key != [0u8; 32]
    }

    pub fn new<R>(rng: &mut R) -> Self
    where
        R: RngCore,
    {
        let mut seed = [0u8; 32];
        rng.fill_bytes(&mut seed);
        let (pub_key, secret) = keypair::new(seed);

        RootSecurity {
            short_id: keypair::get_short_id(&pub_key.to_bytes()),
            public_key: pub_key.to_bytes(),
            secret_key: secret.to_bytes(),
            pin: rng.next_u32().to_le_bytes(),
            server_public_key: [0u8; 32],
            root_key: [0u8; 32],
        }
    }
}

impl From<&RootSecurity> for [u8; RootSecurity::LENGTH] {
    fn from(value: &RootSecurity) -> Self {
        let mut out = [0u8; RootSecurity::LENGTH];
        out[0..32].copy_from_slice(&value.public_key);
        out[32..64].copy_from_slice(&value.secret_key);
        out[64..68].copy_from_slice(&value.short_id);
        out[68..72].copy_from_slice(&value.pin);
        out[72..104].copy_from_slice(&value.server_public_key);
        out[104..136].copy_from_slice(&value.root_key);

        out
    }
}

impl From<&[u8; RootSecurity::LENGTH]> for RootSecurity {
    fn from(value: &[u8; RootSecurity::LENGTH]) -> Self {
        RootSecurity {
            public_key: value[0..32].try_into().unwrap(),
            secret_key: value[32..64].try_into().unwrap(),
            short_id: value[64..68].try_into().unwrap(),
            pin: value[68..72].try_into().unwrap(),
            server_public_key: value[72..104].try_into().unwrap(),
            root_key: value[104..136].try_into().unwrap(),
        }
    }
}

#[derive(Default)]
pub struct Counters {
    pub activation_request_counter: u32,
    pub activation_response_counter: u32,
    pub last_key_exchange_request_nonce: u32,
}

impl Counters {
    pub const LENGTH: usize = 12;
}

impl From<&Counters> for [u8; Counters::LENGTH] {
    fn from(value: &Counters) -> Self {
        let mut out = [0u8; Counters::LENGTH];
        out[0..4].copy_from_slice(&value.activation_request_counter.to_le_bytes());
        out[4..8].copy_from_slice(&value.activation_response_counter.to_le_bytes());
        out[8..12].copy_from_slice(&value.last_key_exchange_request_nonce.to_le_bytes());
        out
    }
}

impl From<&[u8; Counters::LENGTH]> for Counters {
    fn from(value: &[u8; Counters::LENGTH]) -> Self {
        Counters {
            activation_request_counter: u32::from_le_bytes(value[0..4].try_into().unwrap()),
            activation_response_counter: u32::from_le_bytes(value[4..8].try_into().unwrap()),
            last_key_exchange_request_nonce: u32::from_le_bytes(value[8..12].try_into().unwrap()),
        }
    }
}

pub fn get_root_security_start_end() -> (u32, u32) {
    let start = core::ptr::addr_of!(__root_security_start) as u32;
    let end = core::ptr::addr_of!(__root_security_end) as u32;
    (start, end)
}

pub fn get_counters_start_end() -> (u32, u32) {
    let start = core::ptr::addr_of!(__counters_start) as u32;
    let end = core::ptr::addr_of!(__counters_end) as u32;
    (start, end)
}
