//! Static aggregate data for checking consumers without implementation ASTs.

use super::ConstantValue;

/// Evaluated immutable data retained alongside an imported runtime global.
///
/// **Documentation:** `docs/pascal/language/basics/constants.md`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum StaticValue {
    /// Scalar data, preserving real bits.
    Scalar(ConstantValue),
    /// Ordered array elements.
    Array(Vec<StaticValue>),
    /// Normalized dictionary entries in insertion order.
    Dictionary(Vec<(StaticValue, StaticValue)>),
    /// Record fields in declaration order; the symbol type supplies nominal identity.
    Record(Vec<(String, StaticValue)>),
    /// Enum variant and associated fields; the symbol type supplies nominal identity.
    EnumVariant(String, Vec<(String, StaticValue)>),
    /// Present option payload.
    OptionSome(Box<StaticValue>),
    /// Absent option value.
    OptionNone,
    /// Successful result payload.
    ResultOk(Box<StaticValue>),
    /// Failed result payload.
    ResultError(Box<StaticValue>),
}

impl StaticValue {
    /// Normalize names recursively without reordering arrays or dictionary entries.
    pub(super) fn canonicalize(&mut self) {
        match self {
            Self::Scalar(value) => super::symbols::canonicalize_constant(value),
            Self::Array(values) => values.iter_mut().for_each(Self::canonicalize),
            Self::Dictionary(pairs) => {
                for (key, value) in pairs {
                    key.canonicalize();
                    value.canonicalize();
                }
            }
            Self::Record(fields) | Self::EnumVariant(_, fields) => {
                for (name, value) in fields {
                    *name = name.to_ascii_lowercase();
                    value.canonicalize();
                }
                if let Self::EnumVariant(name, _) = self {
                    *name = name.to_ascii_lowercase();
                }
            }
            Self::OptionSome(value) | Self::ResultOk(value) | Self::ResultError(value) => {
                value.canonicalize()
            }
            Self::OptionNone => {}
        }
    }
}
