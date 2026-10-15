//! Source positions and lexical scopes without debugger session state.

use fpas_bytecode::{
    DebugSourceLocation, Executable, FunctionId, InstructionAddress, SequencePoint,
    VerifiedExecutable,
};

/// Exact sequence point at a stopped instruction boundary.
pub(in crate::vm) fn point_at(
    executable: &VerifiedExecutable,
    function: FunctionId,
    instruction: InstructionAddress,
) -> Option<&SequencePoint> {
    let points = &executable
        .executable()
        .functions
        .get(usize::from(function.get()))?
        .debug
        .sequence_points;
    points
        .binary_search_by_key(&instruction, |point| point.instruction)
        .ok()
        .and_then(|index| points.get(index))
}

/// Nearest preceding lexical point within this function, including non-statement instructions.
pub(in crate::vm) fn scope_at(
    image: &Executable,
    function: FunctionId,
    instruction: InstructionAddress,
) -> Option<u32> {
    let function = image.functions.get(usize::from(function.get()))?;
    function
        .debug
        .sequence_points
        .iter()
        .rev()
        .find(|point| point.instruction <= instruction)
        .map(|point| point.scope)
}

/// Active source position, preferring exact sequence points over the diagnostic source map.
pub(in crate::vm) fn position_at(
    image: &Executable,
    function: FunctionId,
    instruction: InstructionAddress,
) -> Option<DebugSourceLocation> {
    let function = image.functions.get(usize::from(function.get()))?;
    if !function.code.contains(instruction) {
        return None;
    }
    function
        .debug
        .sequence_points
        .iter()
        .find(|point| point.instruction == instruction)
        .map(|point| point.location)
        .or_else(|| {
            let run = image.source_map.lookup(instruction)?;
            Some(DebugSourceLocation {
                source: run.source,
                line: run.line,
                column: run.column,
            })
        })
}

/// Portable source path stored in the image, with no host filesystem lookup.
pub(in crate::vm) fn source_path(
    image: &Executable,
    location: DebugSourceLocation,
) -> Option<&str> {
    image
        .source_map
        .sources
        .get(location.source.get() as usize)
        .and_then(|name| image.strings.get(*name))
}

/// Innermost-to-outermost lexical scopes, stopping at absent metadata or a cycle.
pub(in crate::vm) fn visible_scope_chain(
    function: &fpas_bytecode::FunctionInfo,
    mut scope: u32,
) -> Vec<u32> {
    let mut visible = Vec::new();
    loop {
        if visible.contains(&scope) {
            break;
        }
        visible.push(scope);
        let Some(parent) = function
            .debug
            .scopes
            .get(scope as usize)
            .and_then(|scope| scope.parent)
        else {
            break;
        };
        scope = parent;
    }
    visible
}

/// Source heading for generated entry routines, stored name for ordinary routines.
pub(in crate::vm) fn routine_name(name: &str) -> &str {
    name.strip_prefix("$entry_").unwrap_or(name)
}
