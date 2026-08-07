//! Register executable verifier and contextual failures.

mod calls;
mod control_flow;
mod instruction;
mod layouts;
mod resources;
mod source_map;

use std::fmt;

use crate::{FunctionId, InstructionAddress, InstructionError, Opcode, ReturnConvention, StringId};

/// Context attached to one deterministic verifier failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    /// Dense function identifier, when the failure belongs to code or function metadata.
    pub function: Option<FunctionId>,
    /// Canonical function name when its string reference was valid.
    pub function_name: Option<String>,
    /// Global instruction address, when the failure belongs to an instruction.
    pub instruction: Option<InstructionAddress>,
    /// Decoded opcode, when available.
    pub opcode: Option<Opcode>,
    /// Specific violated invariant.
    pub kind: ValidationErrorKind,
}

impl ValidationError {
    pub(super) fn executable(kind: ValidationErrorKind) -> Self {
        Self {
            function: None,
            function_name: None,
            instruction: None,
            opcode: None,
            kind,
        }
    }

    pub(super) fn function(
        executable: &crate::Executable,
        function: FunctionId,
        kind: ValidationErrorKind,
    ) -> Self {
        let function_name = executable
            .functions
            .get(usize::from(function.get()))
            .and_then(|info| executable.strings.get(info.name))
            .map(str::to_owned);
        Self {
            function: Some(function),
            function_name,
            instruction: None,
            opcode: None,
            kind,
        }
    }

    pub(super) fn instruction(
        executable: &crate::Executable,
        function: FunctionId,
        instruction: InstructionAddress,
        opcode: Option<Opcode>,
        kind: ValidationErrorKind,
    ) -> Self {
        let mut error = Self::function(executable, function, kind);
        error.instruction = Some(instruction);
        error.opcode = opcode;
        error
    }
}

/// Specific register executable invariant violated by untrusted input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationErrorKind {
    /// A bounded table or byte count exceeds its configured maximum.
    ResourceLimit {
        /// Resource whose limit was exceeded.
        resource: &'static str,
        /// Observed count.
        actual: usize,
        /// Accepted maximum.
        maximum: usize,
    },
    /// The root entry function is not dense function zero or does not exist.
    EntryFunction {
        /// Encoded entry identifier.
        actual: u16,
        /// Number of available functions.
        functions: usize,
    },
    /// Function zero does not have the required root initializer signature.
    EntrySignature {
        /// Encoded parameter count.
        arity: u8,
        /// Encoded capture count.
        captures: u16,
        /// Encoded return convention.
        return_convention: ReturnConvention,
    },
    /// A metadata string identifier does not exist.
    StringReference {
        /// Metadata concern containing the reference.
        owner: &'static str,
        /// Invalid string identifier.
        actual: u32,
        /// Number of available strings.
        strings: usize,
    },
    /// An interned string occurs more than once.
    DuplicateString {
        /// Later duplicate string identifier.
        duplicate: StringId,
        /// First matching string identifier.
        first: StringId,
    },
    /// A persistent function constant is capturing or task-bound.
    ConstantFunction {
        /// Referenced function identifier.
        function: u16,
        /// Referenced function capture count.
        captures: u16,
        /// Encoded task-bound flag.
        task_bound: bool,
    },
    /// A function has an empty or reversed code range.
    EmptyCodeRange {
        /// Encoded range start.
        start: u32,
        /// Encoded range end.
        end: u32,
    },
    /// A function code range exceeds the executable stream.
    CodeRange {
        /// Encoded range start.
        start: u32,
        /// Encoded range end.
        end: u32,
        /// Total instruction count.
        code: usize,
    },
    /// Function ranges overlap or leave executable code unowned.
    FunctionPartition {
        /// Expected start from the preceding function end.
        expected_start: u32,
        /// Actual start of the current function or final code end.
        actual_start: u32,
    },
    /// Parameters and captures do not fit in the declared register window.
    FrameWindow {
        /// Parameter count.
        arity: u8,
        /// Capture count.
        captures: u16,
        /// Declared register count.
        registers: u16,
    },
    /// A packed instruction has an unknown opcode or invalid form.
    Instruction(InstructionError),
    /// The known Ax opcode is reserved and cannot execute yet.
    ReservedOpcode,
    /// An operand required to be zero or the sentinel is not canonical.
    NonCanonicalOperand {
        /// Operand name.
        operand: &'static str,
        /// Encoded value.
        actual: u64,
        /// Required encoded value.
        expected: u64,
    },
    /// A register operand is the sentinel or outside the owning frame.
    Register {
        /// Operand role.
        operand: &'static str,
        /// Encoded register.
        actual: u16,
        /// Exclusive valid register limit.
        register_count: u16,
    },
    /// An instruction or constant references a missing dense table entry.
    TableReference {
        /// Referenced table.
        table: &'static str,
        /// Operand role.
        operand: &'static str,
        /// Encoded identifier.
        actual: u64,
        /// Exclusive table length.
        length: usize,
    },
    /// An intrinsic wire identifier has no registered operation.
    UnknownIntrinsic {
        /// Unknown intrinsic identifier.
        actual: u16,
    },
    /// A contiguous register window is outside the function frame.
    RegisterWindow {
        /// Window purpose.
        operand: &'static str,
        /// First encoded register.
        base: u16,
        /// Number of registers.
        count: usize,
        /// Exclusive frame register limit.
        register_count: u16,
    },
    /// A direct call's argument count differs from target metadata.
    CallArity {
        /// Target function identifier.
        target: u16,
        /// Declared target arity.
        expected: u8,
        /// Encoded argument count.
        actual: u8,
    },
    /// A branch leaves the current function's code range.
    BranchTarget {
        /// Encoded target address.
        target: u32,
        /// Inclusive function start.
        start: u32,
        /// Exclusive function end.
        end: u32,
    },
    /// Reachable execution falls past a function's final instruction.
    Fallthrough,
    /// A return operand does not match function metadata.
    ReturnConvention {
        /// Declared convention.
        expected: ReturnConvention,
        /// Encoded return register or sentinel.
        actual: u16,
    },
    /// A field or variant does not belong to a valid aggregate layout.
    LayoutReference {
        /// Aggregate relationship being checked.
        operand: &'static str,
        /// Encoded local or global slot.
        actual: u16,
        /// Largest available exclusive slot count.
        available: usize,
    },
    /// Sparse source runs are not strictly ordered.
    SourceRunOrder {
        /// Previous run start.
        previous: u32,
        /// Current run start.
        actual: u32,
    },
    /// A source run address is outside executable code.
    SourceRunAddress {
        /// Invalid instruction address.
        actual: u32,
        /// Total instruction count.
        code: usize,
    },
    /// A source run references no source path.
    SourceReference {
        /// Invalid source identifier.
        actual: u32,
        /// Number of source paths.
        sources: usize,
    },
    /// A source location uses zero for a one-based line or column.
    SourcePosition {
        /// Invalid line.
        line: u32,
        /// Invalid column.
        column: u32,
    },
    /// A function boundary has no explicit source run.
    MissingFunctionSource {
        /// Function start requiring a run.
        start: u32,
    },
    /// Function spawn metadata disagrees with emitted operations.
    SpawnFlag {
        /// Declared metadata flag.
        declared: bool,
        /// Whether a spawn operation was emitted.
        emitted: bool,
    },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid register executable")?;
        if let Some(function) = self.function {
            write!(formatter, " in function {}", function.get())?;
            if let Some(name) = &self.function_name {
                write!(formatter, " (`{name}`)")?;
            }
        }
        if let Some(instruction) = self.instruction {
            write!(formatter, " at instruction {}", instruction.get())?;
        }
        if let Some(opcode) = self.opcode {
            write!(formatter, " ({opcode:?})")?;
        }
        write!(formatter, ": {:?}", self.kind)
    }
}

impl std::error::Error for ValidationError {}

pub(super) fn validate(executable: &crate::Executable) -> Result<(), ValidationError> {
    resources::validate_resources(executable)?;
    layouts::validate_tables(executable)?;
    control_flow::validate_functions(executable)?;
    source_map::validate_source_map(executable)?;
    Ok(())
}

pub(super) fn limit(
    resource: &'static str,
    actual: usize,
    maximum: usize,
) -> Result<(), ValidationError> {
    if actual <= maximum {
        Ok(())
    } else {
        Err(ValidationError::executable(
            ValidationErrorKind::ResourceLimit {
                resource,
                actual,
                maximum,
            },
        ))
    }
}
