//! Debug binding register assignment and portable type display names.

use fpas_bytecode::{
    DebugBinding, DebugBindingKind, DebugScope, DebugSourceLocation, FunctionDebugInfo,
    InstructionAddress, SequencePoint, SourceId,
};
use fpas_ir::{BlockId, Function, IrType, Program, TypeId};

use crate::CompileError;

use super::allocation::Allocation;
use super::metadata::MetadataBuilder;

pub(super) fn compile_debug_info(
    program: &Program,
    function: &Function,
    allocation: &Allocation,
    point_addresses: &[(BlockId, usize, InstructionAddress)],
    metadata: &mut MetadataBuilder,
) -> Result<FunctionDebugInfo, CompileError> {
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
                name: metadata.intern_string(&binding.name)?,
                type_name: metadata.intern_string(&type_name(program, binding.ty, 0))?,
                register: allocation.local(binding.local)?,
                kind: match binding.kind {
                    fpas_ir::DebugBindingKind::Parameter => DebugBindingKind::Parameter,
                    fpas_ir::DebugBindingKind::Local => DebugBindingKind::Local,
                    fpas_ir::DebugBindingKind::Capture => DebugBindingKind::Capture,
                },
                mutable: binding.mutable,
                scope: binding.scope,
                declaration: binding.declaration.map(location),
                hidden: binding.hidden,
                cell_backed: binding.cell_backed,
            })
        })
        .collect::<Result<Vec<_>, CompileError>>()?;
    let sequence_points = point_addresses
        .iter()
        .filter_map(|(block, instruction, address)| {
            function
                .debug
                .sequence_points
                .iter()
                .find(|point| point.block == *block && point.instruction == *instruction)
                .map(|point| SequencePoint {
                    instruction: *address,
                    location: location(point.source),
                    scope: point.scope,
                })
        })
        .collect();
    Ok(FunctionDebugInfo {
        scopes,
        bindings,
        sequence_points,
    })
}

fn location(span: fpas_ir::SourceSpan) -> DebugSourceLocation {
    DebugSourceLocation {
        source: SourceId::new(0),
        line: span.line(),
        column: span.column(),
    }
}

fn type_name(program: &Program, ty: TypeId, depth: usize) -> String {
    if depth >= 16 {
        return "dynamic".to_string();
    }
    let Some(definition) = program.ty(ty) else {
        return "dynamic".to_string();
    };
    match &definition.kind {
        IrType::Unit => "unit".to_string(),
        IrType::Boolean => "boolean".to_string(),
        IrType::Integer => "integer".to_string(),
        IrType::Real => "real".to_string(),
        IrType::String => "string".to_string(),
        IrType::Dynamic => "dynamic".to_string(),
        IrType::Array(element) => {
            format!("array of {}", type_name(program, *element, depth + 1))
        }
        IrType::Dictionary { key, value } => format!(
            "dictionary of {}, {}",
            type_name(program, *key, depth + 1),
            type_name(program, *value, depth + 1)
        ),
        IrType::Result { ok, error } => format!(
            "result of {}, {}",
            type_name(program, *ok, depth + 1),
            type_name(program, *error, depth + 1)
        ),
        IrType::Option(inner) => format!("option of {}", type_name(program, *inner, depth + 1)),
        IrType::Function { parameters, result } => {
            let parameters = parameters
                .iter()
                .map(|parameter| type_name(program, *parameter, depth + 1))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "function({parameters}): {}",
                type_name(program, *result, depth + 1)
            )
        }
        IrType::Record(layout) => program
            .record_layout(*layout)
            .map_or_else(|| "record".to_string(), |layout| layout.name.clone()),
        IrType::Enum(layout) => program
            .enum_layout(*layout)
            .map_or_else(|| "enum".to_string(), |layout| layout.name.clone()),
        IrType::Cell(inner) => type_name(program, *inner, depth + 1),
        IrType::Task(inner) => format!("task of {}", type_name(program, *inner, depth + 1)),
    }
}
