//! Source path merging and sparse source-run rebasing.

use std::collections::HashMap;

use fpas_bytecode::{
    DebugBinding, DebugBindingKind, DebugCaptureKind, DebugCaptureSource, DebugScope,
    DebugSourceLocation, FunctionDebugInfo, FunctionId, InstructionAddress, Register,
    SequencePoint, SourceId, SourceMap, SourceRun,
};
use fpas_unit::object::{ObjectDebugBindingKind, ObjectDebugLocation, RelocatableObject};

use crate::LinkError;
use crate::plan::DebugTypeIds;

use super::strings::StringInterner;

pub(super) fn merge(
    objects: &[&RelocatableObject],
    function_order: &[(usize, usize)],
    function_maps: &[Vec<Option<FunctionId>>],
    code_starts: &[u32],
    code_bases: &[u32],
    debug_types: &DebugTypeIds,
    strings: &mut StringInterner,
) -> Result<(SourceMap, Vec<FunctionDebugInfo>), LinkError> {
    let mut source_paths = Vec::new();
    let mut source_ids = HashMap::<String, SourceId>::new();
    let mut runs = Vec::new();
    let mut function_debug = Vec::with_capacity(function_order.len());
    for (final_index, (object_index, function_index)) in function_order.iter().copied().enumerate()
    {
        let object = objects[object_index];
        let function = &object.functions[function_index];
        if code_bases[final_index] > code_starts[final_index] {
            let first = function
                .source_runs
                .first()
                .ok_or(LinkError::Overflow("initializer source run"))?;
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
                .ok_or(LinkError::Overflow("source instruction address"))?;
            runs.push(SourceRun {
                instruction_start: InstructionAddress::new(instruction_start),
                source,
                line: run.line,
                column: run.column,
            });
        }
        function_debug.push(merge_debug(
            object,
            function,
            object_index,
            function_maps,
            code_bases[final_index],
            &mut source_paths,
            &mut source_ids,
            debug_types,
            strings,
        )?);
    }
    Ok((
        SourceMap {
            sources: source_paths,
            runs,
        },
        function_debug,
    ))
}

#[expect(
    clippy::too_many_arguments,
    reason = "debug merge needs source, string, and type translation tables"
)]
fn merge_debug(
    object: &RelocatableObject,
    function: &fpas_unit::object::ObjectFunction,
    object_index: usize,
    function_maps: &[Vec<Option<FunctionId>>],
    code_base: u32,
    source_paths: &mut Vec<fpas_bytecode::StringId>,
    source_ids: &mut HashMap<String, SourceId>,
    debug_types: &DebugTypeIds,
    strings: &mut StringInterner,
) -> Result<FunctionDebugInfo, LinkError> {
    let scopes = function
        .debug
        .scopes
        .iter()
        .map(|scope| DebugScope {
            id: scope.id,
            parent: scope.parent,
        })
        .collect();
    let bindings = function
        .debug
        .bindings
        .iter()
        .map(|binding| {
            Ok(DebugBinding {
                name: strings.intern(&binding.name)?,
                type_name: strings.intern(&binding.type_name)?,
                ty: debug_types.translate(object_index, binding.ty)?,
                register: Register::new(binding.register)
                    .map_err(|_| LinkError::Overflow("debug binding register"))?,
                kind: match binding.kind {
                    ObjectDebugBindingKind::Parameter => DebugBindingKind::Parameter,
                    ObjectDebugBindingKind::Local => DebugBindingKind::Local,
                    ObjectDebugBindingKind::Capture => DebugBindingKind::Capture,
                },
                mutable: binding.mutable,
                scope: binding.scope,
                declaration: binding
                    .declaration
                    .map(|location| {
                        merge_location(object, location, source_paths, source_ids, strings)
                    })
                    .transpose()?,
                hidden: binding.hidden,
                cell_backed: binding.cell_backed,
                initializer: binding
                    .initializer_start
                    .map(|instruction| {
                        code_base
                            .checked_add(instruction)
                            .map(InstructionAddress::new)
                            .ok_or(LinkError::Overflow("debug binding initializer address"))
                    })
                    .transpose()?,
            })
        })
        .collect::<Result<Vec<_>, LinkError>>()?;
    let sequence_points = function
        .debug
        .sequence_points
        .iter()
        .map(|point| {
            Ok(SequencePoint {
                instruction: InstructionAddress::new(
                    code_base
                        .checked_add(point.instruction_start)
                        .ok_or(LinkError::Overflow("debug sequence point address"))?,
                ),
                location: merge_location(
                    object,
                    point.location,
                    source_paths,
                    source_ids,
                    strings,
                )?,
                scope: point.scope,
            })
        })
        .collect::<Result<Vec<_>, LinkError>>()?;
    Ok(FunctionDebugInfo {
        scopes,
        bindings,
        sequence_points,
        result_type: function
            .debug
            .result_type
            .map(|ty| debug_types.translate(object_index, ty))
            .transpose()?,
        lexical_owner: function
            .debug
            .lexical_owner
            .map(|local| {
                function_maps
                    .get(object_index)
                    .and_then(|map| map.get(local as usize).copied().flatten())
                    .ok_or(LinkError::Overflow("lexical owner function ID"))
            })
            .transpose()?,
        capture_sources: function
            .debug
            .capture_sources
            .iter()
            .map(|source| {
                Ok(DebugCaptureSource {
                    binding: fpas_bytecode::DebugBindingId::new(source.binding),
                    ty: debug_types.translate(object_index, source.ty)?,
                    kind: match source.kind {
                        fpas_unit::object::ObjectCaptureKind::Value => DebugCaptureKind::Value,
                        fpas_unit::object::ObjectCaptureKind::Cell => DebugCaptureKind::Cell,
                        fpas_unit::object::ObjectCaptureKind::EnclosingCell => {
                            DebugCaptureKind::EnclosingCell
                        }
                    },
                })
            })
            .collect::<Result<Vec<_>, LinkError>>()?,
    })
}

fn merge_location(
    object: &RelocatableObject,
    location: ObjectDebugLocation,
    source_paths: &mut Vec<fpas_bytecode::StringId>,
    source_ids: &mut HashMap<String, SourceId>,
    strings: &mut StringInterner,
) -> Result<DebugSourceLocation, LinkError> {
    Ok(DebugSourceLocation {
        source: intern_source(object, location.source, source_paths, source_ids, strings)?,
        line: location.line,
        column: location.column,
    })
}

fn intern_source(
    object: &RelocatableObject,
    local_source: u32,
    source_paths: &mut Vec<fpas_bytecode::StringId>,
    source_ids: &mut HashMap<String, SourceId>,
    strings: &mut StringInterner,
) -> Result<SourceId, LinkError> {
    let path = object
        .sources
        .get(local_source as usize)
        .ok_or(LinkError::Overflow("source path reference"))?;
    if let Some(id) = source_ids.get(path) {
        return Ok(*id);
    }
    let id = SourceId::try_from_index(source_paths.len())
        .map_err(|_| LinkError::Overflow("source IDs"))?;
    source_paths.push(strings.intern(path)?);
    source_ids.insert(path.clone(), id);
    Ok(id)
}
