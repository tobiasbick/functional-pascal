//! Validated raw JSON retaining number spellings, object order and unknown fields.

use std::collections::HashSet;
use std::fmt;

use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::value::RawValue;

use crate::{CodecError, ErrorKind};

/// Maximum raw value accepted independently of a smaller connection message budget.
pub const MAX_RAW_BYTES: usize = 64 * 1024 * 1024;
/// Maximum object/array nesting accepted by the raw boundary.
pub const MAX_DEPTH: usize = 128;

/// A detached, immutable JSON document whose text is never converted through f64.
#[derive(Clone)]
pub struct RawJson(Box<RawValue>);

impl RawJson {
    /// Parse exactly one JSON value and reject duplicate decoded object keys.
    pub fn parse(text: &str) -> Result<Self, CodecError> {
        Self::parse_bounded(text, MAX_RAW_BYTES)
    }

    /// Parse a value after checking the caller's byte budget.
    pub fn parse_bounded(text: &str, maximum: usize) -> Result<Self, CodecError> {
        if text.len() > maximum.min(MAX_RAW_BYTES) {
            return Err(CodecError::new(ErrorKind::LimitExceeded));
        }
        let raw = serde_json::from_str::<Box<RawValue>>(text).map_err(CodecError::json)?;
        let mut failure = None;
        // Nested raw-value checks do not have whole-document parser positions.
        inspect(raw.get(), 0, &mut failure)
            .map_err(|_| CodecError::new(failure.unwrap_or(ErrorKind::InvalidMessage)))?;
        Ok(Self(raw))
    }

    /// Original validated JSON text, including numeric spelling and object order.
    #[must_use]
    pub fn text(&self) -> &str {
        self.0.get()
    }

    /// Extract an original object member without normalizing its JSON numbers.
    /// A missing member returns `None`; non-object documents are rejected.
    pub fn member(&self, name: &str) -> Result<Option<Self>, CodecError> {
        let mut members: std::collections::HashMap<String, Box<RawValue>> =
            serde_json::from_str(self.text()).map_err(CodecError::json)?;
        Ok(members.remove(name).map(Self))
    }
}

impl PartialEq for RawJson {
    fn eq(&self, other: &Self) -> bool {
        self.text() == other.text()
    }
}
impl Eq for RawJson {}

impl fmt::Debug for RawJson {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RawJson")
            .field("bytes", &self.text().len())
            .finish_non_exhaustive()
    }
}

impl Serialize for RawJson {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RawJson {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = Box::<RawValue>::deserialize(deserializer)?;
        Self::parse(raw.get()).map_err(serde::de::Error::custom)
    }
}

fn inspect(
    text: &str,
    depth: usize,
    failure: &mut Option<ErrorKind>,
) -> Result<(), serde_json::Error> {
    let first = text.trim_start().as_bytes().first().copied();
    if first == Some(b'"') {
        serde_json::from_str::<String>(text)?;
    }
    if !matches!(first, Some(b'{' | b'[')) {
        return Ok(());
    }
    if depth >= MAX_DEPTH {
        *failure = Some(ErrorKind::DepthExceeded);
        return Err(serde::de::Error::custom("JSON depth exceeded"));
    }
    let mut deserializer = serde_json::Deserializer::from_str(text);
    Keys {
        depth: depth + 1,
        failure,
    }
    .deserialize(&mut deserializer)
}

struct Keys<'a> {
    depth: usize,
    failure: &'a mut Option<ErrorKind>,
}

impl<'de> DeserializeSeed<'de> for Keys<'_> {
    type Value = ();
    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for Keys<'_> {
    type Value = ();
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON object or array")
    }

    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<(), M::Error> {
        let mut keys = HashSet::new();
        while let Some(key) = map.next_key::<String>()? {
            if !keys.insert(key) {
                *self.failure = Some(ErrorKind::DuplicateKey);
                return Err(serde::de::Error::custom("Duplicate JSON key"));
            }
            let raw = map.next_value::<Box<RawValue>>()?;
            inspect(raw.get(), self.depth, self.failure).map_err(serde::de::Error::custom)?;
        }
        Ok(())
    }

    fn visit_seq<S: SeqAccess<'de>>(self, mut sequence: S) -> Result<(), S::Error> {
        while let Some(raw) = sequence.next_element::<Box<RawValue>>()? {
            inspect(raw.get(), self.depth, self.failure).map_err(serde::de::Error::custom)?;
        }
        Ok(())
    }
}
