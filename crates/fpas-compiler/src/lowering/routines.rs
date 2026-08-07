//! Deterministic named-routine discovery and scalar signature lowering.

use std::collections::BTreeMap;

use fpas_ir::{Function, FunctionId, TypeId};
use fpas_parser::{Decl, FormalParam, FuncBody, FunctionDecl, ProcedureDecl};
use fpas_sema::AnalysisMetadata;

use crate::CompileError;

use super::context::{Callable, FunctionInput, LoweringContext, unsupported};
use super::types;

pub(super) enum Routine<'a> {
    Function(&'a FunctionDecl),
    Procedure(&'a ProcedureDecl),
}

impl Routine<'_> {
    fn name(&self) -> &str {
        match self {
            Self::Function(function) => &function.name,
            Self::Procedure(procedure) => &procedure.name,
        }
    }

    fn params(&self) -> &[FormalParam] {
        match self {
            Self::Function(function) => &function.params,
            Self::Procedure(procedure) => &procedure.params,
        }
    }

    fn body(&self) -> &FuncBody {
        match self {
            Self::Function(function) => &function.body,
            Self::Procedure(procedure) => &procedure.body,
        }
    }

    pub fn statements(&self) -> &[fpas_parser::Stmt] {
        let FuncBody::Block { stmts, .. } = self.body();
        stmts
    }

    fn result(&self, types: &mut types::TypeTable) -> Result<TypeId, CompileError> {
        match self {
            Self::Function(function) => types.type_expr(&function.return_type),
            Self::Procedure(_) => Ok(types::UNIT),
        }
    }

    fn span(&self) -> fpas_lexer::Span {
        match self {
            Self::Function(function) => function.span,
            Self::Procedure(procedure) => procedure.span,
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
            Decl::Const(_) | Decl::Var(_) | Decl::MutableVar(_) | Decl::TypeDef(_) => {}
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
            .map(|parameter| types.type_expr(&parameter.type_expr))
            .collect::<Result<Vec<_>, _>>()?;
        let result = routine.result(types)?;
        let value_type = types.function_type(parameters, result, routine.span())?;
        let captures = metadata
            .nested_routine_captures
            .get(&routine.name().to_ascii_lowercase())
            .map(|info| {
                info.captures
                    .iter()
                    .map(|capture| {
                        let ty = super::closures::find_capture_type(
                            routine.body(),
                            &capture.name,
                            metadata,
                        )
                        .ok_or_else(|| unsupported(routine.span(), "untyped nested capture"))?;
                        let ty = types.intern(&ty, routine.span().line, routine.span().column)?;
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
            routine.name().to_ascii_lowercase(),
            Callable {
                function,
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
    id: FunctionId,
    metadata: &AnalysisMetadata,
    callables: &BTreeMap<String, Callable>,
    types: &mut types::TypeTable,
    closure_targets: std::collections::HashMap<usize, super::context::ClosureTarget>,
    cell_names: std::collections::BTreeSet<String>,
) -> Result<Function, CompileError> {
    let captures = callables
        .get(&routine.name().to_ascii_lowercase())
        .map(|callable| callable.captures.clone())
        .unwrap_or_default();
    let parameters = routine
        .params()
        .iter()
        .map(|parameter| {
            types
                .type_expr(&parameter.type_expr)
                .map(|ty| (parameter.name.clone(), ty))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut context = LoweringContext::new(FunctionInput {
        name: routine.name(),
        id,
        result: routine.result(types)?,
        parameters: &parameters,
        captures: &captures,
        metadata,
        callables: callables.clone(),
        closure_targets,
        cell_names,
        type_table: types.clone(),
    })?;
    let FuncBody::Block { stmts, .. } = routine.body();
    context.lower_statements(stmts)?;
    context.finish(routine.span())
}
