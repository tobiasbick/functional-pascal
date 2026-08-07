//! Register function metadata and code ranges.

use crate::{InstructionAddress, StringId};

/// Half-open instruction range owned by one function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodeRange {
    /// First instruction in the function.
    pub start: InstructionAddress,
    /// First instruction after the function.
    pub end: InstructionAddress,
}

impl CodeRange {
    /// Construct a half-open code range.
    #[must_use]
    pub const fn new(start: InstructionAddress, end: InstructionAddress) -> Self {
        Self { start, end }
    }

    /// Return the fixed-width instruction count when the range is ordered.
    #[must_use]
    pub const fn len(self) -> Option<u32> {
        self.end.get().checked_sub(self.start.get())
    }

    /// Return whether the range contains no instructions or is reversed.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start.get() >= self.end.get()
    }

    /// Return whether an instruction address belongs to the range.
    #[must_use]
    pub const fn contains(self, address: InstructionAddress) -> bool {
        self.start.get() <= address.get() && address.get() < self.end.get()
    }
}

/// Value convention enforced at a function's return instructions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnConvention {
    /// The function returns Unit and encodes no return register.
    Unit,
    /// The function returns a value from a register.
    Value,
}

/// Boolean properties validated against emitted operations.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FunctionFlags {
    /// Whether the function contains retained or detached task spawning.
    pub uses_spawn_tasks: bool,
}

/// Metadata for one dense register-bytecode function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionInfo {
    /// Canonical diagnostic name in the executable string table.
    pub name: StringId,
    /// Half-open instruction range owned by the function.
    pub code: CodeRange,
    /// Number of parameter registers at the start of the frame.
    pub arity: u8,
    /// Number of capture registers immediately following parameters.
    pub capture_count: u16,
    /// Total initialized register window size, excluding the sentinel.
    pub register_count: u16,
    /// Return operand convention.
    pub return_convention: ReturnConvention,
    /// Validated behavioral flags.
    pub flags: FunctionFlags,
}
