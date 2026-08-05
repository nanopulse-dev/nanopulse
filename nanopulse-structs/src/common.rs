use minicbor::{Decode, Encode};

#[derive(Default, Debug, Clone, Copy, Encode, Decode, PartialEq, Eq)]
#[cbor(index_only)]
pub enum CodingRate {
    #[default]
    #[n(0)]
    Cr45,
    #[n(1)]
    Cr46,
    #[n(2)]
    Cr47,
    #[n(3)]
    Cr48,
}

#[derive(Debug, Clone, Copy, Encode, Decode)]
#[cbor(index_only)]
pub enum Modulation {
    #[n(0)]
    Lora,
    #[n(1)]
    Fsk,
}
