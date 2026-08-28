//! Structured validation for typed IR programs.

mod control_flow;
mod debug;
mod operands;

use std::fmt;

use crate::{BlockId, FunctionId, IdConversionError, Program, ValueId, checked_count};

/// Identifies the IR entity named by a validation error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityKind {
    /// A function identifier.
    Function,
    /// A basic-block identifier.
    Block,
    /// A value identifier.
    Value,
    /// An instruction index within a basic block.
    Instruction,
    /// A local identifier.
    Local,
    /// A type identifier.
    Type,
    /// A global identifier.
    Global,
    /// A record-layout identifier.
    RecordLayout,
    /// An enum-layout identifier.
    EnumLayout,
    /// A field identifier.
    Field,
    /// A variant identifier.
    Variant,
    /// An intrinsic identifier.
    Intrinsic,
    /// Function-local lexical debugger scope.
    DebugScope,
    /// Function-local debugger binding identity.
    DebugBinding,
}

/// Identifies where a validation error occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidationLocation {
    /// Function that owns the invalid entity, when applicable.
    pub function: Option<FunctionId>,
    /// Basic block that owns the invalid entity, when applicable.
    pub block: Option<BlockId>,
    /// Instruction index in the owning block, when applicable.
    pub instruction: Option<usize>,
}

impl ValidationLocation {
    /// Creates a location with no function-local owner.
    #[must_use]
    pub const fn program() -> Self {
        Self {
            function: None,
            block: None,
            instruction: None,
        }
    }
}

/// Describes the exact invariant rejected by typed-IR validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationErrorKind {
    /// An identifier appears more than once in one deterministic namespace.
    DuplicateId {
        /// Entity namespace containing the duplicate.
        entity: EntityKind,
        /// Duplicate raw identifier.
        id: u32,
    },
    /// An identifier does not match its positional table index.
    PositionalId {
        /// Positional entity table containing the mismatch.
        entity: EntityKind,
        /// Identifier required at this table index.
        expected: u32,
        /// Identifier stored at this table index.
        actual: u32,
    },
    /// An instruction or declaration references an identifier that is not present.
    UnknownId {
        /// Referenced entity namespace.
        entity: EntityKind,
        /// Missing raw identifier.
        id: u32,
    },
    /// A known value is used before its definition becomes available in its block.
    UseBeforeDefinition {
        /// Value whose definition is not yet available in this block.
        value: ValueId,
    },
    /// A block has no terminator.
    MissingTerminator,
    /// A block has more than one terminator.
    MultipleTerminators {
        /// Number of terminators attached to the block.
        count: usize,
    },
    /// A block is present but cannot be reached from the function entry.
    UnreachableBlock {
        /// Unreachable block identifier.
        block: BlockId,
    },
    /// A target receives the wrong number of block arguments.
    BlockArgumentCount {
        /// Number of target block parameters.
        expected: usize,
        /// Number of supplied arguments.
        actual: usize,
    },
    /// A target receives a block argument of the wrong type.
    BlockArgumentType {
        /// Expected raw type identifier.
        expected: u32,
        /// Actual raw type identifier.
        actual: u32,
    },
    /// An operand or result has the wrong type for its operation.
    OperandType {
        /// Human-readable operand role.
        operand: &'static str,
        /// Expected raw type identifier.
        expected: u32,
        /// Actual raw type identifier.
        actual: u32,
    },
    /// An operand or result has the wrong compact type category for its operation.
    TypeCategory {
        /// Human-readable operand role.
        operand: &'static str,
        /// Required compact type category.
        expected: &'static str,
        /// Actual raw type identifier.
        actual: u32,
    },
    /// A value-producing operation has no result definition.
    MissingResult,
    /// A side-effect-only operation unexpectedly defines a result value.
    UnexpectedResult,
    /// A direct call does not match its target signature.
    DirectCallSignature {
        /// Expected number of arguments.
        expected: usize,
        /// Actual number of arguments.
        actual: usize,
    },
    /// A call through a value does not have a function type.
    CallValueType {
        /// Actual raw type identifier of the callee value.
        actual: u32,
    },
    /// A function result does not match its declared type.
    ReturnType {
        /// Expected raw return type identifier.
        expected: u32,
        /// Actual raw return type identifier.
        actual: u32,
    },
    /// A closure supplies the wrong number of captures for the target function.
    ClosureCaptureCount {
        /// Expected number of captures.
        expected: usize,
        /// Actual number of captures.
        actual: usize,
    },
    /// A closure capture has the wrong type or representation for the target function.
    ClosureCaptureType {
        /// Capture position in semantic order.
        index: usize,
        /// Expected raw type identifier.
        expected: u32,
        /// Actual raw type identifier.
        actual: u32,
    },
    /// A record or enum operation does not match its layout type.
    LayoutReference {
        /// Expected raw layout identifier.
        expected: u32,
        /// Actual raw type identifier.
        actual: u32,
    },
    /// A fixed-width identifier or collection count would overflow.
    Conversion(IdConversionError),
    /// Initializer metadata identifies an instruction with the wrong operation.
    InvalidInitializerOperation {
        /// Entity whose initializer metadata is invalid.
        owner: EntityKind,
        /// Operation required at the initializer location.
        expected: &'static str,
    },
    /// Initializer metadata identifies a store to the wrong target.
    InvalidInitializerTarget {
        /// Entity whose initializer metadata is invalid.
        owner: EntityKind,
        /// Namespace of the store target.
        target: EntityKind,
        /// Required raw target identifier.
        expected: u32,
        /// Raw target identifier used by the store.
        actual: u32,
    },
    /// Debugger capture provenance is incomplete, ordered wrongly, or refers to an invalid owner.
    CaptureProvenance {
        /// Human-readable invariant that failed.
        reason: &'static str,
        /// Related raw identifier when one exists.
        actual: u32,
    },
}

/// A structured typed-IR validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    /// Stable owner location for the failed invariant.
    pub location: ValidationLocation,
    /// Exact rejected invariant.
    pub kind: ValidationErrorKind,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid typed IR at {:?}: {:?}",
            self.location, self.kind
        )
    }
}

impl std::error::Error for ValidationError {}

impl Program {
    /// Validates all IR tables, control flow, operands, and deterministic references.
    ///
    /// # Errors
    ///
    /// Returns a structured [`ValidationError`] instead of panicking for invalid IR.
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_program_counts(self)?;
        operands::validate_program_tables(self)?;
        debug::validate_program(self)?;
        for function in &self.functions {
            control_flow::validate_function(function)?;
            operands::validate_function(self, function)?;
            debug::validate_function(self, function)?;
        }
        if self.function(self.entry).is_none() {
            return Err(program_error(ValidationErrorKind::UnknownId {
                entity: EntityKind::Function,
                id: self.entry.get(),
            }));
        }
        Ok(())
    }
}

pub(crate) fn program_error(kind: ValidationErrorKind) -> ValidationError {
    ValidationError {
        location: ValidationLocation::program(),
        kind,
    }
}

pub(crate) fn function_error(
    function: FunctionId,
    block: Option<BlockId>,
    instruction: Option<usize>,
    kind: ValidationErrorKind,
) -> ValidationError {
    ValidationError {
        location: ValidationLocation {
            function: Some(function),
            block,
            instruction,
        },
        kind,
    }
}

fn validate_program_counts(program: &Program) -> Result<(), ValidationError> {
    let collections = [
        ("types", program.types.len()),
        ("globals", program.globals.len()),
        ("record layouts", program.record_layouts.len()),
        ("enum layouts", program.enum_layouts.len()),
        ("intrinsics", program.intrinsics.len()),
        ("functions", program.functions.len()),
    ];
    for (resource, count) in collections {
        checked_count(resource, count)
            .map(drop)
            .map_err(|error| program_error(ValidationErrorKind::Conversion(error)))?;
    }
    Ok(())
}
