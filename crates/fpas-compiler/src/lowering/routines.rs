//! Deterministic named-routine discovery and scalar signature lowering.

use std::collections::BTreeMap;

use fpas_ir::{Function, FunctionId, TypeId};
use fpas_parser::{
    Decl, FormalParam, FuncBody, FunctionDecl, ProcedureDecl, RecordMethod, TypeBody,
};
use fpas_sema::AnalysisMetadata;

use crate::CompileError;

use super::context::{Callable, FunctionInput, LoweringContext, unsupported};
use super::types;

pub(super) enum Routine<'a> {
    Function(&'a FunctionDecl),
    Procedure(&'a ProcedureDecl),
    RecordFunction(&'a str, &'a FunctionDecl),
    RecordProcedure(&'a str, &'a ProcedureDecl),
}

pub(super) struct LoweringInput<'a> {
    pub id: FunctionId,
    pub metadata: &'a AnalysisMetadata,
    pub callables: &'a BTreeMap<String, Callable>,
    pub globals: &'a BTreeMap<String, super::context::GlobalBinding>,
    pub constants: &'a BTreeMap<String, fpas_ir::Constant>,
    pub closure_targets: std::collections::HashMap<usize, super::context::ClosureTarget>,
    pub bound_method_targets: std::collections::HashMap<usize, super::context::BoundMethodTarget>,
    pub cell_names: std::collections::BTreeSet<String>,
}

impl Routine<'_> {
    fn name(&self) -> &str {
        match self {
            Self::Function(function) | Self::RecordFunction(_, function) => &function.name,
            Self::Procedure(procedure) | Self::RecordProcedure(_, procedure) => &procedure.name,
        }
    }

    fn params(&self) -> &[FormalParam] {
        match self {
            Self::Function(function) | Self::RecordFunction(_, function) => &function.params,
            Self::Procedure(procedure) | Self::RecordProcedure(_, procedure) => &procedure.params,
        }
    }

    fn type_params(&self) -> &[fpas_parser::TypeParam] {
        match self {
            Self::Function(function) | Self::RecordFunction(_, function) => &function.type_params,
            Self::Procedure(procedure) | Self::RecordProcedure(_, procedure) => {
                &procedure.type_params
            }
        }
    }

    fn body(&self) -> &FuncBody {
        match self {
            Self::Function(function) | Self::RecordFunction(_, function) => &function.body,
            Self::Procedure(procedure) | Self::RecordProcedure(_, procedure) => &procedure.body,
        }
    }

    pub fn statements(&self) -> &[fpas_parser::Stmt] {
        let FuncBody::Block { stmts, .. } = self.body();
        stmts
    }

    fn result(&self, types: &mut types::TypeTable) -> Result<TypeId, CompileError> {
        match self {
            Self::Function(function) | Self::RecordFunction(_, function) => {
                types.type_expr_with_params(&function.return_type, &function.type_params)
            }
            Self::Procedure(_) | Self::RecordProcedure(_, _) => Ok(types::UNIT),
        }
    }

    fn span(&self) -> fpas_lexer::Span {
        match self {
            Self::Function(function) | Self::RecordFunction(_, function) => function.span,
            Self::Procedure(procedure) | Self::RecordProcedure(_, procedure) => procedure.span,
        }
    }

    fn runtime_name(&self) -> String {
        match self {
            Self::RecordFunction(owner, function) => format!("{owner}.{}", function.name),
            Self::RecordProcedure(owner, procedure) => format!("{owner}.{}", procedure.name),
            _ => self.name().to_string(),
        }
    }
}

pub(super) fn collect<'a>(declarations: &'a [Decl], routines: &mut Vec<Routine<'a>>) {
    for declaration in declarations {
        match declaration {
            Decl::Function(function) => {
                routines.push(Routine::Function(function));
                let FuncBody::Block { nested, .. } = &function.body;
                collect(nested, routines);
            }
            Decl::Procedure(procedure) => {
                routines.push(Routine::Procedure(procedure));
                let FuncBody::Block { nested, .. } = &procedure.body;
                collect(nested, routines);
            }
            Decl::TypeDef(definition) => {
                if let TypeBody::Record(record) = &definition.body {
                    for method in &record.methods {
                        match method {
                            RecordMethod::Function(function)
                            | RecordMethod::StaticFunction(function) => {
                                routines.push(Routine::RecordFunction(&definition.name, function))
                            }
                            RecordMethod::Procedure(procedure)
                            | RecordMethod::StaticProcedure(procedure) => {
                                routines.push(Routine::RecordProcedure(&definition.name, procedure))
                            }
                        }
                    }
                }
            }
            Decl::Const(_) | Decl::Var(_) | Decl::MutableVar(_) => {}
        }
    }
}

pub(super) fn callable_table(
    routines: &[Routine<'_>],
    types: &mut types::TypeTable,
    metadata: &AnalysisMetadata,
) -> Result<BTreeMap<String, Callable>, CompileError> {
    let mut table = BTreeMap::new();
    for (index, routine) in routines.iter().enumerate() {
        let function = FunctionId::new(
            u32::try_from(index.saturating_add(1))
                .map_err(|_| unsupported(routine.span(), "function identifier overflow"))?,
        );
        let parameters = routine
            .params()
            .iter()
            .map(|parameter| {
                types.type_expr_with_params(&parameter.type_expr, routine.type_params())
            })
            .collect::<Result<Vec<_>, _>>()?;
        let result = routine.result(types)?;
        let value_type = types.function_type(parameters.clone(), result, routine.span())?;
        let captures = metadata
            .nested_routine_captures
            .get(&routine.name().to_ascii_lowercase())
            .map(|info| {
                info.captures
                    .iter()
                    .map(|capture| {
                        let ty = types.intern(
                            &capture.ty,
                            routine.span().line,
                            routine.span().column,
                        )?;
                        let kind = if capture.mutable {
                            fpas_ir::CaptureKind::Cell
                        } else {
                            fpas_ir::CaptureKind::Value
                        };
                        let storage_ty = if capture.mutable {
                            types.cell_type(ty, routine.span())?
                        } else {
                            ty
                        };
                        Ok(super::context::CaptureInput {
                            name: capture.name.clone(),
                            ty,
                            storage_ty,
                            kind,
                        })
                    })
                    .collect::<Result<Vec<_>, CompileError>>()
            })
            .transpose()?
            .unwrap_or_default();
        table.insert(
            routine.runtime_name().to_ascii_lowercase(),
            Callable {
                function,
                parameters,
                result,
                value_type,
                captures,
            },
        );
    }
    Ok(table)
}

pub(super) fn lower(
    routine: &Routine<'_>,
    types: &mut types::TypeTable,
    input: LoweringInput<'_>,
) -> Result<(Function, types::TypeTable), CompileError> {
    let LoweringInput {
        id,
        metadata,
        callables,
        globals,
        constants,
        closure_targets,
        bound_method_targets,
        cell_names,
    } = input;
    let runtime_name = routine.runtime_name();
    let captures = callables
        .get(&runtime_name.to_ascii_lowercase())
        .map(|callable| callable.captures.clone())
        .unwrap_or_default();
    let parameters = routine
        .params()
        .iter()
        .map(|parameter| {
            types
                .type_expr_with_params(&parameter.type_expr, routine.type_params())
                .map(|ty| (parameter.name.clone(), ty))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut context = LoweringContext::new(FunctionInput {
        name: &runtime_name,
        id,
        result: routine.result(types)?,
        parameters: &parameters,
        captures: &captures,
        globals: globals.clone(),
        constants: constants.clone(),
        metadata,
        callables: callables.clone(),
        closure_targets,
        bound_method_targets,
        cell_names,
        type_table: types.clone(),
    })?;
    let FuncBody::Block { stmts, .. } = routine.body();
    context.lower_statements(stmts)?;
    context.finish(routine.span())
}
