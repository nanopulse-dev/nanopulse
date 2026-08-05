#[cfg(feature = "std")]
use thiserror::Error;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "std", derive(Error, Debug))]
pub enum Error {
    #[cfg_attr(feature = "std", error("Not enough bytes"))]
    NotEnoughBytes,
    #[cfg_attr(feature = "std", error("Invalid frame type"))]
    InvalidFrameType,
    #[cfg_attr(feature = "std", error("Already encrypted"))]
    AlreadyEncrypted,
    #[cfg_attr(feature = "std", error("Must be encrypted first"))]
    MustBeEncryptedFirst,
    #[cfg_attr(feature = "std", error("Invalid payload"))]
    InvalidPayload,
    #[cfg_attr(feature = "std", error("heapless::Vec error"))]
    Heapless,
}
