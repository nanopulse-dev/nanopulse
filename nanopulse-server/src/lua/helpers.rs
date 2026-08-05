use mlua::{Error, Function, Result, Table};

pub fn decode_u16_le(t: &Table) -> Result<()> {
    t.set(
        "decode_u16_le",
        Function::wrap(|(b0, b1): (u8, u8)| {
            let v = u16::from_le_bytes([b0, b1]);
            Ok::<u16, Error>(v)
        }),
    )
}

pub fn decode_i16_le(t: &Table) -> Result<()> {
    t.set(
        "decode_i16_le",
        Function::wrap(|(b0, b1): (u8, u8)| {
            let v = i16::from_le_bytes([b0, b1]);
            Ok::<i16, Error>(v)
        }),
    )
}

pub fn decode_u32_le(t: &Table) -> Result<()> {
    t.set(
        "decode_u32_le",
        Function::wrap(|(b0, b1, b2, b3): (u8, u8, u8, u8)| {
            let v = u32::from_le_bytes([b0, b1, b2, b3]);
            Ok::<u32, Error>(v)
        }),
    )
}

pub fn decode_i32_le(t: &Table) -> Result<()> {
    t.set(
        "decode_i32_le",
        Function::wrap(|(b0, b1, b2, b3): (u8, u8, u8, u8)| {
            let v = i32::from_le_bytes([b0, b1, b2, b3]);
            Ok::<i32, Error>(v)
        }),
    )
}
