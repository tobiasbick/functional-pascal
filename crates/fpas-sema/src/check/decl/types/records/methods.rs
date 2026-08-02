//! Record method collection and routine registration.

use super::Checker;
use crate::scope::{Symbol, SymbolKind, canonical_symbol_name};
use crate::types::{FunctionTy, MethodKind, ParamTy, ProcedureTy, Ty};
use fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION;
use fpas_parser::{FuncBody, FunctionDecl, RecordMethod, TypeParam};
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
            match method {
                RecordMethod::Function(function) => {
                    if !self.register_record_member_name(
                        type_name,
                        &function.name,
                        function.span,
                        seen_members,
                    ) {
                        continue;
                    }
                    if let Some((entry, pending)) =
                        self.check_instance_function(type_name, record_ty, function)
                    {
                        checked_methods.push(entry);
                        pending_bodies.push(pending);
                    }
                }
                RecordMethod::StaticFunction(function) => {
                    if !self.register_record_member_name(
                        type_name,
                        &function.name,
                        function.span,
                        seen_members,
                    ) {
                        continue;
                    }
                    if let Some((entry, pending)) =
                        self.check_static_function(type_name, record_ty, function)
                    {
                        checked_static.push(entry);
                        pending_bodies.push(pending);
                    }
                }
                RecordMethod::StaticProcedure(procedure) => {
                    if !self.register_record_member_name(
                        type_name,
                        &procedure.name,
                        procedure.span,
                        seen_members,
                    ) {
                        continue;
                    }
                    if let Some((entry, pending)) =
                        self.check_static_procedure(type_name, record_ty, procedure)
                    {
                        checked_static_procedures.push(entry);
                        pending_bodies.push(pending);
                    }
                }
                RecordMethod::Procedure(procedure) => {
                    if !self.register_record_member_name(
                        type_name,
                        &procedure.name,
                        procedure.span,
                        seen_members,
                    ) {
                        continue;
                    }
                    if let Some((entry, pending)) =
                        self.check_instance_procedure(type_name, record_ty, procedure)
                    {
                        checked_methods.push(entry);
                        pending_bodies.push(pending);
                    }
                }
            }
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

    fn register_record_member_name(
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

    fn check_instance_function<'a>(
        &mut self,
        type_name: &str,
        record_ty: &Ty,
        function: &'a FunctionDecl,
    ) -> Option<((String, MethodKind), PendingMethodBody<'a>)> {
        self.check_unique_formal_param_names(&function.params);

        let type_param_defs = Self::resolve_type_params(&function.type_params);

        let (return_ty, params) =
            self.with_type_params(&function.type_params, function.span, |checker| {
                let return_ty =
                    checker.resolve_method_param_type(&function.return_type, type_name, record_ty);
                let params: Vec<ParamTy> = function
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

        if !self.validate_record_method_signature(type_name, &function.name, &params, function.span)
        {
            return None;
        }

        let function_ty = FunctionTy {
            type_params: type_param_defs,
            params: params.clone(),
            return_type: Box::new(return_ty.clone()),
            variadic: false,
        };

        let qualified = format!("{type_name}.{}", function.name);
        self.scopes.define(
            &qualified,
            Symbol {
                ty: Ty::Function(function_ty.clone()),
                mutable: false,
                kind: SymbolKind::Function,
                task_bound: false,
            },
        );

        Some((
            (function.name.clone(), MethodKind::Function(function_ty)),
            PendingMethodBody {
                qualified_name: qualified,
                type_params: &function.type_params,
                params,
                return_type: Some(return_ty),
                body: &function.body,
            },
        ))
    }

    fn check_static_function<'a>(
        &mut self,
        type_name: &str,
        record_ty: &Ty,
        function: &'a FunctionDecl,
    ) -> Option<((String, FunctionTy), PendingMethodBody<'a>)> {
        self.check_unique_formal_param_names(&function.params);

        let type_param_defs = Self::resolve_type_params(&function.type_params);

        let (return_ty, params) =
            self.with_type_params(&function.type_params, function.span, |checker| {
                let return_ty =
                    checker.resolve_method_param_type(&function.return_type, type_name, record_ty);
                let params: Vec<ParamTy> = function
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

        if !self.validate_static_function_signature(
            type_name,
            &function.name,
            &params,
            function.span,
        ) {
            return None;
        }

        let function_ty = FunctionTy {
            type_params: type_param_defs,
            params: params.clone(),
            return_type: Box::new(return_ty.clone()),
            variadic: false,
        };

        let qualified = format!("{type_name}.{}", function.name);
        self.scopes.define(
            &qualified,
            Symbol {
                ty: Ty::Function(function_ty.clone()),
                mutable: false,
                kind: SymbolKind::Function,
                task_bound: false,
            },
        );

        Some((
            (function.name.clone(), function_ty),
            PendingMethodBody {
                qualified_name: qualified,
                type_params: &function.type_params,
                params,
                return_type: Some(return_ty),
                body: &function.body,
            },
        ))
    }

    fn check_instance_procedure<'a>(
        &mut self,
        type_name: &str,
        record_ty: &Ty,
        procedure: &'a fpas_parser::ProcedureDecl,
    ) -> Option<((String, MethodKind), PendingMethodBody<'a>)> {
        self.check_unique_formal_param_names(&procedure.params);

        let type_param_defs = Self::resolve_type_params(&procedure.type_params);

        let params = self.with_type_params(&procedure.type_params, procedure.span, |checker| {
            procedure
                .params
                .iter()
                .map(|param| ParamTy {
                    mutable: param.mutable,
                    name: param.name.clone(),
                    ty: checker.resolve_method_param_type(&param.type_expr, type_name, record_ty),
                })
                .collect::<Vec<_>>()
        });

        if !self.validate_record_method_signature(
            type_name,
            &procedure.name,
            &params,
            procedure.span,
        ) {
            return None;
        }

        let procedure_ty = ProcedureTy {
            type_params: type_param_defs,
            variadic: false,
            params: params.clone(),
        };

        let qualified = format!("{type_name}.{}", procedure.name);
        self.scopes.define(
            &qualified,
            Symbol {
                ty: Ty::Procedure(procedure_ty.clone()),
                mutable: false,
                kind: SymbolKind::Procedure,
                task_bound: false,
            },
        );

        Some((
            (procedure.name.clone(), MethodKind::Procedure(procedure_ty)),
            PendingMethodBody {
                qualified_name: qualified,
                type_params: &procedure.type_params,
                params,
                return_type: None,
                body: &procedure.body,
            },
        ))
    }

    fn check_static_procedure<'a>(
        &mut self,
        type_name: &str,
        record_ty: &Ty,
        procedure: &'a fpas_parser::ProcedureDecl,
    ) -> Option<((String, ProcedureTy), PendingMethodBody<'a>)> {
        self.check_unique_formal_param_names(&procedure.params);

        let type_param_defs = Self::resolve_type_params(&procedure.type_params);
        let params = self.with_type_params(&procedure.type_params, procedure.span, |checker| {
            procedure
                .params
                .iter()
                .map(|param| ParamTy {
                    mutable: param.mutable,
                    name: param.name.clone(),
                    ty: checker.resolve_method_param_type(&param.type_expr, type_name, record_ty),
                })
                .collect::<Vec<_>>()
        });

        if !self.validate_static_routine_signature(
            type_name,
            &procedure.name,
            &params,
            procedure.span,
            "procedure",
        ) {
            return None;
        }

        let procedure_ty = ProcedureTy {
            type_params: type_param_defs,
            variadic: false,
            params: params.clone(),
        };
        let qualified = format!("{type_name}.{}", procedure.name);
        self.scopes.define(
            &qualified,
            Symbol {
                ty: Ty::Procedure(procedure_ty.clone()),
                mutable: false,
                kind: SymbolKind::Procedure,
                task_bound: false,
            },
        );

        Some((
            (procedure.name.clone(), procedure_ty),
            PendingMethodBody {
                qualified_name: qualified,
                type_params: &procedure.type_params,
                params,
                return_type: None,
                body: &procedure.body,
            },
        ))
    }
}
