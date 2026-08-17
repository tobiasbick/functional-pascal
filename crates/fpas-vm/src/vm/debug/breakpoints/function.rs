//! Exact executable-identity binding for logical function breakpoints.

use fpas_bytecode::{FunctionId, VerifiedExecutable};

use super::source::source_location;
use crate::vm::debug::routines::matching_functions;
use crate::vm::debug::types::SourceLocation;

/// Resource bounds for session-local debugger breakpoint state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebugBreakpointLimits {
    /// Maximum logical source, function, and data breakpoints retained by a session.
    pub max_breakpoints: usize,
    /// Maximum exact executable functions bound by one logical function selector.
    pub max_function_bindings: usize,
    /// Maximum UTF-8 bytes in one function selector.
    pub max_function_name_bytes: usize,
}

impl Default for DebugBreakpointLimits {
    fn default() -> Self {
        Self {
            max_breakpoints: 256,
            max_function_bindings: 64,
            max_function_name_bytes: 1_024,
        }
    }
}

/// Requested source-independent function breakpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionBreakpoint {
    /// Canonical or short routine selector matched against executable metadata.
    pub name: String,
}

/// One logical function breakpoint bound to exact executable identities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundFunctionBreakpoint {
    /// Stable session-local logical breakpoint identifier.
    pub id: u64,
    /// Original selector request.
    pub requested: FunctionBreakpoint,
    /// Every exact matching function identity in ascending executable order.
    pub functions: Vec<FunctionId>,
    /// Executable entry sequence-point addresses for the matching functions.
    pub instructions: Vec<u32>,
    /// Source locations corresponding to `instructions` in the same order.
    pub locations: Vec<SourceLocation>,
}

impl BoundFunctionBreakpoint {
    /// Return whether at least one matching function has an executable entry sequence point.
    #[must_use]
    pub fn is_verified(&self) -> bool {
        !self.instructions.is_empty()
    }
}

pub(in crate::vm::debug) fn bind(
    executable: &VerifiedExecutable,
    id: u64,
    requested: FunctionBreakpoint,
) -> BoundFunctionBreakpoint {
    let functions = matching_functions(executable, &requested.name);
    let mut instructions = Vec::with_capacity(functions.len());
    let mut locations = Vec::with_capacity(functions.len());
    for function in &functions {
        let Some(point) = executable
            .executable()
            .functions
            .get(usize::from(function.get()))
            .and_then(|info| info.debug.sequence_points.first())
        else {
            continue;
        };
        let Some(location) = source_location(executable, point) else {
            continue;
        };
        instructions.push(point.instruction.get());
        locations.push(location);
    }
    BoundFunctionBreakpoint {
        id,
        requested,
        functions,
        instructions,
        locations,
    }
}
