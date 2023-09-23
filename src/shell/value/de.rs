use std::fmt;

use indexmap::IndexMap;
use serde::{
    de::{MapAccess, SeqAccess, Visitor},
    Deserialize,
};

use super::Value;

impl<'de> Deserialize<'de> for Value {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("data that can be represented as a value")
            }

            #[inline]
            fn visit_bool<E>(self, value: bool) -> Result<Value, E> {
                Ok(Value::Bool(value))
            }

            #[inline]
            fn visit_f64<E>(self, value: f64) -> Result<Value, E> {
                Ok(Value::Float(value))
            }

            #[inline]
            fn visit_f32<E>(self, value: f32) -> Result<Value, E> {
                Ok(Value::Float(value.into()))
            }

            #[inline]
            fn visit_i128<E>(self, value: i128) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Int(
                    value.try_into().map_err(serde::de::Error::custom)?,
                ))
            }

            #[inline]
            fn visit_i64<E>(self, value: i64) -> Result<Value, E> {
                Ok(Value::Int(value))
            }

            #[inline]
            fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E> {
                Ok(Value::Int(value.into()))
            }

            #[inline]
            fn visit_i16<E>(self, value: i16) -> Result<Self::Value, E> {
                Ok(Value::Int(value.into()))
            }

            #[inline]
            fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E> {
                Ok(Value::Int(value.into()))
            }

            #[inline]
            fn visit_u128<E>(self, value: u128) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Int(
                    value.try_into().map_err(serde::de::Error::custom)?,
                ))
            }

            #[inline]
            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Int(
                    value.try_into().map_err(serde::de::Error::custom)?,
                ))
            }

            #[inline]
            fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E> {
                Ok(Value::Int(value.into()))
            }

            #[inline]
            fn visit_u16<E>(self, value: u16) -> Result<Self::Value, E> {
                Ok(Value::Int(value.into()))
            }

            #[inline]
            fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E> {
                Ok(Value::Int(value.into()))
            }

            #[inline]
            fn visit_char<E>(self, value: char) -> Result<Self::Value, E> {
                Ok(Value::from(value.to_string()))
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_string(String::from(value))
            }

            #[inline]
            fn visit_string<E>(self, value: String) -> Result<Value, E> {
                Ok(Value::from(value))
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Value, E> {
                Ok(Value::Null)
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Value, E> {
                Ok(Value::Null)
            }

            #[inline]
            fn visit_seq<V>(self, mut visitor: V) -> Result<Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut vec: Vec<Value> = Vec::new();
                while let Some(elem) = visitor.next_element()? {
                    vec.push(elem);
                }

                Ok(Value::from(vec))
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::from(v.to_vec()))
            }

            #[inline]
            fn visit_map<V>(self, mut visitor: V) -> Result<Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut values = IndexMap::new();
                while let Some((key, value)) = visitor.next_entry::<String, _>()? {
                    values.insert(key.into(), value);
                }

                Ok(Value::from(values))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_test() {
        let json = r#"
            {
                "abc": "oof",
                "123": [1, 2, 3]
            }
        "#;
        let _: Value = serde_json::from_str(json).unwrap();

        let json = "1";
        let _: Value = serde_json::from_str(json).unwrap();
    }
}
