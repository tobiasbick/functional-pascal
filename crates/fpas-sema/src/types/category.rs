//! Stable semantic categories for tooling; independent of formatted type spelling.

use super::Ty;

impl Ty {
    /// Returns the exhaustive category of this checked semantic type.
    #[must_use]
    pub fn category(&self) -> &'static str {
        match self {
            Self::Integer => "integer",
            Self::Subrange(_) => "subrange",
            Self::Real => "real",
            Self::Boolean => "boolean",
            Self::String => "string",
            Self::Unit => "unit",
            Self::Array(_) => "array",
            Self::Channel(_) => "channel",
            Self::Record(_) => "record",
            Self::Enum(_) => "enum",
            Self::Distinct(_) => "distinct",
            Self::Function(_) | Self::Procedure(_) => "callable",
            Self::Named(_) => "unresolved",
            Self::Result(..) => "result",
            Self::Option(_) => "option",
            Self::GenericParam(..) => "generic_parameter",
            Self::Dict(..) => "dict",
            Self::Task(_) => "task",
            Self::Error => "error",
        }
    }
}
