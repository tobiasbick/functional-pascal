//! Links relocatable Functional Pascal objects into the VM's executable [`Chunk`].

mod append;
mod definitions;
mod operands;

use std::fmt;

use fpas_bytecode::{Chunk, ExecutableError, validate_executable};
use fpas_unit::object::{DefinitionKind, ObjectError, RelocatableObject};

use append::{append_object, validate_retained_function_entries};
use definitions::validate_definitions_and_imports;

/// Object-linking failure with a deterministic diagnostic message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkError {
    /// An input object failed its structural validation.
    InvalidObject {
        /// Object owner.
        owner: String,
        /// Validation detail.
        detail: String,
    },
    /// Two objects define the same canonical runtime name.
    DuplicateDefinition(String),
    /// An import has no public compatible definition.
    UnresolvedImport {
        /// Importing object.
        owner: String,
        /// Missing definition.
        name: String,
        /// Required category.
        kind: DefinitionKind,
    },
    /// Two callable tables define the same name.
    DuplicateFunction(String),
    /// A callable definition has no implementation in its owning object.
    MissingFunctionImplementation {
        /// Object that declares the callable definition.
        owner: String,
        /// Callable definition without a matching function-table entry.
        name: String,
    },
    /// A Unit callable points at code removed during startup-section concatenation.
    StrippedFunctionEntry {
        /// Object containing the invalid function entry.
        owner: String,
        /// Callable name.
        name: String,
        /// Object-local callable offset.
        offset: u32,
        /// Number of instructions retained from the object.
        retained_code: usize,
    },
    /// A local or final index exceeds the bytecode representation.
    Overflow(&'static str),
    /// Relocation metadata does not match its opcode.
    InvalidRelocation {
        /// Object owner.
        owner: String,
        /// Object-local instruction.
        instruction: u32,
    },
    /// Constant insertion failed.
    ConstantPool(String),
    /// The completed linked executable violates a bytecode invariant.
    InvalidExecutable(ExecutableError),
    /// No executable root object was supplied.
    MissingProgram,
}

impl fmt::Display for LinkError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidObject { owner, detail } => {
                write!(formatter, "cannot link invalid object `{owner}`: {detail}")
            }
            Self::DuplicateDefinition(name) => {
                write!(formatter, "duplicate linked definition `{name}`")
            }
            Self::UnresolvedImport { owner, name, kind } => write!(
                formatter,
                "object `{owner}` requires missing public {kind:?} definition `{name}`"
            ),
            Self::DuplicateFunction(name) => {
                write!(formatter, "duplicate linked callable `{name}`")
            }
            Self::MissingFunctionImplementation { owner, name } => write!(
                formatter,
                "object `{owner}` defines callable `{name}` without a matching function implementation"
            ),
            Self::StrippedFunctionEntry {
                owner,
                name,
                offset,
                retained_code,
            } => write!(
                formatter,
                "callable `{name}` in object `{owner}` starts at {offset}, but only {retained_code} instructions remain after removing the Unit terminator"
            ),
            Self::Overflow(field) => write!(formatter, "linked {field} exceeds bytecode limits"),
            Self::InvalidRelocation { owner, instruction } => write!(
                formatter,
                "object `{owner}` has invalid relocation at instruction {instruction}"
            ),
            Self::ConstantPool(detail) => {
                write!(formatter, "cannot construct linked constant pool: {detail}")
            }
            Self::InvalidExecutable(error) => {
                write!(formatter, "linked executable validation failed: {error}")
            }
            Self::MissingProgram => write!(formatter, "cannot link without a program object"),
        }
    }
}

impl std::error::Error for LinkError {}

/// Link dependency-first unit objects followed by one root program object.
///
/// # Errors
///
/// Returns [`LinkError`] when an input object, symbol relationship, relocation,
/// or the completed executable image is invalid.
pub fn link_objects(
    units: &[RelocatableObject],
    program: &RelocatableObject,
) -> Result<Chunk, LinkError> {
    validate_objects(units, program)?;
    validate_definitions_and_imports(units, program)?;

    let mut chunk = Chunk::new();
    for object in units {
        append_object(&mut chunk, object, false)?;
    }
    append_object(&mut chunk, program, true)?;
    validate_executable(&chunk).map_err(LinkError::InvalidExecutable)?;
    Ok(chunk)
}

fn validate_objects(
    units: &[RelocatableObject],
    program: &RelocatableObject,
) -> Result<(), LinkError> {
    if program.code.is_empty() {
        return Err(LinkError::MissingProgram);
    }
    for object in units.iter().chain(std::iter::once(program)) {
        object
            .validate()
            .map_err(|error| invalid_object(object, error))?;
    }
    for object in units {
        validate_retained_function_entries(object)?;
    }
    Ok(())
}

fn invalid_object(object: &RelocatableObject, error: ObjectError) -> LinkError {
    LinkError::InvalidObject {
        owner: object.owner.clone(),
        detail: error.to_string(),
    }
}
