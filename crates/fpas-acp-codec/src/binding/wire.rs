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
            Self::Raw(raw) => Ok(raw
                .view()
                .members()?
                .into_iter()
                .map(|(name, raw)| (name, Self::Raw(raw.to_owned())))
                .collect()),
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
