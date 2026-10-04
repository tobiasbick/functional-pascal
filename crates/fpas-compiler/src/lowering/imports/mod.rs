//! Interface-backed register bindings and verifier-safe imported function stubs.

mod defaults;
mod layouts;
mod stubs;
use layouts::collect_layouts;
use stubs::imported_stub;

use std::collections::{BTreeMap, BTreeSet};

use fpas_ir::{Function, FunctionId, Global, GlobalId};
use fpas_unit::interface::{CallableType, ConstantValue, InterfaceType, SymbolKind, UnitInterface};
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

pub(super) struct BindingTables<'a> {
    pub types: &'a mut TypeTable,
    pub callables: &'a mut BTreeMap<String, Callable>,
    pub globals: &'a mut Vec<Global>,
    pub global_bindings: &'a mut BTreeMap<String, GlobalBinding>,
    pub constants: &'a mut BTreeMap<String, fpas_ir::Constant>,
}

pub(super) fn install(
    interfaces: InterfaceSet<'_>,
    bindings: BindingTables<'_>,
    first_function: u32,
    span: fpas_lexer::Span,
) -> Result<(ImportPlan, Vec<Function>), CompileError> {
    let BindingTables {
        types,
        callables,
        globals,
        global_bindings,
        constants,
    } = bindings;
    let mut symbols = interfaces
        .direct
        .iter()
        .flat_map(|interface| &interface.symbols)
        .collect::<Vec<_>>();
    symbols.sort_by_key(|symbol| symbol.qualified_name.to_ascii_lowercase());
    symbols.dedup_by(|left, right| {
        left.qualified_name
            .eq_ignore_ascii_case(&right.qualified_name)
    });
    let short_constant_counts = symbols
        .iter()
        .filter(|symbol| {
            matches!(
                symbol.kind,
                SymbolKind::Constant(Some(_)) | SymbolKind::EnumMember(_)
            )
        })
        .fold(BTreeMap::<String, usize>::new(), |mut counts, symbol| {
            *counts.entry(symbol.name.to_ascii_lowercase()).or_default() += 1;
            counts
        });
    let mut plan = ImportPlan::default();
    let mut stubs = Vec::new();
    let mut installed_callables = BTreeSet::new();
    for symbol in symbols {
        match &symbol.kind {
            SymbolKind::Function | SymbolKind::Procedure => {
                let callable_type = match &symbol.ty {
                    InterfaceType::Function(callable) | InterfaceType::Procedure(callable) => {
                        callable
                    }
                    _ => return Err(unsupported(span, "imported callable interface type")),
                };
                install_callable(
                    &symbol.qualified_name,
                    Some(&symbol.name),
                    callable_type,
                    types,
                    callables,
                    &mut plan,
                    &mut stubs,
                    &mut installed_callables,
                    first_function,
                    span,
                )?;
            }
            SymbolKind::Variable
            | SymbolKind::MutableVariable
            | SymbolKind::Constant(None)
            | SymbolKind::AggregateConstant(_) => {
                let id = GlobalId::try_from_index(globals.len())
                    .map_err(|_| unsupported(span, "imported global identifier overflow"))?;
                let ty = interface_type_id(types, &symbol.ty, span)?;
                let mutable = symbol.kind == SymbolKind::MutableVariable;
                globals.push(Global {
                    id,
                    name: symbol.qualified_name.to_ascii_lowercase(),
                    ty: if mutable {
                        types.cell_type(ty, span)?
                    } else {
                        ty
                    },
                    mutable,
                    initializer: None,
                });
                let binding = GlobalBinding {
                    id,
                    ty,
                    cell: mutable,
                };
                // Imported storage is reachable only through its canonical unit prefix.
                // Short names can collide with source import aliases and misresolve paths.
                global_bindings.insert(symbol.qualified_name.to_ascii_lowercase(), binding);
                plan.globals.push((
                    id,
                    ObjectImport {
                        name: symbol.qualified_name.to_ascii_lowercase(),
                        shape: ImportShape::Global { mutable },
                    },
                ));
            }
            SymbolKind::Constant(Some(value)) | SymbolKind::EnumMember(value) => {
                let value = lower_constant(value);
                constants.insert(symbol.qualified_name.to_ascii_lowercase(), value.clone());
                if short_constant_counts
                    .get(&symbol.name.to_ascii_lowercase())
                    .copied()
                    == Some(1)
                {
                    constants
                        .entry(symbol.name.to_ascii_lowercase())
                        .or_insert(value);
                }
            }
            SymbolKind::Type | SymbolKind::EnumVariantConstructor => {}
        }
    }
    for interface in interfaces.direct.iter().chain(interfaces.supporting) {
        for symbol in &interface.symbols {
            defaults::install(
                &symbol.ty,
                types,
                callables,
                &mut plan,
                &mut stubs,
                &mut installed_callables,
                first_function,
                span,
            )?;
        }
    }
    collect_layouts(interfaces.supporting, &mut plan);
    Ok((plan, stubs))
}

#[allow(clippy::too_many_arguments)]
fn install_callable(
    qualified_name: &str,
    short_name: Option<&str>,
    callable_type: &CallableType,
    types: &mut TypeTable,
    callables: &mut BTreeMap<String, Callable>,
    plan: &mut ImportPlan,
    stubs: &mut Vec<Function>,
    installed: &mut BTreeSet<String>,
    first_function: u32,
    span: fpas_lexer::Span,
) -> Result<(), CompileError> {
    let qualified_name = qualified_name.to_ascii_lowercase();
    if !installed.insert(qualified_name.clone()) {
        return Ok(());
    }
    if callable_type.variadic {
        return Err(unsupported(span, "variadic imported callable"));
    }
    let parameters = callable_type
        .parameters
        .iter()
        .map(|parameter| {
            let value = interface_type_id(types, &parameter.ty, span)?;
            if parameter.mutable {
                types.reference_type(value, span)
            } else {
                Ok(value)
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    let result = callable_type
        .result
        .as_deref()
        .map(|result| interface_type_id(types, result, span))
        .transpose()?
        .unwrap_or(types::UNIT);
    let function = FunctionId::new(
        first_function
            .checked_add(
                u32::try_from(stubs.len())
                    .map_err(|_| unsupported(span, "imported function identifier overflow"))?,
            )
            .ok_or_else(|| unsupported(span, "imported function identifier overflow"))?,
    );
    let value_type = types.function_type(parameters.clone(), result, span)?;
    let callable = Callable {
        function,
        parameters: parameters.clone(),
        result,
        value_type,
        captures: Vec::new(),
    };
    if let Some(short_name) = short_name {
        callables
            .entry(short_name.to_ascii_lowercase())
            .or_insert_with(|| callable.clone());
    }
    callables.insert(qualified_name.clone(), callable);
    plan.functions.push((
        function,
        ObjectImport {
            name: qualified_name.clone(),
            shape: ImportShape::Function {
                arity: u8::try_from(parameters.len())
                    .map_err(|_| unsupported(span, "imported callable arity overflow"))?,
                capture_count: 0,
                var_parameters: callable_type
                    .parameters
                    .iter()
                    .enumerate()
                    .filter_map(|(index, parameter)| parameter.mutable.then_some(index as u8))
                    .collect(),
                returns_value: callable_type.result.is_some(),
            },
        },
    ));
    stubs.push(imported_stub(
        function,
        &qualified_name,
        parameters,
        result,
        span,
    )?);
    Ok(())
}

fn lower_constant(value: &ConstantValue) -> fpas_ir::Constant {
    match value {
        ConstantValue::Integer(value) => fpas_ir::Constant::Integer(*value),
        ConstantValue::Real(value) => fpas_ir::Constant::Real(f64::from_bits(*value)),
        ConstantValue::Boolean(value) => fpas_ir::Constant::Boolean(*value),
        ConstantValue::String(value) => fpas_ir::Constant::String(value.clone()),
        ConstantValue::EnumValue { backing_value, .. } => {
            fpas_ir::Constant::Integer(*backing_value)
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
