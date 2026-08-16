//! Source breakpoint requests and deterministic sequence-point binding.

use fpas_bytecode::{InstructionAddress, VerifiedExecutable};

use crate::vm::debug::types::SourceLocation;

/// Requested source breakpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceBreakpoint {
    /// Portable source path recorded in the executable.
    pub source: String,
    /// One-based requested line.
    pub line: u32,
    /// Optional one-based requested column.
    pub column: Option<u32>,
}

/// Breakpoint binding result retained by a debug session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundBreakpoint {
    /// Stable session-local breakpoint identifier.
    pub id: u64,
    /// Original request.
    pub requested: SourceBreakpoint,
    /// Verified executable location, absent when the requested line has no sequence point.
    pub location: Option<SourceLocation>,
    /// Bound global instruction address, absent for an unverified breakpoint.
    pub instruction: Option<u32>,
}

impl BoundBreakpoint {
    /// Return whether this breakpoint is bound to executable code.
    #[must_use]
    pub const fn is_verified(&self) -> bool {
        self.instruction.is_some()
    }
}

pub(in crate::vm::debug) fn bind(
    executable: &VerifiedExecutable,
    id: u64,
    requested: SourceBreakpoint,
) -> BoundBreakpoint {
    let image = executable.executable();
    let candidate = image
        .functions
        .iter()
        .flat_map(|function| &function.debug.sequence_points)
        .filter_map(|point| {
            let source = image
                .source_map
                .sources
                .get(point.location.source.get() as usize)
                .and_then(|name| image.strings.get(*name))?;
            if source != requested.source || point.location.line != requested.line {
                return None;
            }
            if requested
                .column
                .is_some_and(|column| point.location.column < column)
            {
                return None;
            }
            Some(point)
        })
        .min_by_key(|point| (point.location.column, point.instruction));
    let (location, instruction) = candidate.map_or((None, None), |point| {
        (
            Some(SourceLocation {
                source: requested.source.clone(),
                line: point.location.line,
                column: point.location.column,
            }),
            Some(point.instruction.get()),
        )
    });
    BoundBreakpoint {
        id,
        requested,
        location,
        instruction,
    }
}

pub(in crate::vm::debug) fn point_at(
    executable: &VerifiedExecutable,
    function: fpas_bytecode::FunctionId,
    instruction: InstructionAddress,
) -> Option<&fpas_bytecode::SequencePoint> {
    executable
        .executable()
        .functions
        .get(usize::from(function.get()))?
        .debug
        .sequence_points
        .binary_search_by_key(&instruction, |point| point.instruction)
        .ok()
        .and_then(|index| {
            executable.executable().functions[usize::from(function.get())]
                .debug
                .sequence_points
                .get(index)
        })
}

pub(in crate::vm::debug) fn source_location(
    executable: &VerifiedExecutable,
    point: &fpas_bytecode::SequencePoint,
) -> Option<SourceLocation> {
    let image = executable.executable();
    let source = image
        .source_map
        .sources
        .get(point.location.source.get() as usize)
        .and_then(|name| image.strings.get(*name))?;
    Some(SourceLocation {
        source: source.to_string(),
        line: point.location.line,
        column: point.location.column,
    })
}
