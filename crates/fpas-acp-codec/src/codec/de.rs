//! Borrowed JSON deserialization with ordinary finite numbers in untagged enums.
//!
//! With arbitrary_precision, serde_json buffers decimals as a private map that
//! serde's untagged numeric variants cannot consume. Keep raw subtrees borrowed,
//! but present finite decimal tokens through the ordinary f64 visitor. Oversized
//! arbitrary numbers still use serde_json's lossless number representation.

use serde::de::{DeserializeSeed, IntoDeserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Deserializer, Error, value::RawValue};

pub(super) struct Sdk<'a>(pub &'a RawValue);

impl<'de> serde::Deserializer<'de> for Sdk<'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let text = self.0.get();
        match text.as_bytes().first() {
            Some(b'{') => {
                let fields: Object<'_> = serde_json::from_str(text)?;
                visitor.visit_map(Fields {
                    fields: fields.0.into_iter(),
                    value: None,
                })
            }
            Some(b'[') => {
                let values: Vec<&RawValue> = serde_json::from_str(text)?;
                visitor.visit_seq(Items(values.into_iter()))
            }
            Some(b'-' | b'0'..=b'9') if text.contains(['.', 'e', 'E']) => {
                if let Ok(value) = text.parse::<f64>()
                    && value.is_finite()
                {
                    visitor.visit_f64(value)
                } else {
                    serde::Deserializer::deserialize_any(&mut Deserializer::from_str(text), visitor)
                }
            }
            _ => serde::Deserializer::deserialize_any(&mut Deserializer::from_str(text), visitor),
        }
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        if self.0.get() == "null" {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        serde::Deserializer::deserialize_newtype_struct(
            &mut Deserializer::from_str(self.0.get()),
            name,
            visitor,
        )
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        serde::Deserializer::deserialize_enum(
            &mut Deserializer::from_str(self.0.get()),
            name,
            variants,
            visitor,
        )
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct seq tuple tuple_struct map struct identifier ignored_any
    }
}

struct Items<'a>(std::vec::IntoIter<&'a RawValue>);

impl<'de> SeqAccess<'de> for Items<'de> {
    type Error = Error;
    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Error> {
        self.0
            .next()
            .map(|raw| seed.deserialize(Sdk(raw)))
            .transpose()
    }
    fn size_hint(&self) -> Option<usize> {
        Some(self.0.len())
    }
}

struct Fields<'a> {
    fields: std::vec::IntoIter<(String, &'a RawValue)>,
    value: Option<&'a RawValue>,
}

impl<'de> MapAccess<'de> for Fields<'de> {
    type Error = Error;
    fn next_key_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Error> {
        let Some((key, value)) = self.fields.next() else {
            return Ok(None);
        };
        self.value = Some(value);
        seed.deserialize(key.into_deserializer()).map(Some)
    }
    fn next_value_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<T::Value, Error> {
        let value = self
            .value
            .take()
            .ok_or_else(|| <Error as serde::de::Error>::custom("missing object value"))?;
        seed.deserialize(Sdk(value))
    }
    fn size_hint(&self) -> Option<usize> {
        Some(self.fields.len())
    }
}

struct Object<'a>(Vec<(String, &'a RawValue)>);

impl<'de> serde::Deserialize<'de> for Object<'de> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ObjectVisitor;
        impl<'de> Visitor<'de> for ObjectVisitor {
            type Value = Object<'de>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("an object")
            }
            fn visit_map<M: MapAccess<'de>>(self, mut fields: M) -> Result<Self::Value, M::Error> {
                let mut values = Vec::new();
                while let Some(field) = fields.next_entry()? {
                    values.push(field);
                }
                Ok(Object(values))
            }
        }
        deserializer.deserialize_map(ObjectVisitor)
    }
}
