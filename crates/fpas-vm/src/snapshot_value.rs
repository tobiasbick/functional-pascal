//! Borrowed JSON value serialization; dictionary entries remain ordered key/value children.

use crate::{FailureScalar, FailureValue};
use serde::{
    Serialize, Serializer,
    ser::{SerializeSeq, SerializeStruct},
};

/// Additional document-size clipping, applied without copying the captured tree.
#[derive(Clone, Copy)]
pub struct Policy {
    /// Maximum children retained by the document view.
    pub children: usize,
    /// Maximum Unicode scalars retained in text payloads.
    pub chars: usize,
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            children: 256,
            chars: 4096,
        }
    }
}

/// View over an already bounded immutable value, without cloning its tree.
pub struct Value<'a> {
    /// Already detached runtime value.
    pub data: &'a FailureValue,
    /// Additional serialized-size clipping policy.
    pub policy: Policy,
}
impl<'a> Value<'a> {
    /// Serialize the complete retained capture unless the planner supplies a smaller policy.
    pub fn new(data: &'a FailureValue) -> Self {
        Self {
            data,
            policy: Policy::default(),
        }
    }
}

impl Serialize for Value<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let value = self.data;
        let mut count = value.children.len().min(self.policy.children);
        if value.kind == "dict" {
            count -= count % 2;
        }
        let summary = prefix(&value.summary, self.policy.chars);
        let clipped = count < value.children.len()
            || summary.len() < value.summary.len()
            || matches!(&value.scalar, Some(FailureScalar::String(text)) if prefix(text, self.policy.chars).len() < text.len());
        let mut object = serializer.serialize_struct("Value", 9)?;
        object.serialize_field("type", &value.type_name)?;
        object.serialize_field("kind", value.kind)?;
        object.serialize_field("status", if clipped { "truncated" } else { value.status })?;
        object.serialize_field("scalar", &Scalar(value.scalar.as_ref(), self.policy.chars))?;
        object.serialize_field("summary", summary)?;
        object.serialize_field("children", &Children(&value.children[..count], self.policy))?;
        object.serialize_field("total_children", &value.total_children)?;
        object.serialize_field(
            "reason",
            &if clipped {
                Some("byte_limit")
            } else {
                value.reason
            },
        )?;
        object.end()
    }
}

struct Scalar<'a>(Option<&'a FailureScalar>, usize);
impl Serialize for Scalar<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.0 {
            Some(FailureScalar::Integer(value)) => serializer.serialize_str(value),
            Some(FailureScalar::String(value)) => serializer.serialize_str(prefix(value, self.1)),
            Some(FailureScalar::Real(value)) => serializer.serialize_f64(*value),
            Some(FailureScalar::Boolean(value)) => serializer.serialize_bool(*value),
            Some(FailureScalar::Task(value)) => serializer.serialize_u64(*value),
            None => serializer.serialize_none(),
        }
    }
}

fn prefix(text: &str, maximum: usize) -> &str {
    text.char_indices()
        .nth(maximum)
        .map_or(text, |(end, _)| &text[..end])
}

struct Children<'a>(&'a [(String, FailureValue)], Policy);
impl Serialize for Children<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut children = serializer.serialize_seq(Some(self.0.len()))?;
        for (name, value) in self.0 {
            #[derive(Serialize)]
            struct Child<'a> {
                name: &'a str,
                value: Value<'a>,
            }
            children.serialize_element(&Child {
                name,
                value: Value {
                    data: value,
                    policy: self.1,
                },
            })?;
        }
        children.end()
    }
}
