//! Sparse source paths, runs, and binary-search lookup.

use crate::{InstructionAddress, SourceId, StringId};

/// A source location that becomes effective at an instruction address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceRun {
    /// First instruction using this location.
    pub instruction_start: InstructionAddress,
    /// Source path table identifier.
    pub source: SourceId,
    /// One-based source line.
    pub line: u32,
    /// One-based source column.
    pub column: u32,
}

/// Sparse source mapping for diagnostics outside the dispatch hot path.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceMap {
    /// Source paths as references into the executable string table.
    pub sources: Vec<StringId>,
    /// Sorted location changes.
    pub runs: Vec<SourceRun>,
}

impl SourceMap {
    /// Resolve the closest source run at or before an instruction address.
    #[must_use]
    pub fn lookup(&self, address: InstructionAddress) -> Option<SourceRun> {
        let index = self
            .runs
            .partition_point(|run| run.instruction_start <= address);
        index
            .checked_sub(1)
            .and_then(|found| self.runs.get(found))
            .copied()
    }
}
