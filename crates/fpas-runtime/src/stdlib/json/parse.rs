//! JSON deserialization that rejects duplicate decoded object member names.
//!
//! **Documentation:** `docs/pascal/std/text/json.md`.

use serde::de::{self, Deserialize, DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::value::RawValue;
use serde_json::{Map, Number, Value};
use std::fmt;

/// Parses JSON while retaining duplicate-name errors before constructing maps.
pub(super) fn parse(text: &str) -> Result<Value, ParseError> {
    let mut deserializer = serde_json::Deserializer::from_str(text);
    let mut source = Source {
        text,
        failure: None,
    };
    let value = UniqueSeed {
        depth: 1,
        source: &mut source,
    }
    .deserialize(&mut deserializer)
    .map_err(|error| {
        source
            .failure
            .take()
            .unwrap_or_else(|| ParseError::new(error))
    })?;
    deserializer.end().map_err(ParseError::new)?;
    Ok(value)
}

#[derive(Debug)]
pub(super) struct ParseError {
    detail: String,
    line: usize,
    column: usize,
}

impl ParseError {
    fn new(error: serde_json::Error) -> Self {
        let rendered = error.to_string();
        let suffix = format!(" at line {} column {}", error.line(), error.column());
        Self {
            detail: rendered
                .strip_suffix(&suffix)
                .unwrap_or(&rendered)
                .to_owned(),
            line: error.line(),
            column: error.column(),
        }
    }

    pub(super) fn line(&self) -> usize {
        self.line
    }
    pub(super) fn column(&self) -> usize {
        self.column
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.detail)?;
        if self.line > 0 {
            write!(formatter, " at line {} column {}", self.line, self.column)?;
        }
        Ok(())
    }
}

struct Source<'a> {
    text: &'a str,
    failure: Option<ParseError>,
}

impl Source<'_> {
    fn remember(&mut self, raw: &str, error: serde_json::Error) {
        if self.failure.is_some() {
            return;
        }
        let mut error = ParseError::new(error);
        let start = (raw.as_ptr() as usize).saturating_sub(self.text.as_ptr() as usize);
        let relative = if error.line > 0 {
            raw.split_inclusive('\n')
                .take(error.line - 1)
                .map(str::len)
                .sum::<usize>()
                + error.column
        } else {
            raw.len()
        };
        let offset = start.saturating_add(relative).min(self.text.len());
        let prefix = &self.text.as_bytes()[..offset];
        error.line = prefix.iter().filter(|byte| **byte == b'\n').count() + 1;
        error.column = prefix
            .iter()
            .rposition(|byte| *byte == b'\n')
            .map_or(offset, |last| offset - last - 1);
        self.failure = Some(error);
    }
}

struct UniqueSeed<'a, 'b> {
    depth: usize,
    source: &'a mut Source<'b>,
}

impl<'de> DeserializeSeed<'de> for UniqueSeed<'_, '_> {
    type Value = Value;
    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        let raw = <&RawValue>::deserialize(deserializer)?;
        value(raw.get(), self.depth, self.source).map_err(|error| {
            self.source.remember(raw.get(), error);
            de::Error::custom("Invalid JSON value")
        })
    }
}

fn value(text: &str, depth: usize, source: &mut Source<'_>) -> Result<Value, serde_json::Error> {
    if depth > crate::stdlib::limits::MAX_JSON_DEPTH {
        return Err(de::Error::custom("JSON nesting exceeds maximum depth"));
    }
    let mut deserializer = serde_json::Deserializer::from_str(text);
    match text.trim_start().as_bytes().first().copied() {
        Some(b'{') => deserializer.deserialize_map(UniqueVisitor { depth, source }),
        Some(b'[') => deserializer.deserialize_seq(UniqueVisitor { depth, source }),
        Some(b'-' | b'0'..=b'9') => {
            // Read the numeric token directly so serde's private arbitrary-precision
            // representation never becomes an FPAS JSON object.
            let number = if text == "-0" {
                Number::from_f64(-0.0).ok_or_else(|| {
                    <serde_json::Error as de::Error>::custom("Invalid negative zero")
                })?
            } else if let Ok(value) = text.parse::<i64>() {
                Number::from(value)
            } else if let Ok(value) = text.parse::<u64>() {
                Number::from(value)
            } else {
                text.parse::<f64>()
                    .ok()
                    .filter(|value| value.is_finite())
                    .and_then(Number::from_f64)
                    .ok_or_else(|| {
                        <serde_json::Error as de::Error>::custom("JSON number is not finite")
                    })?
            };
            Ok(Value::Number(number))
        }
        _ => serde_json::from_str(text),
    }
}

struct UniqueVisitor<'a, 'b> {
    depth: usize,
    source: &'a mut Source<'b>,
}

impl<'de> Visitor<'de> for UniqueVisitor<'_, '_> {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value with unique object member names")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Self::Value, A::Error> {
        let mut items = Vec::new();
        while let Some(value) = sequence.next_element_seed(UniqueSeed {
            depth: self.depth + 1,
            source: self.source,
        })? {
            items.push(value);
        }
        Ok(Value::Array(items))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut object: A) -> Result<Self::Value, A::Error> {
        let mut fields = Map::new();
        while let Some(name) = object.next_key::<String>()? {
            if fields.contains_key(&name) {
                return Err(de::Error::custom(format!(
                    "duplicate JSON object member {name:?}"
                )));
            }
            let value = object.next_value_seed(UniqueSeed {
                depth: self.depth + 1,
                source: self.source,
            })?;
            fields.insert(name, value);
        }
        Ok(Value::Object(fields))
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
    fn nested_errors_retain_the_original_document_position() {
        let error = parse("{\n \"outer\": {\n  \"key\": 1,\n  \"key\": 2\n }\n}").unwrap_err();
        assert_eq!((error.line(), error.column()), (4, 7));
        assert_eq!(
            error.to_string(),
            "duplicate JSON object member \"key\" at line 4 column 7"
        );
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

    #[test]
    fn arbitrary_precision_unification_preserves_finite_reals_and_literal_private_keys() {
        let value =
            parse(r#"{"real":1.25,"key":{"$serde_json::private::Number":"ordinary"}}"#).unwrap();
        assert_eq!(value["real"].as_f64(), Some(1.25));
        assert_eq!(value["key"]["$serde_json::private::Number"], "ordinary");
        assert!(parse("-0").unwrap().as_f64().unwrap().is_sign_negative());
        for text in ["1e400", "-1e400", "NaN", "[1e400]"] {
            assert!(parse(text).is_err(), "{text}");
        }
    }
}
