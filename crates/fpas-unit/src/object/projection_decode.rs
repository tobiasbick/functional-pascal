//! Charge owned projection budgets before allocating children or nominal names.

use super::{ObjectDebugProjection, ObjectDebugProjectionKind};
use serde::de::{self, DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor};
use std::fmt;

#[derive(Default)]
struct Budget {
    nodes: usize,
    names: usize,
}

struct Node<'a> {
    budget: &'a mut Budget,
    depth: usize,
}

impl<'de> serde::Deserialize<'de> for ObjectDebugProjection {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Node {
            budget: &mut Budget::default(),
            depth: 0,
        }
        .deserialize(deserializer)
    }
}

impl<'de> DeserializeSeed<'de> for Node<'_> {
    type Value = ObjectDebugProjection;
    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        self.budget.nodes += 1;
        if self.depth > fpas_bytecode::MAX_DEBUG_PROJECTION_DEPTH
            || self.budget.nodes > fpas_bytecode::MAX_DEBUG_PROJECTION_NODES
        {
            return Err(de::Error::custom(
                "debug projection depth or node count exceeds limit",
            ));
        }
        deserializer.deserialize_struct("ObjectDebugProjection", &["ty", "type_name", "kind"], self)
    }
}

impl<'de> Visitor<'de> for Node<'_> {
    type Value = ObjectDebugProjection;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a bounded debug projection")
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut ty = None;
        let mut name = None;
        let mut kind = None;
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "ty" => {
                    if ty.is_some() {
                        return Err(de::Error::duplicate_field("ty"));
                    }
                    ty = Some(map.next_value()?);
                }
                "type_name" => {
                    if name.is_some() {
                        return Err(de::Error::duplicate_field("type_name"));
                    }
                    name = Some(map.next_value_seed(Name {
                        budget: self.budget,
                    })?);
                }
                "kind" => {
                    if kind.is_some() {
                        return Err(de::Error::duplicate_field("kind"));
                    }
                    kind = Some(map.next_value_seed(Kind {
                        budget: self.budget,
                        depth: self.depth,
                    })?);
                }
                _ => {
                    let _ = map.next_value::<de::IgnoredAny>()?;
                }
            }
        }
        Ok(ObjectDebugProjection {
            ty: ty.ok_or_else(|| de::Error::missing_field("ty"))?,
            type_name: name.unwrap_or(None),
            kind: kind.ok_or_else(|| de::Error::missing_field("kind"))?,
        })
    }
}

struct Kind<'a> {
    budget: &'a mut Budget,
    depth: usize,
}

impl<'de> DeserializeSeed<'de> for Kind<'_> {
    type Value = ObjectDebugProjectionKind;
    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_enum(
            "ObjectDebugProjectionKind",
            &[
                "Unsupported",
                "Operational",
                "Scalar",
                "Array",
                "Record",
                "Reference",
            ],
            self,
        )
    }
}

impl<'de> Visitor<'de> for Kind<'_> {
    type Value = ObjectDebugProjectionKind;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a stored projection kind")
    }
    fn visit_enum<A: EnumAccess<'de>>(self, data: A) -> Result<Self::Value, A::Error> {
        let (tag, variant) = data.variant::<String>()?;
        match tag.as_str() {
            "Unsupported" => {
                variant.unit_variant()?;
                Ok(ObjectDebugProjectionKind::Unsupported)
            }
            "Operational" => {
                variant.unit_variant()?;
                Ok(ObjectDebugProjectionKind::Operational)
            }
            "Scalar" => {
                variant.unit_variant()?;
                Ok(ObjectDebugProjectionKind::Scalar)
            }
            "Array" | "Reference" => {
                let child = variant.newtype_variant_seed(Node {
                    budget: self.budget,
                    depth: self.depth + 1,
                })?;
                Ok(if tag == "Array" {
                    ObjectDebugProjectionKind::Array(Box::new(child))
                } else {
                    ObjectDebugProjectionKind::Reference(Box::new(child))
                })
            }
            "Record" => variant
                .newtype_variant_seed(Fields {
                    budget: self.budget,
                    depth: self.depth + 1,
                })
                .map(ObjectDebugProjectionKind::Record),
            _ => Err(de::Error::unknown_variant(
                &tag,
                &[
                    "Unsupported",
                    "Operational",
                    "Scalar",
                    "Array",
                    "Record",
                    "Reference",
                ],
            )),
        }
    }
}

struct Fields<'a> {
    budget: &'a mut Budget,
    depth: usize,
}

impl<'de> DeserializeSeed<'de> for Fields<'_> {
    type Value = Vec<ObjectDebugProjection>;
    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_seq(self)
    }
}

impl<'de> Visitor<'de> for Fields<'_> {
    type Value = Vec<ObjectDebugProjection>;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("bounded stored projection fields")
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Self::Value, A::Error> {
        let mut fields = Vec::new();
        while let Some(field) = sequence.next_element_seed(Node {
            budget: self.budget,
            depth: self.depth,
        })? {
            fields.push(field);
        }
        Ok(fields)
    }
}

struct Name<'a> {
    budget: &'a mut Budget,
}

impl<'de> DeserializeSeed<'de> for Name<'_> {
    type Value = Option<String>;
    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_option(self)
    }
}

impl<'de> Visitor<'de> for Name<'_> {
    type Value = Option<String>;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an optional bounded nominal name")
    }
    fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(None)
    }
    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(None)
    }
    fn visit_some<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        deserializer
            .deserialize_str(NameText {
                budget: self.budget,
            })
            .map(Some)
    }
}

struct NameText<'a> {
    budget: &'a mut Budget,
}

impl<'de> Visitor<'de> for NameText<'_> {
    type Value = String;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a bounded nominal name")
    }
    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        self.budget.names = self.budget.names.saturating_add(value.len());
        if value.is_empty() || self.budget.names > fpas_bytecode::MAX_DEBUG_PROJECTION_NAME_BYTES {
            return Err(E::custom("debug projection nominal names exceed limit"));
        }
        Ok(value.to_owned())
    }
}
