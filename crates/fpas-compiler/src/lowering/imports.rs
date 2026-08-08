//! Interface-backed register bindings and verifier-safe imported function stubs.

use std::collections::BTreeMap;

use fpas_ir::{
    BasicBlock, BlockId, Function, FunctionId, FunctionSignature, Global, GlobalId, Instruction,
    Operation, Terminator, ValueDefinition, ValueId,
};
use fpas_unit::interface::{InterfaceType, SymbolKind, UnitInterface};
use fpas_unit::object::{ImportShape, ObjectImport};

use crate::CompileError;

use super::context::{Callable, GlobalBinding, unsupported};
use super::types::{self, TypeTable};

#[derive(Default)]
pub(crate) struct ImportPlan {
    pub functions: Vec<(FunctionId, ObjectImport)>,
    pub globals: Vec<(GlobalId, ObjectImport)>,
    pub layouts: Vec<ObjectImport>,
}

pub(super) struct InterfaceSet<'a> {
    pub direct: &'a [UnitInterface],
    pub supporting: &'a [UnitInterface],
}

pub(super) fn install(
    interfaces: InterfaceSet<'_>,
    types: &mut TypeTable,
    callables: &mut BTreeMap<String, Callable>,
    globals: &mut Vec<Global>,
    global_bindings: &mut BTreeMap<String, GlobalBinding>,
    first_function: u32,
    span: fpas_lexer::Span,
) -> Result<(ImportPlan, Vec<Function>), CompileError> {
    let mut symbols = interfaces
        .direct
        .iter()
        .flat_map(|interface| &interface.symbols)
        .collect::<Vec<_>>();
    symbols.sort_by_key(|symbol| symbol.qualified_name.to_ascii_lowercase());
    let mut plan = ImportPlan::default();
    let mut stubs = Vec::new();
    for symbol in symbols {
        match symbol.kind {
            SymbolKind::Function | SymbolKind::Procedure => {
                let callable_type = match &symbol.ty {
                    InterfaceType::Function(callable) | InterfaceType::Procedure(callable) => {
                        callable
                    }
                    _ => return Err(unsupported(span, "imported callable interface type")),
                };
                if !callable_type.type_parameters.is_empty() || callable_type.variadic {
                    return Err(unsupported(span, "generic or variadic imported callable"));
                }
                let parameters = callable_type
                    .parameters
                    .iter()
                    .map(|parameter| interface_type_id(types, &parameter.ty, span))
                    .collect::<Result<Vec<_>, _>>()?;
                let result = callable_type
                    .result
                    .as_deref()
                    .map(|result| interface_type_id(types, result, span))
                    .transpose()?
                    .unwrap_or(types::UNIT);
                let function = FunctionId::new(
                    first_function
                        .checked_add(u32::try_from(stubs.len()).map_err(|_| {
                            unsupported(span, "imported function identifier overflow")
                        })?)
                        .ok_or_else(|| {
                            unsupported(span, "imported function identifier overflow")
                        })?,
                );
                let value_type = types.function_type(parameters.clone(), result, span)?;
                let callable = Callable {
                    function,
                    parameters: parameters.clone(),
                    result,
                    value_type,
                    captures: Vec::new(),
                };
                callables.insert(symbol.name.to_ascii_lowercase(), callable.clone());
                callables.insert(symbol.qualified_name.to_ascii_lowercase(), callable);
                let returns_value = callable_type.result.is_some();
                plan.functions.push((
                    function,
                    ObjectImport {
                        name: symbol.qualified_name.to_ascii_lowercase(),
                        shape: ImportShape::Function {
                            arity: u8::try_from(parameters.len()).map_err(|_| {
                                unsupported(span, "imported callable arity overflow")
                            })?,
                            capture_count: 0,
                            returns_value,
                        },
                    },
                ));
                stubs.push(imported_stub(
                    function,
                    &symbol.qualified_name,
                    parameters,
                    result,
                    span,
                )?);
            }
            SymbolKind::Variable | SymbolKind::MutableVariable => {
                let id = GlobalId::try_from_index(globals.len())
                    .map_err(|_| unsupported(span, "imported global identifier overflow"))?;
                let ty = interface_type_id(types, &symbol.ty, span)?;
                let mutable = symbol.kind == SymbolKind::MutableVariable;
                globals.push(Global {
                    id,
                    name: symbol.qualified_name.to_ascii_lowercase(),
                    ty,
                    mutable,
                });
                let binding = GlobalBinding { id, ty };
                global_bindings.insert(symbol.name.to_ascii_lowercase(), binding);
                global_bindings.insert(symbol.qualified_name.to_ascii_lowercase(), binding);
                plan.globals.push((
                    id,
                    ObjectImport {
                        name: symbol.qualified_name.to_ascii_lowercase(),
                        shape: ImportShape::Global { mutable },
                    },
                ));
            }
            SymbolKind::Type => {}
            SymbolKind::Constant(_)
            | SymbolKind::EnumMember(_)
            | SymbolKind::EnumVariantConstructor => {}
        }
    }
    collect_layouts(interfaces.supporting, &mut plan);
    Ok((plan, stubs))
}

fn collect_layouts(interfaces: &[UnitInterface], plan: &mut ImportPlan) {
    for symbol in interfaces
        .iter()
        .flat_map(|interface| &interface.symbols)
        .filter(|symbol| symbol.kind == SymbolKind::Type)
    {
        let shape = match &symbol.ty {
            InterfaceType::Record(record) => Some(ImportShape::Record {
                fields: record
                    .fields
                    .iter()
                    .map(|field| field.name.to_ascii_lowercase())
                    .collect(),
            }),
            InterfaceType::Enum(enumeration)
                if enumeration
                    .variants
                    .iter()
                    .any(|variant| !variant.fields.is_empty()) =>
            {
                Some(ImportShape::Enum {
                    variants: enumeration
                        .variants
                        .iter()
                        .map(|variant| {
                            (
                                variant.name.to_ascii_lowercase(),
                                variant
                                    .fields
                                    .iter()
                                    .map(|field| field.name.to_ascii_lowercase())
                                    .collect(),
                            )
                        })
                        .collect(),
                })
            }
            InterfaceType::Enum(_) | InterfaceType::Named(_) => None,
            _ => None,
        };
        if let Some(shape) = shape {
            plan.layouts.push(ObjectImport {
                name: symbol.qualified_name.to_ascii_lowercase(),
                shape,
            });
        }
    }
}

fn interface_type_id(
    types: &mut TypeTable,
    interface: &InterfaceType,
    span: fpas_lexer::Span,
) -> Result<fpas_ir::TypeId, CompileError> {
    let ty = fpas_sema::interface_type_to_ty(interface)
        .map_err(|_| unsupported(span, "imported interface type conversion"))?;
    types.intern(&ty, span.line, span.column)
}

fn imported_stub(
    id: FunctionId,
    name: &str,
    parameter_types: Vec<fpas_ir::TypeId>,
    result: fpas_ir::TypeId,
    span: fpas_lexer::Span,
) -> Result<Function, CompileError> {
    let parameters = parameter_types
        .iter()
        .copied()
        .enumerate()
        .map(|(index, ty)| {
            ValueId::try_from_index(index)
                .map(|id| ValueDefinition { id, ty })
                .map_err(|_| unsupported(span, "import parameter overflow"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let panic_id = ValueId::try_from_index(parameters.len())
        .map_err(|_| unsupported(span, "import stub value overflow"))?;
    Ok(Function {
        id,
        name: name.to_ascii_lowercase(),
        signature: FunctionSignature {
            parameters: parameter_types,
            result,
        },
        parameters,
        locals: Vec::new(),
        captures: Vec::new(),
        blocks: vec![BasicBlock {
            id: BlockId::new(0),
            parameters: Vec::new(),
            instructions: vec![Instruction {
                source: None,
                result: Some(ValueDefinition {
                    id: panic_id,
                    ty: types::STRING,
                }),
                operation: Operation::Const(fpas_ir::Constant::String(
                    "unlinked imported callable".to_string(),
                )),
            }],
            terminators: vec![Terminator::Panic(panic_id)],
        }],
        entry: BlockId::new(0),
        max_call_arguments: 0,
        can_spawn_tasks: false,
    })
}
