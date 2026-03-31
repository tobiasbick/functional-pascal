//! Record type checking.
//!
//! **Documentation:** `docs/pascal/05-types.md`

use super::Checker;
use crate::scope::{FunctionCtx, Symbol, SymbolKind};
use crate::types::{
    FunctionTy, InterfaceTy, MethodKind, ParamTy, ProcedureTy, RecordTy, Ty, TypeConstraint,
};
use fpas_diagnostics::codes::{SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_TYPE};
use fpas_parser::{FuncBody, RecordMethod, RecordType, TypeDef, TypeParam};

impl Checker {
    pub(super) fn check_record_type_def(&mut self, td: &TypeDef, record: &RecordType) {
        if !self.scopes.define(
            &td.name,
            Symbol {
                ty: Ty::Named(td.name.clone()),
                mutable: false,
                kind: SymbolKind::Type,
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
        self.pending_record_types.insert(td.name.clone());

        let fields = self.with_type_params(&td.type_params, td.span, |checker| {
            record
                .fields
                .iter()
                .map(|field| {
                    (
                        field.name.clone(),
                        checker.resolve_type_expr(&field.type_expr),
                    )
                })
                .collect::<Vec<_>>()
        });

        // Validate default values and build the defaults map entry.
        // We check defaults outside `with_type_params` because defaults are evaluated
        // in the outer scope (they cannot reference the type's own generic params).
        let defaults_entry: Vec<(String, Option<fpas_parser::Expr>)> = record
            .fields
            .iter()
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
            type_params: Self::resolve_type_params(&td.type_params),
            fields,
            methods: Vec::new(),
            implements: Vec::new(),
        };
        let mut ty = Ty::Record(record_ty);

        let methods = self.with_type_params(&td.type_params, td.span, |checker| {
            checker.check_record_methods(&td.name, &ty, &record.methods)
        });

        // Resolve and validate `implements` clauses.
        let implements: Vec<String> = record
            .implements
            .iter()
            .filter_map(|te| {
                let resolved = self.resolve_type_expr(te);
                match resolved {
                    Ty::Interface(iface) => Some(iface.name.clone()),
                    _ => {
                        self.error_with_code(
                            SEMA_UNKNOWN_TYPE,
                            format!("`{}` is not an interface", td.name),
                            "Only interface names can appear in an `implements` clause.",
                            td.span,
                        );
                        None
                    }
                }
            })
            .collect();

        if let Ty::Record(record_ty) = &mut ty {
            record_ty.methods = methods.clone();
            record_ty.implements = implements.clone();
        }

        // Validate that the record actually implements every declared interface.
        let iface_types: Vec<InterfaceTy> = implements
            .iter()
            .filter_map(|name| {
                self.scopes.lookup(name).and_then(|sym| {
                    if let Ty::Interface(iface) = &sym.ty {
                        Some(iface.clone())
                    } else {
                        None
                    }
                })
            })
            .collect();
        for iface in &iface_types {
            self.validate_implements(&td.name, &methods, iface, td.span);
        }

        if let Some(existing) = self.scopes.lookup_mut(&td.name) {
            *existing.ty_mut() = ty;
        }
        self.pending_record_types.remove(&td.name);
    }

    fn check_record_methods(
        &mut self,
        type_name: &str,
        record_ty: &Ty,
        methods: &[RecordMethod],
    ) -> Vec<(String, MethodKind)> {
        let mut checked_methods = Vec::new();

        for method in methods {
            match method {
                RecordMethod::Function(function) => {
                    let type_param_defs = Self::resolve_type_params(&function.type_params);

                    // Resolve param/return types with the method's own type params in scope
                    // so expressions like `Value: T` resolve `T` as a generic param.
                    let (return_ty, params) =
                        self.with_type_params(&function.type_params, function.span, |checker| {
                            let return_ty = checker.resolve_method_param_type(
                                &function.return_type,
                                type_name,
                                record_ty,
                            );
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

                    if !self.validate_record_method_signature(
                        type_name,
                        &function.name,
                        &params,
                        function.span,
                    ) {
                        continue;
                    }

                    let function_ty = FunctionTy {
                        type_params: type_param_defs,
                        params: params.clone(),
                        return_type: Box::new(return_ty.clone()),
                    };

                    let qualified = format!("{type_name}.{}", function.name);
                    self.scopes.define(
                        &qualified,
                        Symbol {
                            ty: Ty::Function(function_ty.clone()),
                            mutable: false,
                            kind: SymbolKind::Function,
                        },
                    );

                    self.check_method_body(
                        &qualified,
                        &function.type_params,
                        &params,
                        Some(return_ty),
                        &function.body,
                    );
                    checked_methods
                        .push((function.name.clone(), MethodKind::Function(function_ty)));
                }
                RecordMethod::Procedure(procedure) => {
                    let type_param_defs = Self::resolve_type_params(&procedure.type_params);

                    let params =
                        self.with_type_params(&procedure.type_params, procedure.span, |checker| {
                            procedure
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
                                .collect::<Vec<_>>()
                        });

                    if !self.validate_record_method_signature(
                        type_name,
                        &procedure.name,
                        &params,
                        procedure.span,
                    ) {
                        continue;
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
                        },
                    );

                    self.check_method_body(
                        &qualified,
                        &procedure.type_params,
                        &params,
                        None,
                        &procedure.body,
                    );
                    checked_methods
                        .push((procedure.name.clone(), MethodKind::Procedure(procedure_ty)));
                }
            }
        }

        checked_methods
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

    fn check_method_body(
        &mut self,
        qualified_name: &str,
        type_params: &[TypeParam],
        params: &[ParamTy],
        return_type: Option<Ty>,
        body: &FuncBody,
    ) {
        let FuncBody::Block { nested, stmts } = body else {
            return;
        };

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
        self.report_missing_forward_declarations_in_current_scope();
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
            && name == type_name
        {
            return record_ty.clone();
        }
        resolved
    }

    /// Verify that `record_name` provides an implementation for every method in `iface`.
    ///
    /// A method is considered implemented when the record has a method with the same name
    /// whose parameter types (after substituting the record type for `Self`) and return type
    /// match those declared in the interface.
    fn validate_implements(
        &mut self,
        record_name: &str,
        record_methods: &[(String, MethodKind)],
        iface: &InterfaceTy,
        span: fpas_lexer::Span,
    ) {
        for (method_name, iface_kind) in &iface.methods {
            let found = record_methods
                .iter()
                .find(|(n, _)| n.eq_ignore_ascii_case(method_name));

            let Some((_, record_kind)) = found else {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!(
                        "Record `{record_name}` does not implement `{}` required by interface `{}`",
                        method_name, iface.name
                    ),
                    format!(
                        "Add `function {method_name}(Self: {record_name}; ...)` (or the matching `procedure`) to `{record_name}`."
                    ),
                    span,
                );
                continue;
            };

            // Compare signatures: skip Self param (index 0) for arity/type checking.
            match (iface_kind, record_kind) {
                (MethodKind::Function(if_fn), MethodKind::Function(rec_fn)) => {
                    self.check_method_sig_compat(
                        record_name,
                        method_name,
                        &iface.name,
                        if_fn.params.get(1..).unwrap_or(&[]),
                        Some(&if_fn.return_type),
                        rec_fn.params.get(1..).unwrap_or(&[]),
                        Some(&rec_fn.return_type),
                        span,
                    );
                }
                (MethodKind::Procedure(if_pr), MethodKind::Procedure(rec_pr)) => {
                    self.check_method_sig_compat(
                        record_name,
                        method_name,
                        &iface.name,
                        if_pr.params.get(1..).unwrap_or(&[]),
                        None,
                        rec_pr.params.get(1..).unwrap_or(&[]),
                        None,
                        span,
                    );
                }
                _ => {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!(
                            "Method `{record_name}.{method_name}` kind (function/procedure) does not match interface `{}`",
                            iface.name
                        ),
                        "Use the same kind (function or procedure) as declared in the interface.",
                        span,
                    );
                }
            }
        }
    }

    /// Check that the visible parameter lists and return type of a record method match
    /// the corresponding interface declaration (Self param excluded on both sides).
    #[allow(clippy::too_many_arguments)]
    fn check_method_sig_compat(
        &mut self,
        record_name: &str,
        method_name: &str,
        iface_name: &str,
        iface_params: &[ParamTy],
        iface_ret: Option<&Ty>,
        rec_params: &[ParamTy],
        rec_ret: Option<&Ty>,
        span: fpas_lexer::Span,
    ) {
        if iface_params.len() != rec_params.len() {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "`{record_name}.{method_name}` has {} parameter(s) but interface `{iface_name}` requires {}",
                    rec_params.len(),
                    iface_params.len()
                ),
                "Adjust the parameter list to match the interface declaration.",
                span,
            );
            return;
        }

        for (i, (ip, rp)) in iface_params.iter().zip(rec_params.iter()).enumerate() {
            if !ip.ty.compatible_with(&rp.ty) {
                self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!(
                            "`{record_name}.{method_name}` parameter {} type mismatch: interface expects `{}`, record has `{}`",
                            i + 1, ip.ty, rp.ty
                        ),
                    "Change the parameter type to match the interface declaration.",
                    span,
                );
            }
        }

        match (iface_ret, rec_ret) {
            (Some(ir), Some(rr)) if !ir.compatible_with(rr) => {
                self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!(
                            "`{record_name}.{method_name}` return type mismatch: interface expects `{ir}`, record has `{rr}`"
                        ),
                    "Change the return type to match the interface declaration.",
                    span,
                );
            }
            _ => {}
        }
    }
}
