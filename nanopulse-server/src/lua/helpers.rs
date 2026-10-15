use mlua::{Error, Function, Result, Table};

pub fn encode_u16_le(t: &Table) -> Result<()> {
    t.set(
        "encode_u16_le",
        Function::wrap(|v: u16| {
            let v = v.to_le_bytes();
            Ok::<[u8; 2], Error>(v)
        }),
    )
}

pub fn decode_u16_le(t: &Table) -> Result<()> {
    t.set(
        "decode_u16_le",
        Function::wrap(|(b0, b1): (u8, u8)| {
            let v = u16::from_le_bytes([b0, b1]);
            Ok::<u16, Error>(v)
        }),
    )
}

pub fn encode_i16_le(t: &Table) -> Result<()> {
    t.set(
        "encode_i16_le",
        Function::wrap(|v: i16| {
            let v = v.to_le_bytes();
            Ok::<[u8; 2], Error>(v)
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

pub fn encode_u32_le(t: &Table) -> Result<()> {
    t.set(
        "encode_u32_le",
        Function::wrap(|v: u32| {
            let v = v.to_le_bytes();
            Ok::<[u8; 4], Error>(v)
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

pub fn encode_i32_le(t: &Table) -> Result<()> {
    t.set(
        "encode_i32_le",
        Function::wrap(|v: i32| {
            let v = v.to_le_bytes();
            Ok::<[u8; 4], Error>(v)
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

#[cfg(test)]
mod test {
    use super::*;
    use mlua::{Function, Lua, LuaSerdeExt, Value};

    #[test]
    fn test_helpers() {
        let lua = Lua::new();
        let t = lua.create_table().unwrap();

        encode_u16_le(&t).unwrap();
        encode_i16_le(&t).unwrap();
        encode_u32_le(&t).unwrap();
        encode_i32_le(&t).unwrap();
        decode_u16_le(&t).unwrap();
        decode_i16_le(&t).unwrap();
        decode_u32_le(&t).unwrap();
        decode_i32_le(&t).unwrap();

        let v: Value = t.get::<Function>("encode_u16_le").unwrap().call(1).unwrap();
        assert_eq!(vec![0x01, 0x00], lua.from_value::<Vec<u8>>(v).unwrap());

        let v: Value = t
            .get::<Function>("decode_u16_le")
            .unwrap()
            .call((0x01, 0x00))
            .unwrap();
        assert_eq!(1, lua.from_value::<u16>(v).unwrap());

        let v: Value = t
            .get::<Function>("encode_i16_le")
            .unwrap()
            .call(-1)
            .unwrap();
        assert_eq!(vec![0xff, 0xff], lua.from_value::<Vec<u8>>(v).unwrap());

        let v: Value = t
            .get::<Function>("decode_i16_le")
            .unwrap()
            .call((0xff, 0xff))
            .unwrap();
        assert_eq!(-1, lua.from_value::<i16>(v).unwrap());

        let v: Value = t.get::<Function>("encode_u32_le").unwrap().call(1).unwrap();
        assert_eq!(
            vec![0x01, 0x00, 0x00, 0x00],
            lua.from_value::<Vec<u8>>(v).unwrap()
        );

        let v: Value = t
            .get::<Function>("decode_u32_le")
            .unwrap()
            .call((0x01, 0x00, 0x00, 0x00))
            .unwrap();
        assert_eq!(1, lua.from_value::<u32>(v).unwrap());

        let v: Value = t
            .get::<Function>("encode_i32_le")
            .unwrap()
            .call(-1)
            .unwrap();
        assert_eq!(
            vec![0xff, 0xff, 0xff, 0xff],
            lua.from_value::<Vec<u8>>(v).unwrap()
        );

        let v: Value = t
            .get::<Function>("decode_i32_le")
            .unwrap()
            .call((0xff, 0xff, 0xff, 0xff))
            .unwrap();
        assert_eq!(-1, lua.from_value::<i32>(v).unwrap());
    }
}
