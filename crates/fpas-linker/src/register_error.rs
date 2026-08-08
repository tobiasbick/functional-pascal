//! Structured register-object linker diagnostics.

use std::fmt;

/// Register-object linking failure with deterministic, agent-readable context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegisterLinkError {
    /// An input object failed local validation.
    InvalidObject {
        /// Canonical object owner.
        owner: String,
        /// Validation detail.
        detail: String,
    },
    /// A dependency object incorrectly declares a program entry.
    UnitEntry(String),
    /// The root program has no entry function.
    MissingProgramEntry,
    /// A unit initializer cannot be called as a zero-argument procedure.
    InvalidInitializer {
        /// Canonical unit owner.
        owner: String,
        /// Concrete ABI mismatch.
        detail: &'static str,
    },
    /// Two objects define the same canonical symbol.
    DuplicateDefinition(String),
    /// No definition satisfies an import.
    UnresolvedImport {
        /// Importing owner.
        owner: String,
        /// Required symbol.
        name: String,
        /// Required category.
        kind: fpas_unit::object::SymbolKind,
    },
    /// A definition is not public across an object boundary.
    PrivateImport {
        /// Importing owner.
        owner: String,
        /// Required symbol.
        name: String,
    },
    /// An import resolves to the wrong category.
    ImportKind {
        /// Importing owner.
        owner: String,
        /// Required symbol.
        name: String,
        /// Required category.
        expected: fpas_unit::object::SymbolKind,
        /// Resolved category.
        actual: fpas_unit::object::SymbolKind,
    },
    /// Callable ABI, global mutability, or type layout is incompatible.
    IncompatibleImport {
        /// Importing owner.
        owner: String,
        /// Required symbol.
        name: String,
        /// Concrete mismatch.
        detail: String,
    },
    /// Fixed-width table ID or address overflowed.
    Overflow(&'static str),
    /// Packed instruction decoding failed.
    Instruction(String),
    /// A relocation record does not match the packed opcode operand.
    InvalidRelocation {
        /// Object owner.
        owner: String,
        /// Object-local function.
        function: u32,
        /// Function-local instruction.
        instruction: u32,
        /// Concrete mismatch.
        detail: String,
    },
    /// Field slot is not present in any compatible layout.
    InvalidField {
        /// Object owner.
        owner: String,
        /// Invalid field slot.
        field: u16,
        /// Largest available field count.
        available: usize,
    },
    /// A referenced enum variant is absent from the resolved layout.
    MissingVariant {
        /// Enum type name.
        enumeration: String,
        /// Required variant.
        variant: String,
    },
    /// The final numeric executable failed full admission verification.
    InvalidExecutable(fpas_bytecode::ValidationError),
}

impl fmt::Display for RegisterLinkError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidObject { owner, detail } => write!(
                formatter,
                "cannot link invalid register object `{owner}`: {detail}"
            ),
            Self::UnitEntry(owner) => write!(
                formatter,
                "unit object `{owner}` declares a program entry; only the root object may do so"
            ),
            Self::MissingProgramEntry => {
                write!(formatter, "root register object has no entry function")
            }
            Self::InvalidInitializer { owner, detail } => write!(
                formatter,
                "unit object `{owner}` has an invalid initializer: {detail}"
            ),
            Self::DuplicateDefinition(name) => {
                write!(formatter, "duplicate canonical definition `{name}`")
            }
            Self::UnresolvedImport { owner, name, kind } => write!(
                formatter,
                "object `{owner}` requires missing public {kind:?} definition `{name}`"
            ),
            Self::PrivateImport { owner, name } => write!(
                formatter,
                "object `{owner}` cannot import private definition `{name}`"
            ),
            Self::ImportKind {
                owner,
                name,
                expected,
                actual,
            } => write!(
                formatter,
                "object `{owner}` imports `{name}` as {expected:?}, but it resolves to {actual:?}"
            ),
            Self::IncompatibleImport {
                owner,
                name,
                detail,
            } => write!(
                formatter,
                "object `{owner}` imports incompatible definition `{name}`: {detail}"
            ),
            Self::Overflow(resource) => write!(
                formatter,
                "linked register {resource} exceeds its fixed-width limit"
            ),
            Self::Instruction(detail) => {
                write!(formatter, "cannot decode relocatable instruction: {detail}")
            }
            Self::InvalidRelocation {
                owner,
                function,
                instruction,
                detail,
            } => write!(
                formatter,
                "object `{owner}` has invalid relocation at function {function}, instruction {instruction}: {detail}"
            ),
            Self::InvalidField {
                owner,
                field,
                available,
            } => write!(
                formatter,
                "object `{owner}` references field slot {field}, but linked layouts provide {available} slots"
            ),
            Self::MissingVariant {
                enumeration,
                variant,
            } => write!(
                formatter,
                "enum `{enumeration}` has no variant `{variant}` required by relocation"
            ),
            Self::InvalidExecutable(error) => {
                write!(
                    formatter,
                    "linked register executable failed verification: {error}"
                )
            }
        }
    }
}

impl std::error::Error for RegisterLinkError {}
