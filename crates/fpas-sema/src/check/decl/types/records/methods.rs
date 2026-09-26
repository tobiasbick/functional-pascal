//! Record method collection and routine registration.

use super::Checker;
use crate::scope::{Symbol, SymbolKind, canonical_symbol_name};
use crate::types::{FunctionTy, MethodKind, ParamTy, ProcedureTy, Ty};
use fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION;
use fpas_parser::{
    FormalParam, FuncBody, FunctionDecl, ProcedureDecl, RecordMethod, TypeExpr, TypeParam,
};
use std::collections::HashSet;

/// Checked callable types grouped by their record dispatch kind.
pub(super) struct CheckedRecordMembers {
    /// Instance methods callable through record values.
    pub(super) instance_methods: Vec<(String, MethodKind)>,
    /// Static functions callable through the record type.
    pub(super) static_functions: Vec<(String, FunctionTy)>,
    /// Static procedures callable through the record type.
    pub(super) static_procedures: Vec<(String, ProcedureTy)>,
}

/// Method body deferred until every record member is visible.
pub(super) struct PendingMethodBody<'a> {
    /// Fully qualified method name used for scope and diagnostic context.
    pub(super) qualified_name: String,
    /// Method-level generic type parameters.
    pub(super) type_params: &'a [TypeParam],
    /// Resolved formal parameters, including an instance receiver when present.
    pub(super) params: Vec<ParamTy>,
    /// Exact source declarations corresponding to `params`.
    pub(super) param_spans: Vec<fpas_lexer::Span>,
    /// Resolved function result, or `None` for a procedure.
    pub(super) return_type: Option<Ty>,
    /// Parsed routine body checked after record registration completes.
    pub(super) body: &'a FuncBody,
}

impl Checker {
    /// Register record routines and collect their bodies for deferred checking.
    pub(super) fn check_record_methods<'a>(
        &mut self,
        type_name: &str,
        record_ty: &Ty,
        methods: &'a [RecordMethod],
        seen_members: &mut HashSet<String>,
    ) -> (CheckedRecordMembers, Vec<PendingMethodBody<'a>>) {
        let mut checked_methods = Vec::new();
        let mut checked_static = Vec::new();
        let mut checked_static_procedures = Vec::new();
        let mut pending_bodies = Vec::new();

        for method in methods {
            let (routine, dispatch) = match method {
                RecordMethod::Function(function) => {
                    (RecordRoutine::function(function), RoutineDispatch::Instance)
                }
                RecordMethod::StaticFunction(function) => {
                    (RecordRoutine::function(function), RoutineDispatch::Static)
                }
                RecordMethod::StaticProcedure(procedure) => {
                    (RecordRoutine::procedure(procedure), RoutineDispatch::Static)
                }
                RecordMethod::Procedure(procedure) => (
                    RecordRoutine::procedure(procedure),
                    RoutineDispatch::Instance,
                ),
            };
            if !self.register_record_member_name(
                type_name,
                routine.name,
                routine.span,
                seen_members,
            ) {
                continue;
            }
            let Some((kind, pending)) =
                self.check_record_routine(type_name, record_ty, dispatch, &routine)
            else {
                continue;
            };
            let name = routine.name.to_owned();
            match (dispatch, kind) {
                (RoutineDispatch::Instance, kind) => checked_methods.push((name, kind)),
                (RoutineDispatch::Static, MethodKind::Function(function_ty)) => {
                    checked_static.push((name, function_ty));
                }
                (RoutineDispatch::Static, MethodKind::Procedure(procedure_ty)) => {
                    checked_static_procedures.push((name, procedure_ty));
                }
            }
            pending_bodies.push(pending);
        }

        (
            CheckedRecordMembers {
                instance_methods: checked_methods,
                static_functions: checked_static,
                static_procedures: checked_static_procedures,
            },
            pending_bodies,
        )
    }

    /// Record a member name, reporting a duplicate declaration when already seen.
    pub(in crate::check::decl::types) fn register_record_member_name(
        &mut self,
        type_name: &str,
        name: &str,
        span: fpas_lexer::Span,
        seen: &mut HashSet<String>,
    ) -> bool {
        if seen.insert(canonical_symbol_name(name)) {
            return true;
        }
        self.error_with_code(
            SEMA_DUPLICATE_DECLARATION,
            format!("Duplicate record member `{type_name}.{name}`"),
            "Each field, method, static routine, property, and event name must be unique within the record type.",
            span,
        );
        false
    }

    /// Resolve, validate, and register one record routine.
    ///
    /// Returns its callable type and the body to check once all members are visible.
    fn check_record_routine<'a>(
        &mut self,
        type_name: &str,
        record_ty: &Ty,
        dispatch: RoutineDispatch,
        routine: &RecordRoutine<'a>,
    ) -> Option<(MethodKind, PendingMethodBody<'a>)> {
        self.check_unique_formal_param_names(routine.params);

        let type_param_defs = Self::resolve_type_params(routine.type_params);

        let (return_ty, params) =
            self.with_type_params(routine.type_params, routine.span, |checker| {
                let return_ty = routine.return_type.map(|return_type| {
                    checker.resolve_method_param_type(return_type, type_name, record_ty)
                });
                let params: Vec<ParamTy> = routine
                    .params
                    .iter()
                    .map(|param| ParamTy {
                        mutable: param.mutable,
                        name: param.name.clone(),
                        ty: checker.resolve_method_param_type(
                            &param.type_expr,
                            type_name,
                            record_ty,
                        ),
                    })
                    .collect();
                (return_ty, params)
            });

        let valid = match dispatch {
            RoutineDispatch::Instance => self.validate_record_method_signature(
                type_name,
                routine.name,
                &params,
                routine.span,
            ),
            RoutineDispatch::Static => self.validate_static_routine_signature(
                type_name,
                routine.name,
                &params,
                routine.span,
                if return_ty.is_some() {
                    "function"
                } else {
                    "procedure"
                },
            ),
        };
        if !valid {
            return None;
        }

        let (kind, ty, symbol_kind) = match &return_ty {
            Some(return_ty) => {
                let function_ty = FunctionTy {
                    type_params: type_param_defs,
                    params: params.clone(),
                    return_type: Box::new(return_ty.clone()),
                    variadic: false,
                };
                (
                    MethodKind::Function(function_ty.clone()),
                    Ty::Function(function_ty),
                    SymbolKind::Function,
                )
            }
            None => {
                let procedure_ty = ProcedureTy {
                    type_params: type_param_defs,
                    variadic: false,
                    params: params.clone(),
                };
                (
                    MethodKind::Procedure(procedure_ty.clone()),
                    Ty::Procedure(procedure_ty),
                    SymbolKind::Procedure,
                )
            }
        };

        let qualified = format!("{type_name}.{}", routine.name);
        self.scopes.define(
            &qualified,
            Symbol {
                ty,
                mutable: false,
                kind: symbol_kind,
                task_bound: false,
            },
        );

        Some((
            kind,
            PendingMethodBody {
                qualified_name: qualified,
                type_params: routine.type_params,
                params,
                param_spans: routine.params.iter().map(|param| param.span).collect(),
                return_type: return_ty,
                body: routine.body,
            },
        ))
    }
}

/// Whether a record routine is called through values or through the type.
#[derive(Clone, Copy)]
enum RoutineDispatch {
    Instance,
    Static,
}

/// Declaration parts shared by record functions and procedures.
struct RecordRoutine<'a> {
    name: &'a str,
    span: fpas_lexer::Span,
    type_params: &'a [TypeParam],
    params: &'a [FormalParam],
    /// Declared result type, or `None` for a procedure.
    return_type: Option<&'a TypeExpr>,
    body: &'a FuncBody,
}

impl<'a> RecordRoutine<'a> {
    fn function(function: &'a FunctionDecl) -> Self {
        Self {
            name: &function.name,
            span: function.span,
            type_params: &function.type_params,
            params: &function.params,
            return_type: Some(&function.return_type),
            body: &function.body,
        }
    }

    fn procedure(procedure: &'a ProcedureDecl) -> Self {
        Self {
            name: &procedure.name,
            span: procedure.span,
            type_params: &procedure.type_params,
            params: &procedure.params,
            return_type: None,
            body: &procedure.body,
        }
    }
}
