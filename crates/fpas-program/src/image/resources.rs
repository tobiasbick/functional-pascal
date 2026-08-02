//! Resource-bounded JSON deserialization for executable payloads.

mod collections;
#[cfg(test)]
mod tests;

use std::fmt;

use fpas_bytecode::{Op, PersistentValue};
use serde::de::{self, DeserializeSeed, MapAccess, Visitor};

use self::collections::{BoundedVecSeed, ConstantsSeed, FunctionsSeed, ensure_string_limit};
use super::ImageError;
use super::payload::{EncodedChunk, EncodedLocation};

pub(crate) const MAX_INSTRUCTIONS: usize = 1024 * 1024;
pub(crate) const MAX_LOCATIONS: usize = MAX_INSTRUCTIONS;
pub(crate) const MAX_FUNCTIONS: usize = 65_535;
pub(crate) const MAX_CONSTANTS: usize = fpas_bytecode::MAX_CONSTANT_INDEX as usize + 1;
pub(crate) const MAX_TOTAL_STRING_BYTES: usize = 16 * 1024 * 1024;

const DEFAULT_LIMITS: ResourceLimits = ResourceLimits {
    instructions: MAX_INSTRUCTIONS,
    locations: MAX_LOCATIONS,
    functions: MAX_FUNCTIONS,
    constants: MAX_CONSTANTS,
    string_bytes: MAX_TOTAL_STRING_BYTES,
};

#[derive(Clone, Copy)]
struct ResourceLimits {
    instructions: usize,
    locations: usize,
    functions: usize,
    constants: usize,
    string_bytes: usize,
}

pub(crate) fn check_resource_size(
    field: &'static str,
    size: usize,
    maximum: usize,
) -> Result<(), ImageError> {
    if size > maximum {
        return Err(ImageError::ResourceLimit {
            field,
            size,
            maximum,
        });
    }
    Ok(())
}

pub(crate) fn add_string_bytes(total: &mut usize, amount: usize) -> Result<(), ImageError> {
    let size = total.checked_add(amount).unwrap_or(usize::MAX);
    check_resource_size("strings", size, MAX_TOTAL_STRING_BYTES)?;
    *total = size;
    Ok(())
}

pub(crate) fn persistent_string_bytes(value: &PersistentValue) -> usize {
    match value {
        PersistentValue::String(value) => value.len(),
        PersistentValue::Function { name, .. } => name.len(),
        PersistentValue::Integer(_)
        | PersistentValue::Real(_)
        | PersistentValue::Boolean(_)
        | PersistentValue::Unit => 0,
    }
}

pub(super) fn decode_encoded_chunk(
    bytes: &[u8],
    initial_string_bytes: usize,
) -> Result<EncodedChunk, serde_json::Error> {
    decode_encoded_chunk_with_limits(bytes, initial_string_bytes, DEFAULT_LIMITS)
}

fn decode_encoded_chunk_with_limits(
    bytes: &[u8],
    initial_string_bytes: usize,
    limits: ResourceLimits,
) -> Result<EncodedChunk, serde_json::Error> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let payload = ChunkSeed {
        limits,
        initial_string_bytes,
    }
    .deserialize(&mut deserializer)?;
    deserializer.end()?;
    Ok(payload)
}

struct ChunkSeed {
    limits: ResourceLimits,
    initial_string_bytes: usize,
}

impl<'de> DeserializeSeed<'de> for ChunkSeed {
    type Value = EncodedChunk;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct(
            "EncodedChunk",
            &["code", "constants", "locations", "functions"],
            ChunkVisitor {
                limits: self.limits,
                initial_string_bytes: self.initial_string_bytes,
            },
        )
    }
}

struct ChunkVisitor {
    limits: ResourceLimits,
    initial_string_bytes: usize,
}

impl<'de> Visitor<'de> for ChunkVisitor {
    type Value = EncodedChunk;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a bounded compiled-program payload")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut code = None;
        let mut constants = None;
        let mut locations = None;
        let mut functions = None;
        let mut string_bytes = self.initial_string_bytes;
        ensure_string_limit::<A::Error>(string_bytes, self.limits.string_bytes)?;

        while let Some(field) = map.next_key::<String>()? {
            match field.as_str() {
                "code" => {
                    reject_duplicate::<A::Error, _>(&code, "code")?;
                    code = Some(map.next_value_seed(BoundedVecSeed::<Op>::new(
                        "instructions",
                        self.limits.instructions,
                    ))?);
                }
                "constants" => {
                    reject_duplicate::<A::Error, _>(&constants, "constants")?;
                    constants = Some(map.next_value_seed(ConstantsSeed::new(
                        self.limits.constants,
                        &mut string_bytes,
                        self.limits.string_bytes,
                    ))?);
                }
                "locations" => {
                    reject_duplicate::<A::Error, _>(&locations, "locations")?;
                    locations = Some(map.next_value_seed(
                        BoundedVecSeed::<EncodedLocation>::new("locations", self.limits.locations),
                    )?);
                }
                "functions" => {
                    reject_duplicate::<A::Error, _>(&functions, "functions")?;
                    functions = Some(map.next_value_seed(FunctionsSeed::new(
                        self.limits.functions,
                        &mut string_bytes,
                        self.limits.string_bytes,
                    ))?);
                }
                unknown => {
                    return Err(de::Error::unknown_field(
                        unknown,
                        &["code", "constants", "locations", "functions"],
                    ));
                }
            }
        }

        Ok(EncodedChunk {
            code: code.ok_or_else(|| de::Error::missing_field("code"))?,
            constants: constants.ok_or_else(|| de::Error::missing_field("constants"))?,
            locations: locations.ok_or_else(|| de::Error::missing_field("locations"))?,
            functions: functions.ok_or_else(|| de::Error::missing_field("functions"))?,
        })
    }
}

fn reject_duplicate<E, T>(value: &Option<T>, field: &'static str) -> Result<(), E>
where
    E: de::Error,
{
    if value.is_some() {
        return Err(de::Error::duplicate_field(field));
    }
    Ok(())
}
