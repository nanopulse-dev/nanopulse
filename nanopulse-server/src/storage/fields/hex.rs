use std::fmt;
use std::str::FromStr;

use anyhow::anyhow;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::sqlite::SqliteTypeInfo;
use sqlx::{Decode, Encode, Sqlite, Type};
use utoipa::{PartialSchema, ToSchema};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct HexArray<const SIZE: usize>([u8; SIZE]);

impl<const SIZE: usize> HexArray<SIZE> {
    pub fn is_empty(&self) -> bool {
        self.0 == [0u8; SIZE]
    }

    pub fn as_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl<const SIZE: usize> Default for HexArray<SIZE> {
    fn default() -> Self {
        HexArray([0u8; SIZE])
    }
}

impl<const SIZE: usize> fmt::Debug for HexArray<SIZE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl<const SIZE: usize> fmt::Display for HexArray<SIZE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl<const SIZE: usize> AsRef<[u8; SIZE]> for HexArray<SIZE> {
    fn as_ref(&self) -> &[u8; SIZE] {
        &self.0
    }
}

impl<const SIZE: usize> AsRef<[u8]> for HexArray<SIZE> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl<const SIZE: usize> FromStr for HexArray<SIZE> {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let b = hex::decode(s).map_err(|e| anyhow!("{}", e))?;
        Ok(HexArray(b.try_into().map_err(|_| anyhow!("invalid size"))?))
    }
}

impl<const SIZE: usize> From<HexArray<SIZE>> for [u8; SIZE] {
    fn from(value: HexArray<SIZE>) -> Self {
        value.0
    }
}

impl<const SIZE: usize> From<[u8; SIZE]> for HexArray<SIZE> {
    fn from(value: [u8; SIZE]) -> Self {
        HexArray(value)
    }
}

impl<const SIZE: usize> From<Vec<u8>> for HexArray<SIZE> {
    fn from(mut value: Vec<u8>) -> Self {
        value.resize(SIZE, 0);
        let mut b = [0u8; SIZE];
        b.copy_from_slice(&value);
        HexArray(b)
    }
}

impl<const SIZE: usize> TryFrom<&[u8]> for HexArray<SIZE> {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut value = value.to_vec();
        value.resize(SIZE, 0);
        let mut b = [0u8; SIZE];
        b.copy_from_slice(&value);

        Ok(HexArray(b))
    }
}

impl<const SIZE: usize> ToSchema for HexArray<SIZE> {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("HEX string")
    }
}

impl<const SIZE: usize> PartialSchema for HexArray<SIZE> {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::RefOr::T(utoipa::openapi::schema::Schema::Object(
            utoipa::openapi::ObjectBuilder::new()
                .schema_type(utoipa::openapi::schema::Type::String)
                .build(),
        ))
    }
}

impl<const SIZE: usize> Type<Sqlite> for HexArray<SIZE> {
    fn type_info() -> SqliteTypeInfo {
        <Vec<u8> as Type<Sqlite>>::type_info()
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <Vec<u8> as Type<Sqlite>>::compatible(ty)
    }
}

impl<const SIZE: usize> Encode<'_, Sqlite> for HexArray<SIZE> {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer,
    ) -> Result<IsNull, BoxDynError> {
        <&[u8] as Encode<Sqlite>>::encode_by_ref(&self.0.as_slice(), buf)
    }
}

impl<const SIZE: usize> Decode<'_, Sqlite> for HexArray<SIZE> {
    fn decode(value: <Sqlite as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        let b = <&[u8] as Decode<Sqlite>>::decode(value)?;
        Self::try_from(b).map_err(|e| e.into_boxed_dyn_error())
    }
}

impl<'de, const SIZE: usize> Deserialize<'de> for HexArray<SIZE> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(de::Error::custom)
    }
}

impl<const SIZE: usize> Serialize for HexArray<SIZE> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
