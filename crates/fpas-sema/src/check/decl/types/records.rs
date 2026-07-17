//! Record type checking.
//!
//! **Documentation:** `docs/pascal/language/types/records.md`,
//! `docs/pascal/language/types/record-methods.md`

use super::Checker;
use crate::scope::{FunctionCtx, Symbol, SymbolKind, canonical_symbol_name};
use crate::types::{FunctionTy, MethodKind, ParamTy, ProcedureTy, RecordTy, Ty, TypeConstraint};
use fpas_diagnostics::codes::{SEMA_DUPLICATE_DECLARATION, SEMA_TYPE_MISMATCH};
use fpas_parser::{FuncBody, FunctionDecl, RecordMethod, RecordType, TypeDef, TypeParam};
use std::collections::HashSet;

struct CheckedRecordMembers {
    instance_methods: Vec<(String, MethodKind)>,
    static_functions: Vec<(String, FunctionTy)>,
}

impl Checker {
    pub(super) fn check_record_type_def(&mut self, td: &TypeDef, record: &RecordType) {
        if !self.scopes.define(
            &td.name,
            Symbol {
                ty: Ty::Named(td.name.clone()),
                mutable: false,
                kind: SymbolKind::Type,
                task_bound: false,
            },
        ) {
            self.error_with_code(
                fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION,
                format!("Duplicate type `{}`", td.name),
                "Each name must be unique in the same scope.",
                td.span,
            );
            return;
        }

        let mut seen_fields = HashSet::new();
        let mut field_indexes = Vec::new();
        let mut fields = Vec::new();
        for (field_index, field) in record.fields.iter().enumerate() {
            if !seen_fields.insert(canonical_symbol_name(&field.name)) {
                self.error_with_code(
                    SEMA_DUPLICATE_DECLARATION,
                    format!("Duplicate record field `{}`", field.name),
                    "Each record field name must be unique within the record type.",
                    field.span,
                );
                continue;
            }
            field_indexes.push(field_index);
            fields.push((field.name.clone(), self.resolve_type_expr(&field.type_expr)));
        }

        // Validate default values and build the defaults map entry.
        let defaults_entry: Vec<(String, Option<fpas_parser::Expr>)> = field_indexes
            .iter()
            .map(|field_index| &record.fields[*field_index])
            .zip(fields.iter())
            .map(|(field_def, (_, field_ty))| {
                if let Some(default_expr) = &field_def.default_value {
                    let default_ty = self.check_expr(default_expr);
                    self.check_type_compat(
                        field_ty,
                        &default_ty,
                        &format!("default value for field `{}`", field_def.name),
                        field_def.span,
                    );
                    (field_def.name.clone(), Some(default_expr.clone()))
                } else {
                    (field_def.name.clone(), None)
                }
            })
            .collect();

        // Only register defaults when at least one field has a default, since the
        // compiler uses the absence of an entry to mean "no defaults, emit as-is".
        if defaults_entry.iter().any(|(_, d)| d.is_some()) {
            self.record_defaults.insert(td.name.clone(), defaults_entry);
        }

        let record_ty = RecordTy {
            name: td.name.clone(),
            fields,
            methods: Vec::new(),
            static_functions: Vec::new(),
        };
        let mut ty = Ty::Record(record_ty);

        let members = self.check_record_methods(&td.name, &ty, &record.methods);

        if let Ty::Record(record_ty) = &mut ty {
            record_ty.methods = members.instance_methods;
            record_ty.static_functions = members.static_functions;
        }

        if let Some(existing) = self.scopes.lookup_mut(&td.name) {
            *existing.ty_mut() = ty;
        }
    }

    fn check_record_methods(
        &mut self,
        type_name: &str,
        record_ty: &Ty,
        methods: &[RecordMethod],
    ) -> CheckedRecordMembers {
        let mut checked_methods = Vec::new();
        let mut checked_static = Vec::new();
        let mut seen_methods = HashSet::new();

        for method in methods {
            match method {
                RecordMethod::Function(function) => {
                    if !self.register_record_member_name(
                        type_name,
                        &function.name,
                        function.span,
                        &mut seen_methods,
                    ) {
                        continue;
                    }
                    if let Some(entry) =
                        self.check_instance_function(type_name, record_ty, function)
                    {
                        checked_methods.push(entry);
                    }
                }
                RecordMethod::StaticFunction(function) => {
                    if !self.register_record_member_name(
                        type_name,
                        &function.name,
                        function.span,
                        &mut seen_methods,
                    ) {
                        continue;
                    }
                    if let Some(entry) = self.check_static_function(type_name, record_ty, function)
                    {
                        checked_static.push(entry);
                    }
                }
                RecordMethod::Procedure(procedure) => {
                    if !self.register_record_member_name(
                        type_name,
                        &procedure.name,
                        procedure.span,
                        &mut seen_methods,
                    ) {
                        continue;
                    }
                    if let Some(entry) =
                        self.check_instance_procedure(type_name, record_ty, procedure)
                    {
                        checked_methods.push(entry);
                    }
                }
            }
        }

        CheckedRecordMembers {
            instance_methods: checked_methods,
            static_functions: checked_static,
        }
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
            format!("Duplicate record method `{type_name}.{name}`"),
            "Each record method name must be unique within the record type (static and instance names share one set).",
            span,
        );
        false
    }

    fn check_instance_function(
        &mut self,
        type_name: &str,
        record_ty: &Ty,
        function: &FunctionDecl,
    ) -> Option<(String, MethodKind)> {
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

        self.check_method_body(
            &qualified,
            &function.type_params,
            &params,
            Some(return_ty),
            &function.body,
        );
        Some((function.name.clone(), MethodKind::Function(function_ty)))
    }

    fn check_static_function(
        &mut self,
        type_name: &str,
        record_ty: &Ty,
        function: &FunctionDecl,
    ) -> Option<(String, FunctionTy)> {
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

        self.check_method_body(
            &qualified,
            &function.type_params,
            &params,
            Some(return_ty),
            &function.body,
        );
        Some((function.name.clone(), function_ty))
    }

    fn check_instance_procedure(
        &mut self,
        type_name: &str,
        record_ty: &Ty,
        procedure: &fpas_parser::ProcedureDecl,
    ) -> Option<(String, MethodKind)> {
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

        self.check_method_body(
            &qualified,
            &procedure.type_params,
            &params,
            None,
            &procedure.body,
        );
        Some((procedure.name.clone(), MethodKind::Procedure(procedure_ty)))
    }

    fn validate_record_method_signature(
        &mut self,
        type_name: &str,
        method_name: &str,
        params: &[ParamTy],
        span: fpas_lexer::Span,
    ) -> bool {
        let Some(self_param) = params.first() else {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "Record method `{type_name}.{method_name}` must declare `Self: {type_name}` as its first parameter"
                ),
                format!(
                    "Use `{method_name}(Self: {type_name}; ...)` so calls like `Value.{method_name}(...)` can pass the receiver implicitly."
                ),
                span,
            );
            return false;
        };

        if !self_param.name.eq_ignore_ascii_case("Self")
            || !matches!(&self_param.ty, Ty::Record(record) if record.name.eq_ignore_ascii_case(type_name))
        {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "Record method `{type_name}.{method_name}` must declare `Self: {type_name}` as its first parameter"
                ),
                format!(
                    "Use `{method_name}(Self: {type_name}; ...)` so calls like `Value.{method_name}(...)` can pass the receiver implicitly."
                ),
                span,
            );
            return false;
        }

        true
    }

    fn validate_static_function_signature(
        &mut self,
        type_name: &str,
        method_name: &str,
        params: &[ParamTy],
        span: fpas_lexer::Span,
    ) -> bool {
        if params
            .iter()
            .any(|param| param.name.eq_ignore_ascii_case("Self"))
        {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "Static record function `{type_name}.{method_name}` must not declare a `Self` parameter"
                ),
                format!(
                    "Remove `Self` and call the function through the type: `{type_name}.{method_name}(...)`."
                ),
                span,
            );
            return false;
        }
        true
    }

    fn check_method_body(
        &mut self,
        qualified_name: &str,
        type_params: &[TypeParam],
        params: &[ParamTy],
        return_type: Option<Ty>,
        body: &FuncBody,
    ) {
        let FuncBody::Block { nested, stmts } = body;

        self.scopes.push_scope();

        // Introduce method-level generic type parameters as `GenericParam` types
        // so that expressions in the body can reference them.
        for tp in type_params {
            let constraint = tp
                .constraint
                .as_ref()
                .and_then(|c| TypeConstraint::from_name(c));
            self.scopes.define(
                &tp.name,
                Symbol {
                    ty: Ty::GenericParam(tp.name.clone(), constraint),
                    mutable: false,
                    kind: SymbolKind::Type,
                    task_bound: false,
                },
            );
        }

        for param in params {
            self.scopes.define(
                &param.name,
                Symbol {
                    ty: param.ty.clone(),
                    mutable: param.mutable,
                    kind: SymbolKind::Param,
                    task_bound: false,
                },
            );
        }
        let previous_ctx = self.scopes.function_ctx.take();
        self.scopes.function_ctx = Some(FunctionCtx {
            name: qualified_name.to_string(),
            return_type,
        });
        for decl in nested {
            self.check_decl(decl);
        }
        for stmt in stmts {
            self.check_stmt(stmt);
        }
        self.scopes.function_ctx = previous_ctx;
        self.scopes.pop_scope();
    }

    /// Resolve a method parameter type, treating the type name as the record type (for `Self`).
    pub(super) fn resolve_method_param_type(
        &mut self,
        type_expr: &fpas_parser::TypeExpr,
        type_name: &str,
        record_ty: &Ty,
    ) -> Ty {
        let resolved = self.resolve_type_expr(type_expr);
        if let Ty::Named(name) = &resolved
            && name.eq_ignore_ascii_case(type_name)
        {
            return record_ty.clone();
        }
        resolved
    }
}
