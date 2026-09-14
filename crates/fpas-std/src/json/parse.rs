//! JSON deserialization that rejects duplicate decoded object member names.
//!
//! **Documentation:** `docs/pascal/std/text/json.md`.

use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value};
use std::fmt;

/// Parses JSON while retaining duplicate-name errors before constructing maps.
pub(super) fn parse(text: &str) -> Result<Value, serde_json::Error> {
    serde_json::from_str::<UniqueValue>(text).map(|value| value.0)
}

struct UniqueValue(Value);

impl<'de> Deserialize<'de> for UniqueValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(UniqueVisitor)
    }
}

struct UniqueVisitor;

impl<'de> Visitor<'de> for UniqueVisitor {
    type Value = UniqueValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value with unique object member names")
    }

    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Null))
    }

    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Bool(value)))
    }

    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Number(value.into())))
    }

    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Number(value.into())))
    }

    fn visit_f64<E: de::Error>(self, value: f64) -> Result<Self::Value, E> {
        Number::from_f64(value)
            .map(|number| UniqueValue(Value::Number(number)))
            .ok_or_else(|| E::custom("JSON number is not finite"))
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        self.visit_string(value.to_owned())
    }

    fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::String(value)))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Self::Value, A::Error> {
        let mut items = Vec::new();
        while let Some(value) = sequence.next_element::<UniqueValue>()? {
            items.push(value.0);
        }
        Ok(UniqueValue(Value::Array(items)))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut object: A) -> Result<Self::Value, A::Error> {
        let mut fields = Map::new();
        while let Some(name) = object.next_key::<String>()? {
            if fields.contains_key(&name) {
                return Err(de::Error::custom(format!(
                    "duplicate JSON object member {name:?}"
                )));
            }
            let value = object.next_value::<UniqueValue>()?;
            fields.insert(name, value.0);
        }
        Ok(UniqueValue(Value::Object(fields)))
    }
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn duplicate_error_identifies_decoded_name_and_location() {
        for text in [r#"{"name":1,"name":2}"#, r#"{"name":1,"\u006eame":2}"#] {
            let error = parse(text).expect_err("duplicate name");
            assert!(
                error
                    .to_string()
                    .contains("duplicate JSON object member \"name\""),
                "{error}"
            );
            assert_eq!(error.line(), 1);
            assert!(error.column() > 1);
        }
    }

    #[test]
    fn ordinary_json_values_keep_their_representation() {
        let text = r#"[null,true,false,-2,18446744073709551615,1.5,"text",{"x":1},{"x":2}]"#;
        assert_eq!(
            parse(text).unwrap(),
            serde_json::from_str::<serde_json::Value>(text).unwrap()
        );
        assert!(parse("null true").is_err());
    }
}
