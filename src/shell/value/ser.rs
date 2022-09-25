use serde::{ser::SerializeSeq, Serialize};

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
            Value::Map(map) => map.serialize(serializer),
            Value::Table(table) => {
                let mut seq = serializer.serialize_seq(Some(table.len()))?;
                for row in table.iter() {
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
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;

    #[test]
    fn json_test() {
        assert_eq!(&serde_json::to_string(&Value::Int(69)).unwrap(), "69");
        assert!(serde_json::to_string(&Value::List(Rc::new(vec![
            Value::Int(1),
            Value::String(Rc::new(String::from("oofers")))
        ])))
        .is_ok());
    }
}
