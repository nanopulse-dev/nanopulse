#[derive(Debug, defmt::Format)]
pub enum Error {
    HeaplessVec,
    UnsupportedModulation,
    RadioError,
    FrameError,
    CryptoError,
    CounterError,
    NotActivated,
    InvalidPayload,
}
