//! Debug binding register assignment and portable type display names.

use fpas_bytecode::{
    DebugBinding, DebugBindingKind, DebugScope, DebugSourceLocation, DebugType, DebugTypeId,
    EnumTypeId, FunctionDebugInfo, InstructionAddress, RecordTypeId, SequencePoint, SourceId,
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
                ty: debug_binding_type(program, binding.ty, binding.cell_backed),
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
                initializer: binding
                    .initializer
                    .map(|initializer| {
                        point_addresses
                            .iter()
                            .find(|(block, instruction, _)| {
                                *block == initializer.block
                                    && *instruction == initializer.instruction
                            })
                            .map(|(_, _, address)| *address)
                            .ok_or_else(|| {
                                super::compile_error(
                                    "debug binding initializer has no emitted store instruction",
                                )
                            })
                    })
                    .transpose()?,
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
    let lexical_owner = function
        .debug
        .lexical_owner
        .map(|owner| {
            u16::try_from(owner.get())
                .map(fpas_bytecode::FunctionId::new)
                .map_err(|_| super::compile_error("lexical owner exceeds bytecode function id"))
        })
        .transpose()?;
    Ok(FunctionDebugInfo {
        scopes,
        bindings,
        sequence_points,
        result_type: Some(DebugTypeId::new(function.signature.result.get())),
        lexical_owner,
        capture_sources: function
            .debug
            .capture_sources
            .iter()
            .map(|source| fpas_bytecode::DebugCaptureSource {
                binding: fpas_bytecode::DebugBindingId::new(source.binding.get()),
                ty: DebugTypeId::new(source.ty.get()),
                kind: match source.kind {
                    fpas_ir::CaptureKind::Value => fpas_bytecode::DebugCaptureKind::Value,
                    fpas_ir::CaptureKind::Cell => fpas_bytecode::DebugCaptureKind::Cell,
                    fpas_ir::CaptureKind::EnclosingCell => {
                        fpas_bytecode::DebugCaptureKind::EnclosingCell
                    }
                },
            })
            .collect(),
    })
}

pub(super) fn compile_debug_types(program: &Program) -> Result<Vec<DebugType>, CompileError> {
    program
        .types
        .iter()
        .enumerate()
        .map(|(index, definition)| {
            if usize::try_from(definition.id.get()).ok() != Some(index) {
                return Err(super::compile_error(
                    "debug type identifiers must be dense and ordered",
                ));
            }
            lower_type(&definition.kind)
        })
        .collect()
}

fn lower_type(ty: &IrType) -> Result<DebugType, CompileError> {
    let id = |ty: TypeId| DebugTypeId::new(ty.get());
    Ok(match ty {
        IrType::Unit => DebugType::Unit,
        IrType::Boolean => DebugType::Boolean,
        IrType::Integer => DebugType::Integer,
        IrType::Real => DebugType::Real,
        IrType::String => DebugType::String,
        IrType::Dynamic => DebugType::Dynamic,
        IrType::Array(element) => DebugType::Array(id(*element)),
        IrType::Dictionary { key, value } => DebugType::Dictionary {
            key: id(*key),
            value: id(*value),
        },
        IrType::Result { ok, error } => DebugType::Result {
            ok: id(*ok),
            error: id(*error),
        },
        IrType::Option(inner) => DebugType::Option(id(*inner)),
        IrType::Function { parameters, result } => DebugType::Function {
            parameters: parameters.iter().copied().map(id).collect(),
            result: id(*result),
        },
        IrType::Record(layout) => DebugType::Record(RecordTypeId::new(
            u16::try_from(layout.get())
                .map_err(|_| super::compile_error("record debug type exceeds u16"))?,
        )),
        IrType::Enum(layout) => DebugType::Enum(EnumTypeId::new(
            u16::try_from(layout.get())
                .map_err(|_| super::compile_error("enum debug type exceeds u16"))?,
        )),
        IrType::Cell(inner) => DebugType::Cell(id(*inner)),
        IrType::Task(inner) => DebugType::Task(id(*inner)),
        IrType::Channel(inner) => DebugType::Channel(id(*inner)),
    })
}

fn debug_binding_type(program: &Program, ty: TypeId, cell_backed: bool) -> DebugTypeId {
    if cell_backed
        && let Some(fpas_ir::TypeDefinition {
            kind: IrType::Cell(inner),
            ..
        }) = program.ty(ty)
    {
        return DebugTypeId::new(inner.get());
    }
    DebugTypeId::new(ty.get())
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
        IrType::Channel(inner) => format!("channel of {}", type_name(program, *inner, depth + 1)),
    }
}
