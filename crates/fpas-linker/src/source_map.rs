//! Source path merging and sparse source-run rebasing.

use std::collections::HashMap;

use fpas_bytecode::{InstructionAddress, SourceId, SourceMap, SourceRun};
use fpas_unit::object::RelocatableObject;

use crate::RegisterLinkError;
use crate::strings::StringInterner;

pub(super) fn merge(
    objects: &[&RelocatableObject],
    function_order: &[(usize, usize)],
    code_starts: &[u32],
    code_bases: &[u32],
    strings: &mut StringInterner,
) -> Result<SourceMap, RegisterLinkError> {
    let mut source_paths = Vec::new();
    let mut source_ids = HashMap::<String, SourceId>::new();
    let mut runs = Vec::new();
    for (final_index, (object_index, function_index)) in function_order.iter().copied().enumerate()
    {
        let object = objects[object_index];
        let function = &object.functions[function_index];
        if code_bases[final_index] > code_starts[final_index] {
            let first = function
                .source_runs
                .first()
                .ok_or(RegisterLinkError::Overflow("initializer source run"))?;
            let source = intern_source(
                object,
                first.source,
                &mut source_paths,
                &mut source_ids,
                strings,
            )?;
            runs.push(SourceRun {
                instruction_start: InstructionAddress::new(code_starts[final_index]),
                source,
                line: first.line,
                column: first.column,
            });
        }
        for run in &function.source_runs {
            let source = intern_source(
                object,
                run.source,
                &mut source_paths,
                &mut source_ids,
                strings,
            )?;
            let instruction_start = code_bases[final_index]
                .checked_add(run.instruction_start)
                .ok_or(RegisterLinkError::Overflow("source instruction address"))?;
            runs.push(SourceRun {
                instruction_start: InstructionAddress::new(instruction_start),
                source,
                line: run.line,
                column: run.column,
            });
        }
    }
    Ok(SourceMap {
        sources: source_paths,
        runs,
    })
}

fn intern_source(
    object: &RelocatableObject,
    local_source: u32,
    source_paths: &mut Vec<fpas_bytecode::StringId>,
    source_ids: &mut HashMap<String, SourceId>,
    strings: &mut StringInterner,
) -> Result<SourceId, RegisterLinkError> {
    let path = object
        .sources
        .get(local_source as usize)
        .ok_or(RegisterLinkError::Overflow("source path reference"))?;
    if let Some(id) = source_ids.get(path) {
        return Ok(*id);
    }
    let id = SourceId::try_from_index(source_paths.len())
        .map_err(|_| RegisterLinkError::Overflow("source IDs"))?;
    source_paths.push(strings.intern(path)?);
    source_ids.insert(path.clone(), id);
    Ok(id)
}
