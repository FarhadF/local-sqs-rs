use serde::de::{self, Visitor};
use serde::{Deserializer, Serializer};
use std::fmt::{self, Display};
use std::str::FromStr;

pub fn deserialize_number_from_string<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    struct StringOrIntVisitor<T>(std::marker::PhantomData<T>);

    impl<'de, T> Visitor<'de> for StringOrIntVisitor<T>
    where
        T: FromStr,
        T::Err: Display,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or an integer")
        }

        fn visit_str<E>(self, value: &str) -> Result<T, E>
        where
            E: de::Error,
        {
            value.parse::<T>().map_err(de::Error::custom)
        }

        fn visit_u64<E>(self, value: u64) -> Result<T, E>
        where
            E: de::Error,
        {
            value.to_string().parse::<T>().map_err(de::Error::custom)
        }

        fn visit_i64<E>(self, value: i64) -> Result<T, E>
        where
            E: de::Error,
        {
            value.to_string().parse::<T>().map_err(de::Error::custom)
        }
    }

    deserializer.deserialize_any(StringOrIntVisitor(std::marker::PhantomData))
}

pub fn serialize_number_to_string<S, T>(n: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Display,
{
    serializer.serialize_str(&n.to_string())
}
