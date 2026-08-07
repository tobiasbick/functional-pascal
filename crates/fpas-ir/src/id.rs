//! Small, non-interchangeable identifiers used by typed IR tables.

use std::fmt;

/// Reports an identifier or collection count that does not fit the portable IR representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdConversionError {
    /// The resource that exceeded its representation.
    pub resource: &'static str,
    /// The rejected source value.
    pub value: u128,
}

impl fmt::Display for IdConversionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} value {} exceeds the maximum portable IR value",
            self.resource, self.value
        )
    }
}

impl std::error::Error for IdConversionError {}

/// Converts a host collection length to the fixed-width count used by the IR.
///
/// # Errors
///
/// Returns [`IdConversionError`] when `count` cannot be represented as `u32`.
pub fn checked_count(resource: &'static str, count: usize) -> Result<u32, IdConversionError> {
    u32::try_from(count).map_err(|_| IdConversionError {
        resource,
        value: count as u128,
    })
}

macro_rules! identifier {
    ($(#[$meta:meta])* $name:ident, $resource:literal) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(transparent)]
        pub struct $name(u32);

        impl $name {
            /// The largest representable identifier.
            pub const MAX: Self = Self(u32::MAX);

            /// Creates an identifier from its already fixed-width representation.
            #[must_use]
            pub const fn new(value: u32) -> Self {
                Self(value)
            }

            /// Returns the fixed-width representation used for deterministic ordering.
            #[must_use]
            pub const fn get(self) -> u32 {
                self.0
            }

            /// Converts a host index without silently truncating it.
            ///
            /// # Errors
            ///
            /// Returns [`IdConversionError`] when `index` does not fit this identifier.
            pub fn try_from_index(index: usize) -> Result<Self, IdConversionError> {
                u32::try_from(index).map(Self).map_err(|_| IdConversionError {
                    resource: $resource,
                    value: index as u128,
                })
            }
        }
    };
}

identifier!(/// Identifies a function in deterministic program order.
FunctionId, "function identifier");
identifier!(/// Identifies a basic block inside a function.
BlockId, "block identifier");
identifier!(/// Identifies an SSA-style value in a function.
ValueId, "value identifier");
identifier!(/// Identifies an explicit FPAS local in a function.
LocalId, "local identifier");
identifier!(/// Identifies a compact lowered type definition.
TypeId, "type identifier");
identifier!(/// Identifies a global declaration in deterministic program order.
GlobalId, "global identifier");
identifier!(/// Identifies a record layout.
RecordLayoutId, "record layout identifier");
identifier!(/// Identifies an enum layout.
EnumLayoutId, "enum layout identifier");
identifier!(/// Identifies a field inside one record layout.
FieldId, "field identifier");
identifier!(/// Identifies a variant inside one enum layout.
VariantId, "variant identifier");
identifier!(/// Identifies a registered intrinsic signature.
IntrinsicId, "intrinsic identifier");
