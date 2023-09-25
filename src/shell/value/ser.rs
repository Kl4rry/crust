use indexmap::IndexMap;
use serde::{
    ser::{Error, SerializeMap, SerializeSeq},
    Serialize,
};

use super::Value;

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::Null => serializer.serialize_none(),
            Value::Int(int) => int.serialize(serializer),
            Value::Float(float) => float.serialize(serializer),
            Value::Bool(boolean) => boolean.serialize(serializer),
            Value::String(string) => string.serialize(serializer),
            Value::List(list) => list.serialize(serializer),
            Value::Map(map) => {
                let mut ser_map = serializer.serialize_map(Some(map.len()))?;
                for (k, v) in map.iter() {
                    ser_map.serialize_entry(&**k, v)?;
                }
                ser_map.end()
            }
            Value::Table(table) => {
                let mut seq = serializer.serialize_seq(Some(table.len()))?;
                for row in table.iter() {
                    // TODO optimize this
                    let row: IndexMap<&str, &Value> = row.iter().map(|(k, v)| (&**k, v)).collect();
                    seq.serialize_element(&row)?;
                }
                seq.end()
            }
            Value::Range(range) => {
                let mut seq = serializer.serialize_seq(None)?;
                let range = (**range).clone();
                for i in range {
                    seq.serialize_element(&i)?;
                }
                seq.end()
            }
            Value::Regex(regex) => regex.1.serialize(serializer),
            Value::Binary(binary) => binary.serialize(serializer),
            Value::Closure(..) => Err(Error::custom("closure cannot be serialized")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_test() {
        assert_eq!(&serde_json::to_string(&Value::Int(69)).unwrap(), "69");
        assert!(serde_json::to_string(&Value::from(vec![
            Value::Int(1),
            Value::from(String::from("oofers"))
        ]))
        .is_ok());
    }
}
