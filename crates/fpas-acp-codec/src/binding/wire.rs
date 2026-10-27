//! Ordered JSON output containing raw arbitrary subtrees.

use crate::{CodecError, RawJson, codec};
use serde::{
    Serialize, Serializer,
    ser::{SerializeMap, SerializeSeq},
};

pub(super) enum Wire {
    Raw(RawJson),
    Array(Vec<Self>),
    Object(Vec<(String, Self)>),
}

impl Wire {
    pub fn parse(text: &str) -> Result<Self, CodecError> {
        RawJson::parse(text).map(Self::Raw)
    }
    pub fn scalar(value: &impl Serialize) -> Result<Self, CodecError> {
        codec::encode(value, crate::raw::MAX_RAW_BYTES).map(Self::Raw)
    }
    pub fn finish(self) -> Result<RawJson, CodecError> {
        codec::encode(&self, crate::raw::MAX_RAW_BYTES)
    }
    pub fn object(self) -> Result<Vec<(String, Self)>, CodecError> {
        match self {
            Self::Object(fields) => Ok(fields),
            Self::Raw(raw) => {
                let fields =
                    serde_json::from_str::<Object<'_>>(raw.text()).map_err(CodecError::json)?;
                fields
                    .0
                    .into_iter()
                    .map(|(name, raw)| Ok((name, Self::parse(raw.get())?)))
                    .collect()
            }
            _ => Err(super::invalid()),
        }
    }
}

impl Serialize for Wire {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Raw(raw) => raw.serialize(serializer),
            Self::Array(values) => {
                let mut sequence = serializer.serialize_seq(Some(values.len()))?;
                for value in values {
                    sequence.serialize_element(value)?;
                }
                sequence.end()
            }
            Self::Object(fields) => {
                let mut map = serializer.serialize_map(Some(fields.len()))?;
                for (name, value) in fields {
                    map.serialize_entry(name, value)?;
                }
                map.end()
            }
        }
    }
}

pub(super) struct Object<'a>(pub Vec<(String, &'a serde_json::value::RawValue)>);

impl<'de> serde::Deserialize<'de> for Object<'de> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Object<'de>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a JSON object")
            }
            fn visit_map<M: serde::de::MapAccess<'de>>(
                self,
                mut map: M,
            ) -> Result<Self::Value, M::Error> {
                let mut values = Vec::new();
                while let Some(value) = map.next_entry()? {
                    values.push(value);
                }
                Ok(Object(values))
            }
        }
        deserializer.deserialize_map(Visitor)
    }
}
