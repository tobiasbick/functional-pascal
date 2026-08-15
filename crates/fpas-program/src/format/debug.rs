//! Bounded binary conversion for function debugger metadata.

use fpas_bytecode::{
    DebugBinding, DebugBindingId, DebugBindingKind, DebugCaptureKind, DebugCaptureSource,
    DebugScope, DebugSourceLocation, DebugTypeId, FunctionDebugInfo, FunctionId,
    InstructionAddress, Register, SequencePoint, SourceId, StringId,
};

use super::sections::{SectionReader, write_u8, write_u16, write_u32};
use super::{FormatError, check_limit, checked_u32};

const NO_PARENT: u32 = u32::MAX;
const BINDING_PARAMETER: u8 = 0;
const BINDING_LOCAL: u8 = 1;
const BINDING_CAPTURE: u8 = 2;
const CAPTURE_VALUE: u8 = 0;
const CAPTURE_CELL: u8 = 1;
const CAPTURE_ENCLOSING_CELL: u8 = 2;

#[derive(Default)]
pub(super) struct DebugCounts {
    scopes: usize,
    bindings: usize,
    sequence_points: usize,
    capture_sources: usize,
}

pub(super) fn encode(output: &mut Vec<u8>, debug: &FunctionDebugInfo) -> Result<(), FormatError> {
    check_limit(
        "debug_scopes",
        debug.scopes.len(),
        fpas_bytecode::limits::MAX_DEBUG_SCOPES,
    )?;
    write_u32(
        output,
        checked_u32("debug_scope_count", debug.scopes.len())?,
    );
    for scope in &debug.scopes {
        write_u32(output, scope.id);
        write_u32(output, scope.parent.unwrap_or(NO_PARENT));
    }

    check_limit(
        "debug_bindings",
        debug.bindings.len(),
        fpas_bytecode::limits::MAX_DEBUG_BINDINGS,
    )?;
    write_u32(
        output,
        checked_u32("debug_binding_count", debug.bindings.len())?,
    );
    for binding in &debug.bindings {
        write_u32(output, binding.name.get());
        write_u32(output, binding.type_name.get());
        write_u32(output, binding.ty.get());
        write_u16(output, binding.register.get());
        write_u8(
            output,
            match binding.kind {
                DebugBindingKind::Parameter => BINDING_PARAMETER,
                DebugBindingKind::Local => BINDING_LOCAL,
                DebugBindingKind::Capture => BINDING_CAPTURE,
            },
        );
        write_bool(output, binding.mutable);
        write_u32(output, binding.scope);
        write_bool(output, binding.declaration.is_some());
        if let Some(location) = binding.declaration {
            encode_location(output, location);
        }
        write_bool(output, binding.hidden);
        write_bool(output, binding.cell_backed);
    }

    check_limit(
        "debug_sequence_points",
        debug.sequence_points.len(),
        fpas_bytecode::limits::MAX_DEBUG_SEQUENCE_POINTS,
    )?;
    write_u32(
        output,
        checked_u32("debug_sequence_point_count", debug.sequence_points.len())?,
    );
    for point in &debug.sequence_points {
        write_u32(output, point.instruction.get());
        encode_location(output, point.location);
        write_u32(output, point.scope);
    }
    write_bool(output, debug.result_type.is_some());
    if let Some(ty) = debug.result_type {
        write_u32(output, ty.get());
    }
    write_bool(output, debug.lexical_owner.is_some());
    if let Some(owner) = debug.lexical_owner {
        write_u16(output, owner.get());
    }
    check_limit(
        "debug_capture_sources",
        debug.capture_sources.len(),
        fpas_bytecode::limits::MAX_CLOSURE_CAPTURES,
    )?;
    write_u32(
        output,
        checked_u32("debug_capture_source_count", debug.capture_sources.len())?,
    );
    for source in &debug.capture_sources {
        write_u32(output, source.binding.get());
        write_u32(output, source.ty.get());
        write_u8(
            output,
            match source.kind {
                DebugCaptureKind::Value => CAPTURE_VALUE,
                DebugCaptureKind::Cell => CAPTURE_CELL,
                DebugCaptureKind::EnclosingCell => CAPTURE_ENCLOSING_CELL,
            },
        );
    }
    Ok(())
}

pub(super) fn decode(
    reader: &mut SectionReader<'_>,
    totals: &mut DebugCounts,
) -> Result<FunctionDebugInfo, FormatError> {
    let scope_count = read_count(
        reader,
        "debug_scope_count",
        &mut totals.scopes,
        fpas_bytecode::limits::MAX_DEBUG_SCOPES,
    )?;
    let mut scopes = Vec::with_capacity(scope_count);
    for _ in 0..scope_count {
        let id = reader.u32("debug_scope_id")?;
        let parent = match reader.u32("debug_scope_parent")? {
            NO_PARENT => None,
            value => Some(value),
        };
        scopes.push(DebugScope { id, parent });
    }

    let binding_count = read_count(
        reader,
        "debug_binding_count",
        &mut totals.bindings,
        fpas_bytecode::limits::MAX_DEBUG_BINDINGS,
    )?;
    let mut bindings = Vec::with_capacity(binding_count);
    for _ in 0..binding_count {
        let name = StringId::new(reader.u32("debug_binding_name")?);
        let type_name = StringId::new(reader.u32("debug_binding_type_name")?);
        let ty = DebugTypeId::new(reader.u32("debug_binding_type")?);
        let register_value = reader.u16("debug_binding_register")?;
        let register = Register::new(register_value).map_err(|_| FormatError::InvalidValue {
            field: "debug_binding_register",
            value: u64::from(register_value),
        })?;
        let kind = match reader.u8("debug_binding_kind")? {
            BINDING_PARAMETER => DebugBindingKind::Parameter,
            BINDING_LOCAL => DebugBindingKind::Local,
            BINDING_CAPTURE => DebugBindingKind::Capture,
            value => {
                return Err(FormatError::InvalidValue {
                    field: "debug_binding_kind",
                    value: u64::from(value),
                });
            }
        };
        let mutable = read_bool(reader, "debug_binding_mutable")?;
        let scope = reader.u32("debug_binding_scope")?;
        let declaration = if read_bool(reader, "debug_binding_has_declaration")? {
            Some(decode_location(reader)?)
        } else {
            None
        };
        let hidden = read_bool(reader, "debug_binding_hidden")?;
        let cell_backed = read_bool(reader, "debug_binding_cell_backed")?;
        bindings.push(DebugBinding {
            name,
            type_name,
            ty,
            register,
            kind,
            mutable,
            scope,
            declaration,
            hidden,
            cell_backed,
        });
    }

    let point_count = read_count(
        reader,
        "debug_sequence_point_count",
        &mut totals.sequence_points,
        fpas_bytecode::limits::MAX_DEBUG_SEQUENCE_POINTS,
    )?;
    let mut sequence_points = Vec::with_capacity(point_count);
    for _ in 0..point_count {
        sequence_points.push(SequencePoint {
            instruction: InstructionAddress::new(reader.u32("debug_sequence_instruction")?),
            location: decode_location(reader)?,
            scope: reader.u32("debug_sequence_scope")?,
        });
    }

    let result_type = if read_bool(reader, "debug_has_result_type")? {
        Some(DebugTypeId::new(reader.u32("debug_result_type")?))
    } else {
        None
    };
    let lexical_owner = if read_bool(reader, "debug_has_lexical_owner")? {
        Some(FunctionId::new(reader.u16("debug_lexical_owner")?))
    } else {
        None
    };
    let source_count = read_count(
        reader,
        "debug_capture_source_count",
        &mut totals.capture_sources,
        fpas_bytecode::limits::MAX_CLOSURE_CAPTURES,
    )?;
    let mut capture_sources = Vec::with_capacity(source_count);
    for _ in 0..source_count {
        let binding = DebugBindingId::new(reader.u32("debug_capture_binding")?);
        let ty = DebugTypeId::new(reader.u32("debug_capture_type")?);
        let kind = match reader.u8("debug_capture_kind")? {
            CAPTURE_VALUE => DebugCaptureKind::Value,
            CAPTURE_CELL => DebugCaptureKind::Cell,
            CAPTURE_ENCLOSING_CELL => DebugCaptureKind::EnclosingCell,
            value => {
                return Err(FormatError::InvalidValue {
                    field: "debug_capture_kind",
                    value: u64::from(value),
                });
            }
        };
        capture_sources.push(DebugCaptureSource { binding, ty, kind });
    }

    Ok(FunctionDebugInfo {
        scopes,
        bindings,
        sequence_points,
        result_type,
        lexical_owner,
        capture_sources,
    })
}

fn encode_location(output: &mut Vec<u8>, location: DebugSourceLocation) {
    write_u32(output, location.source.get());
    write_u32(output, location.line);
    write_u32(output, location.column);
}

fn decode_location(reader: &mut SectionReader<'_>) -> Result<DebugSourceLocation, FormatError> {
    Ok(DebugSourceLocation {
        source: SourceId::new(reader.u32("debug_location_source")?),
        line: reader.u32("debug_location_line")?,
        column: reader.u32("debug_location_column")?,
    })
}

fn write_bool(output: &mut Vec<u8>, value: bool) {
    write_u8(output, u8::from(value));
}

fn read_bool(reader: &mut SectionReader<'_>, field: &'static str) -> Result<bool, FormatError> {
    match reader.u8(field)? {
        0 => Ok(false),
        1 => Ok(true),
        value => Err(FormatError::InvalidValue {
            field,
            value: u64::from(value),
        }),
    }
}

fn read_count(
    reader: &mut SectionReader<'_>,
    field: &'static str,
    total: &mut usize,
    maximum: usize,
) -> Result<usize, FormatError> {
    let count = reader.u32(field)? as usize;
    *total = total.checked_add(count).ok_or(FormatError::LimitExceeded {
        field,
        size: usize::MAX,
        maximum,
    })?;
    check_limit(field, *total, maximum)?;
    Ok(count)
}
