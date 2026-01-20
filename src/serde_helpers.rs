use serde::{self, Deserialize, Deserializer, Serializer};
use std::fmt::Display;
use std::str::FromStr;

pub fn deserialize_number_from_string<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse::<T>().map_err(serde::de::Error::custom)
}

pub fn serialize_number_to_string<S, T>(n: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Display,
{
    serializer.serialize_str(&n.to_string())
}
