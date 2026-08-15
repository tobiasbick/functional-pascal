//! Fixed-width register-bytecode operands.

use std::fmt;

/// Encoded register sentinel used when an opcode has no destination or value operand.
pub const NO_REGISTER: u16 = u16::MAX;

/// A relative register in the current function frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Register(u16);

impl Register {
    /// Largest register value that can address a frame slot.
    pub const MAX: Self = Self(u16::MAX - 1);

    /// Construct a register unless `value` is the reserved sentinel.
    ///
    /// # Errors
    ///
    /// Returns [`OperandError::ReservedRegister`] for [`NO_REGISTER`].
    pub const fn new(value: u16) -> Result<Self, OperandError> {
        if value == NO_REGISTER {
            Err(OperandError::ReservedRegister)
        } else {
            Ok(Self(value))
        }
    }

    /// Return the fixed-width encoded register value.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }

    /// Convert a collection index without narrowing it.
    ///
    /// # Errors
    ///
    /// Returns a structured error when the index cannot be encoded or is the reserved sentinel.
    pub fn try_from_index(index: usize) -> Result<Self, OperandError> {
        let value = u16::try_from(index).map_err(|_| OperandError::ResourceExhausted {
            resource: "register",
            actual: index,
            maximum: usize::from(Self::MAX.get()),
        })?;
        Self::new(value)
    }
}

macro_rules! fixed_operand {
    ($name:ident, $storage:ty, $resource:literal) => {
        #[doc = concat!("Fixed-width identifier for a ", $resource, ".")]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(transparent)]
        pub struct $name($storage);

        impl $name {
            #[doc = concat!("Largest representable ", $resource, " identifier.")]
            pub const MAX: Self = Self(<$storage>::MAX);

            #[doc = concat!("Construct a ", $resource, " identifier from its encoded value.")]
            #[must_use]
            pub const fn new(value: $storage) -> Self {
                Self(value)
            }

            #[doc = concat!("Return the encoded ", $resource, " identifier.")]
            #[must_use]
            pub const fn get(self) -> $storage {
                self.0
            }

            #[doc = concat!("Convert a collection index into a ", $resource, " identifier without narrowing it.")]
            ///
            /// # Errors
            ///
            /// Returns [`OperandError::ResourceExhausted`] when the index is not representable.
            pub fn try_from_index(index: usize) -> Result<Self, OperandError> {
                let value = <$storage>::try_from(index).map_err(|_| OperandError::ResourceExhausted {
                    resource: $resource,
                    actual: index,
                    maximum: <$storage>::MAX as usize,
                })?;
                Ok(Self(value))
            }
        }
    };
}

fixed_operand!(ConstantId, u32, "constant");
fixed_operand!(StringId, u32, "string");
fixed_operand!(FunctionId, u16, "function");
fixed_operand!(GlobalId, u32, "global");
fixed_operand!(RecordTypeId, u16, "record type");
fixed_operand!(RecordFieldId, u16, "record field");
fixed_operand!(EnumTypeId, u16, "enum type");
fixed_operand!(EnumVariantId, u16, "enum variant");
fixed_operand!(IntrinsicId, u16, "intrinsic");
fixed_operand!(InstructionAddress, u32, "instruction address");
fixed_operand!(SourceId, u32, "source");
fixed_operand!(DebugTypeId, u32, "debug type");
fixed_operand!(DebugBindingId, u32, "debug binding");

/// A fixed-width operand cannot represent the requested value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperandError {
    /// The register value is reserved to encode an absent operand.
    ReservedRegister,
    /// A collection index exceeds its bytecode representation.
    ResourceExhausted {
        /// Resource whose identifier was requested.
        resource: &'static str,
        /// Requested collection index.
        actual: usize,
        /// Largest representable index.
        maximum: usize,
    },
}

impl fmt::Display for OperandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReservedRegister => write!(
                formatter,
                "register value {NO_REGISTER} is reserved for an absent operand"
            ),
            Self::ResourceExhausted {
                resource,
                actual,
                maximum,
            } => write!(
                formatter,
                "{resource} index {actual} exceeds the bytecode maximum {maximum}"
            ),
        }
    }
}

impl std::error::Error for OperandError {}
