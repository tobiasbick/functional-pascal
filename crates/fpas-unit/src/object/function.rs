//! Independently encoded register functions.

use crate::object::ObjectSourceRun;

/// Return operand convention stored without process-local Rust layout assumptions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ObjectReturn {
    /// Procedure-like Unit return.
    Unit,
    /// Function-like value return.
    Value,
}

/// One independently encoded function with local registers and branch addresses.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ObjectFunction {
    /// Canonical diagnostic name.
    pub name: String,
    /// Packed logical instruction words with function-local branch targets.
    pub code: Vec<u64>,
    /// Positional argument count.
    pub arity: u8,
    /// Captured register count immediately following parameters.
    pub capture_count: u16,
    /// Total register window size.
    pub register_count: u16,
    /// Return operand convention.
    pub returns: ObjectReturn,
    /// Whether retained or detached task spawning occurs in this function.
    pub uses_spawn_tasks: bool,
    /// Sparse source runs using function-local instruction addresses.
    pub source_runs: Vec<ObjectSourceRun>,
}
