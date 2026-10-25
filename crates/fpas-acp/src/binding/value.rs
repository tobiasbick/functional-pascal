//! Owned values passed between the protocol codec and a language runtime.

use crate::RawJson;

/// Detached protocol value, independent of VM layout identifiers and callbacks.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// Boolean scalar.
    Boolean(bool),
    /// Signed scalar.
    Integer(i64),
    /// Exact unsigned scalar.
    Unsigned(u64),
    /// Finite binary64 scalar.
    Real(f64),
    /// Text or an erased nominal string ID.
    String(String),
    /// Validated arbitrary protocol data.
    Raw(RawJson),
    /// Ordered array.
    Array(Vec<Self>),
    /// Ordered string-keyed dictionary.
    Dictionary(Vec<(String, Self)>),
    /// Absent optional value.
    None,
    /// Present optional value.
    Some(Box<Self>),
    /// Omitted patch.
    Missing,
    /// Explicit null patch.
    Null,
    /// Patch replacement.
    Replacement(Box<Self>),
    /// Record with qualified nominal identity and declaration-ordered fields.
    Record {
        /// Qualified native declaration.
        name: String,
        /// Ordered values.
        fields: Vec<Self>,
    },
    /// Data enum with qualified nominal identity.
    Enum {
        /// Qualified native declaration.
        name: String,
        /// Source variant name.
        variant: String,
        /// Ordered associated values.
        fields: Vec<Self>,
    },
}

impl Value {
    pub(super) fn contains_unknown(&self) -> bool {
        match self {
            Self::Enum { variant, .. } if variant == "Unknown" => true,
            Self::Record { fields, .. } | Self::Enum { fields, .. } | Self::Array(fields) => {
                fields.iter().any(Self::contains_unknown)
            }
            Self::Some(value) | Self::Replacement(value) => value.contains_unknown(),
            Self::Dictionary(fields) => fields.iter().any(|(_, value)| value.contains_unknown()),
            _ => false,
        }
    }
}
