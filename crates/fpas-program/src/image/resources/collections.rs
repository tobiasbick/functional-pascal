//! Bounded collection visitors used by the payload decoder.

use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::fmt;
use std::marker::PhantomData;

use fpas_bytecode::PersistentValue;
use serde::Deserialize;
use serde::de::{self, DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};

use super::persistent_string_bytes;
use crate::image::payload::EncodedFunction;

pub(super) struct BoundedVecSeed<T> {
    field: &'static str,
    maximum: usize,
    marker: PhantomData<T>,
}

impl<T> BoundedVecSeed<T> {
    pub(super) fn new(field: &'static str, maximum: usize) -> Self {
        Self {
            field,
            maximum,
            marker: PhantomData,
        }
    }
}

impl<'de, T> DeserializeSeed<'de> for BoundedVecSeed<T>
where
    T: Deserialize<'de>,
{
    type Value = Vec<T>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(BoundedVecVisitor::<T> {
            field: self.field,
            maximum: self.maximum,
            marker: PhantomData,
        })
    }
}

struct BoundedVecVisitor<T> {
    field: &'static str,
    maximum: usize,
    marker: PhantomData<T>,
}

impl<'de, T> Visitor<'de> for BoundedVecVisitor<T>
where
    T: Deserialize<'de>,
{
    type Value = Vec<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "at most {} {}", self.maximum, self.field)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        reject_size_hint::<A::Error>(self.field, sequence.size_hint(), self.maximum)?;
        let mut values = Vec::with_capacity(sequence.size_hint().unwrap_or(0).min(self.maximum));
        loop {
            if values.len() == self.maximum {
                return match sequence.next_element::<IgnoredAny>()? {
                    Some(_) => Err(resource_error(self.field, self.maximum + 1, self.maximum)),
                    None => Ok(values),
                };
            }
            match sequence.next_element()? {
                Some(value) => values.push(value),
                None => return Ok(values),
            }
        }
    }
}

pub(super) struct ConstantsSeed<'a> {
    maximum: usize,
    string_bytes: &'a mut usize,
    maximum_string_bytes: usize,
}

impl<'a> ConstantsSeed<'a> {
    pub(super) fn new(
        maximum: usize,
        string_bytes: &'a mut usize,
        maximum_string_bytes: usize,
    ) -> Self {
        Self {
            maximum,
            string_bytes,
            maximum_string_bytes,
        }
    }
}

impl<'de> DeserializeSeed<'de> for ConstantsSeed<'_> {
    type Value = Vec<PersistentValue>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ConstantsVisitor {
            maximum: self.maximum,
            string_bytes: self.string_bytes,
            maximum_string_bytes: self.maximum_string_bytes,
        })
    }
}

struct ConstantsVisitor<'a> {
    maximum: usize,
    string_bytes: &'a mut usize,
    maximum_string_bytes: usize,
}

impl<'de> Visitor<'de> for ConstantsVisitor<'_> {
    type Value = Vec<PersistentValue>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "at most {} persistent constants", self.maximum)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        reject_size_hint::<A::Error>("constants", sequence.size_hint(), self.maximum)?;
        let mut values = Vec::with_capacity(sequence.size_hint().unwrap_or(0).min(self.maximum));
        loop {
            if values.len() == self.maximum {
                return match sequence.next_element::<IgnoredAny>()? {
                    Some(_) => Err(resource_error("constants", self.maximum + 1, self.maximum)),
                    None => Ok(values),
                };
            }
            let Some(value) = sequence.next_element::<PersistentValue>()? else {
                return Ok(values);
            };
            add_decoded_string_bytes::<A::Error>(
                self.string_bytes,
                persistent_string_bytes(&value),
                self.maximum_string_bytes,
            )?;
            values.push(value);
        }
    }
}

pub(super) struct FunctionsSeed<'a> {
    maximum: usize,
    string_bytes: &'a mut usize,
    maximum_string_bytes: usize,
}

impl<'a> FunctionsSeed<'a> {
    pub(super) fn new(
        maximum: usize,
        string_bytes: &'a mut usize,
        maximum_string_bytes: usize,
    ) -> Self {
        Self {
            maximum,
            string_bytes,
            maximum_string_bytes,
        }
    }
}

impl<'de> DeserializeSeed<'de> for FunctionsSeed<'_> {
    type Value = BTreeMap<String, EncodedFunction>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(FunctionsVisitor {
            maximum: self.maximum,
            string_bytes: self.string_bytes,
            maximum_string_bytes: self.maximum_string_bytes,
        })
    }
}

struct FunctionsVisitor<'a> {
    maximum: usize,
    string_bytes: &'a mut usize,
    maximum_string_bytes: usize,
}

impl<'de> Visitor<'de> for FunctionsVisitor<'_> {
    type Value = BTreeMap<String, EncodedFunction>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "at most {} function entries", self.maximum)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        reject_size_hint::<A::Error>("functions", map.size_hint(), self.maximum)?;
        let mut functions = BTreeMap::new();
        while let Some(name) = map.next_key::<String>()? {
            if functions.len() == self.maximum {
                return Err(resource_error("functions", self.maximum + 1, self.maximum));
            }
            add_decoded_string_bytes::<A::Error>(
                self.string_bytes,
                name.len(),
                self.maximum_string_bytes,
            )?;
            let function = map.next_value::<EncodedFunction>()?;
            match functions.entry(name) {
                Entry::Vacant(entry) => {
                    entry.insert(function);
                }
                Entry::Occupied(entry) => {
                    return Err(de::Error::custom(format_args!(
                        "duplicate function entry `{}`",
                        entry.key()
                    )));
                }
            }
        }
        Ok(functions)
    }
}

fn reject_size_hint<E>(field: &'static str, size: Option<usize>, maximum: usize) -> Result<(), E>
where
    E: de::Error,
{
    if let Some(size) = size
        && size > maximum
    {
        return Err(resource_error(field, size, maximum));
    }
    Ok(())
}

pub(super) fn ensure_string_limit<E>(size: usize, maximum: usize) -> Result<(), E>
where
    E: de::Error,
{
    if size > maximum {
        return Err(resource_error("strings", size, maximum));
    }
    Ok(())
}

fn add_decoded_string_bytes<E>(total: &mut usize, amount: usize, maximum: usize) -> Result<(), E>
where
    E: de::Error,
{
    let size = total.checked_add(amount).unwrap_or(usize::MAX);
    ensure_string_limit::<E>(size, maximum)?;
    *total = size;
    Ok(())
}

fn resource_error<E>(field: &'static str, size: usize, maximum: usize) -> E
where
    E: de::Error,
{
    de::Error::custom(format_args!(
        "payload resource `{field}` has size {size}, exceeding limit {maximum}"
    ))
}
