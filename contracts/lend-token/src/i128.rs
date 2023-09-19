use schemars::JsonSchema;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// Just a helper for (de)serializable i128 as string for json compatibility, not providing any
/// operations.
#[derive(
    Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, JsonSchema, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Int128(
    #[schemars(with = "String")]
    #[serde(
        serialize_with = "i128_serialize",
        deserialize_with = "i128_deserialize"
    )]
    i128,
);

impl Int128 {
    pub fn zero() -> Self {
        0.into()
    }
}

fn i128_serialize<S>(val: &i128, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    ser.serialize_str(&val.to_string())
}

fn i128_deserialize<'de, D>(de: D) -> Result<i128, D::Error>
where
    D: Deserializer<'de>,
{
    de.deserialize_str(Int128Visitor)
}

struct Int128Visitor;

impl<'de> Visitor<'de> for Int128Visitor {
    type Value = i128;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string-encoded integer")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v.parse::<i128>() {
            Ok(i) => Ok(i),
            Err(e) => Err(E::custom(format!("invalid Int128 '{}' - {}", v, e))),
        }
    }
}

impl From<i128> for Int128 {
    fn from(val: i128) -> Self {
        Self(val)
    }
}

impl From<Int128> for i128 {
    fn from(val: Int128) -> Self {
        val.0
    }
}
