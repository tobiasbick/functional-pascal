//! Expression typing.
//!
//! **Documentation:** `docs/pascal/02-basics.md`, `docs/pascal/04-functions.md`,
//! and `docs/pascal/07-error-handling.md` (from the repository root).

mod calls;
mod designator;
mod operators;

use super::Checker;
use crate::types::Ty;
use fpas_parser::*;

impl Checker {
    pub(crate) fn check_expr(&mut self, expr: &Expr) -> Ty {
        let ty = match expr {
            Expr::Integer(_, _) => Ty::Integer,
            Expr::Real(_, _) => Ty::Real,
            Expr::Str(_, _) => Ty::String,
            Expr::Bool(_, _) => Ty::Boolean,
            Expr::Designator(designator) => self.check_designator_expr(designator),
            Expr::Call {
                designator,
                args,
                span,
            } => self.check_call_expr(expr, designator, args, *span),
            Expr::UnaryOp { op, operand, span } => self.check_unary_expr(*op, operand, *span),
            Expr::BinaryOp {
                op,
                left,
                right,
                span,
            } => self.check_binary_expr(*op, left, right, *span),
            Expr::Paren(inner, _) => self.check_expr(inner),
            Expr::ArrayLiteral(elements, _) => self.check_array_literal(elements),
            Expr::DictLiteral(pairs, _) => self.check_dict_literal(pairs),
            Expr::RecordLiteral { fields, .. } => self.check_record_literal(fields),
            Expr::New {
                type_expr,
                fields,
                span,
            } => self.check_new_expr(type_expr, fields, *span),
            Expr::ResultOk(inner, _) => {
                let inner_ty = self.check_expr(inner);
                Ty::Result(Box::new(inner_ty), Box::new(Ty::Error))
            }
            Expr::ResultError(inner, _) => {
                let inner_ty = self.check_expr(inner);
                Ty::Result(Box::new(Ty::Error), Box::new(inner_ty))
            }
            Expr::OptionSome(inner, _) => {
                let inner_ty = self.check_expr(inner);
                Ty::Option(Box::new(inner_ty))
            }
            Expr::OptionNone(_) => Ty::Option(Box::new(Ty::Error)),
            Expr::Try(inner, span) => self.check_try_expr(inner, *span),
            Expr::Function {
                params,
                return_type,
                body,
                span: _,
            } => self.check_function_expr(params, return_type, body),
            Expr::Go(inner, span) => self.check_go_expr(inner, *span),
            Expr::RecordUpdate { base, fields, span } => {
                self.check_record_update(base, fields, *span)
            }
            Expr::Error(_) => Ty::Error,
        };
        let key = Self::expr_lookup_key(expr);
        self.expr_types.insert(key, ty.clone());
        ty
    }

    fn check_go_expr(&mut self, inner: &Expr, span: fpas_lexer::Span) -> Ty {
        let inner_ty = match inner {
            Expr::Call {
                designator,
                args,
                span: call_span,
            } => {
                let name = Self::resolve_designator_name(designator);
                self.ensure_fq_std_unit_loaded(&name);

                if let Some(symbol) = self.scopes.lookup(&name) {
                    self.check_known_go_call_symbol(
                        &name,
                        symbol.kind,
                        symbol.ty.clone(),
                        args,
                        *call_span,
                    )
                } else if let Some(result) =
                    self.try_check_method_go_call(inner, designator, args, *call_span)
                {
                    result
                } else if let Some(hint) = self.ambiguous_hint(&name) {
                    self.error_with_code(
                        fpas_diagnostics::codes::SEMA_AMBIGUOUS_IMPORTED_NAME,
                        format!("Ambiguous name `{name}`"),
                        hint,
                        *call_span,
                    );
                    self.check_args_only(args);
                    Ty::Error
                } else {
                    let hint = self.hint_unknown_callable(&name);
                    self.error_with_code(
                        fpas_diagnostics::codes::SEMA_UNKNOWN_NAME,
                        format!("Unknown function or procedure `{name}`"),
                        hint,
                        *call_span,
                    );
                    self.check_args_only(args);
                    Ty::Error
                }
            }
            _ => {
                let _ = self.check_expr(inner);
                self.error_with_code(
                    fpas_diagnostics::codes::SEMA_TYPE_MISMATCH,
                    "`go` requires a function or procedure call",
                    "Use `go FunctionName(args)` or `go SomeCallable(args)`.",
                    span,
                );
                Ty::Error
            }
        };

        let inner_key = Self::expr_lookup_key(inner);
        self.expr_types.insert(inner_key, inner_ty.clone());
        Ty::Task(Box::new(inner_ty))
    }

    fn check_array_literal(&mut self, elements: &[Expr]) -> Ty {
        if elements.is_empty() {
            return Ty::Array(Box::new(Ty::Error));
        }

        let first_ty = self.check_expr(&elements[0]);
        for element in &elements[1..] {
            let element_ty = self.check_expr(element);
            self.check_type_compat(
                &first_ty,
                &element_ty,
                "array element",
                super::spans::expr_span(element),
            );
        }

        Ty::Array(Box::new(first_ty))
    }

    fn check_dict_literal(&mut self, pairs: &[(Expr, Expr)]) -> Ty {
        if pairs.is_empty() {
            return Ty::Dict(Box::new(Ty::Error), Box::new(Ty::Error));
        }

        let first_key_ty = self.check_expr(&pairs[0].0);
        let first_val_ty = self.check_expr(&pairs[0].1);
        for (key, val) in &pairs[1..] {
            let key_ty = self.check_expr(key);
            self.check_type_compat(
                &first_key_ty,
                &key_ty,
                "dict key",
                super::spans::expr_span(key),
            );
            let val_ty = self.check_expr(val);
            self.check_type_compat(
                &first_val_ty,
                &val_ty,
                "dict value",
                super::spans::expr_span(val),
            );
        }

        Ty::Dict(Box::new(first_key_ty), Box::new(first_val_ty))
    }

    fn check_record_literal(&mut self, fields: &[FieldInit]) -> Ty {
        let field_types = fields
            .iter()
            .map(|field| (field.name.clone(), self.check_expr(&field.value)))
            .collect();

        Ty::Record(crate::types::RecordTy {
            name: "<anonymous>".into(),
            type_params: Vec::new(),
            fields: field_types,
            methods: Vec::new(),
            implements: Vec::new(),
        })
    }

    fn check_new_expr(
        &mut self,
        type_expr: &TypeExpr,
        fields: &[FieldInit],
        span: fpas_lexer::Span,
    ) -> Ty {
        let target_ty = self.resolve_type_expr(type_expr);
        if !matches!(self.resolve_visible_type(&target_ty), Ty::Record(_)) && !target_ty.is_error()
        {
            self.error_with_code(
                fpas_diagnostics::codes::SEMA_TYPE_MISMATCH,
                format!("`new` requires a record type, found `{target_ty:?}`"),
                "Use `new RecordType with Field := Value; ... end`.",
                span,
            );
            for field in fields {
                let _ = self.check_expr(&field.value);
            }
            return Ty::Error;
        }

        let value_ty = self.check_record_literal(fields);
        if !target_ty.is_error() && !value_ty.is_error() {
            self.check_type_compat(&target_ty, &value_ty, "new initializer", span);
        }

        Ty::Ref(Box::new(target_ty))
    }

    /// Type-check a record update expression: `base with Field := Value; … end`.
    ///
    /// The base must resolve to a record type. Each override field must exist in
    /// that record and have a compatible value type. The result has the same type
    /// as the base expression.
    ///
    /// **Documentation:** `docs/pascal/05-types.md` (Record Update Expression)
    fn check_record_update(
        &mut self,
        base: &Expr,
        fields: &[FieldInit],
        span: fpas_lexer::Span,
    ) -> Ty {
        let base_ty = self.check_expr(base);
        let resolved = self.resolve_visible_type(&base_ty);

        let record_ty = match resolved {
            Ty::Record(r) => r,
            _ if !base_ty.is_error() => {
                self.error_with_code(
                    fpas_diagnostics::codes::SEMA_TYPE_MISMATCH,
                    format!("`with` update requires a record value, found `{base_ty:?}`"),
                    "Use `RecordExpr with Field := NewValue; … end` on a record value.",
                    span,
                );
                for field in fields {
                    let _ = self.check_expr(&field.value);
                }
                return Ty::Error;
            }
            _ => {
                for field in fields {
                    let _ = self.check_expr(&field.value);
                }
                return Ty::Error;
            }
        };

        // Validate each override field.
        for field_init in fields {
            if let Some((_, field_ty)) = record_ty
                .fields
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(&field_init.name))
            {
                let value_ty = self.check_expr(&field_init.value);
                self.check_type_compat(
                    field_ty,
                    &value_ty,
                    &format!("field update `{}`", field_init.name),
                    span,
                );
            } else {
                let known: Vec<&str> = record_ty.fields.iter().map(|(n, _)| n.as_str()).collect();
                self.error_with_code(
                    fpas_diagnostics::codes::SEMA_UNKNOWN_NAME,
                    format!(
                        "Record type `{}` has no field `{}`",
                        record_ty.name, field_init.name
                    ),
                    format!(
                        "Known fields: {}. Use an existing field name in the update.",
                        known.join(", ")
                    ),
                    span,
                );
                let _ = self.check_expr(&field_init.value);
            }
        }

        // Return the same type as the base (named or anonymous).
        base_ty
    }

    fn check_try_expr(&mut self, inner: &Expr, span: fpas_lexer::Span) -> Ty {
        let inner_ty = self.check_expr(inner);
        match &inner_ty {
            Ty::Result(ok, _) => {
                self.check_try_context(&inner_ty, span);
                *ok.clone()
            }
            Ty::Option(inner) => {
                self.check_try_context(&inner_ty, span);
                *inner.clone()
            }
            Ty::Error => Ty::Error,
            _ => {
                self.error_with_code(
                    fpas_diagnostics::codes::SEMA_TYPE_MISMATCH,
                    format!("try requires Result or Option, found `{inner_ty:?}`"),
                    "Use try only on Result or Option values.".to_string(),
                    span,
                );
                Ty::Error
            }
        }
    }

    fn check_try_context(&mut self, inner_ty: &Ty, span: fpas_lexer::Span) {
        let Some(function_ctx) = self.scopes.function_ctx.clone() else {
            self.error_with_code(
                fpas_diagnostics::codes::SEMA_TYPE_MISMATCH,
                "`try` can only be used inside a function that returns Result or Option",
                "Wrap the expression in a function that returns `Result of T, E` or `Option of T`.",
                span,
            );
            return;
        };

        let Some(return_ty) = function_ctx.return_type else {
            self.error_with_code(
                fpas_diagnostics::codes::SEMA_TYPE_MISMATCH,
                format!(
                    "Procedure `{}` cannot use `try` because it does not return a value",
                    function_ctx.name
                ),
                "Use `try` inside a function that returns `Result of T, E` or `Option of T`.",
                span,
            );
            return;
        };

        if return_ty.is_error() {
            return;
        }

        match (inner_ty, &return_ty) {
            (Ty::Result(_, inner_err), Ty::Result(_, outer_err)) => {
                if !outer_err.compatible_with(inner_err) {
                    self.error_with_code(
                        fpas_diagnostics::codes::SEMA_TYPE_MISMATCH,
                        format!(
                            "`try` propagates `{inner_ty:?}`, but function `{}` returns `{return_ty:?}`",
                            function_ctx.name
                        ),
                        "Make the enclosing function return `Result of <value>, <same error type>`.",
                        span,
                    );
                }
            }
            (Ty::Option(_), Ty::Option(_)) => {}
            (Ty::Result(_, _), _) => {
                self.error_with_code(
                    fpas_diagnostics::codes::SEMA_TYPE_MISMATCH,
                    format!(
                        "`try` propagates `{inner_ty:?}`, but function `{}` returns `{return_ty:?}`",
                        function_ctx.name
                    ),
                    "Use `try` on `Result` only inside a function that returns `Result of T, E` with a compatible error type.",
                    span,
                );
            }
            (Ty::Option(_), _) => {
                self.error_with_code(
                    fpas_diagnostics::codes::SEMA_TYPE_MISMATCH,
                    format!(
                        "`try` propagates `{inner_ty:?}`, but function `{}` returns `{return_ty:?}`",
                        function_ctx.name
                    ),
                    "Use `try` on `Option` only inside a function that returns `Option of T`.",
                    span,
                );
            }
            _ => {}
        }
    }

    /// Type-check an anonymous function expression (lambda / closure).
    ///
    /// **Documentation:** `docs/pascal/04-functions.md`
    fn check_function_expr(
        &mut self,
        params: &[FormalParam],
        return_type_expr: &TypeExpr,
        body: &FuncBody,
    ) -> Ty {
        use crate::scope::{FunctionCtx, Symbol, SymbolKind};
        use crate::types::{FunctionTy, ParamTy};

        let return_ty = self.resolve_type_expr(return_type_expr);
        let param_tys: Vec<ParamTy> = params
            .iter()
            .map(|p| ParamTy {
                mutable: p.mutable,
                name: p.name.clone(),
                ty: self.resolve_type_expr(&p.type_expr),
            })
            .collect();

        if let FuncBody::Block { nested, stmts } = body {
            self.scopes.push_scope();

            for p in &param_tys {
                self.scopes.define(
                    &p.name,
                    Symbol {
                        ty: p.ty.clone(),
                        mutable: p.mutable,
                        kind: SymbolKind::Param,
                    },
                );
            }

            let prev_ctx = self.scopes.function_ctx.take();
            self.scopes.function_ctx = Some(FunctionCtx {
                name: "<lambda>".into(),
                return_type: Some(return_ty.clone()),
            });

            for decl in nested {
                self.check_decl(decl);
            }
            for stmt in stmts {
                self.check_stmt(stmt);
            }

            self.report_missing_forward_declarations_in_current_scope();
            self.scopes.function_ctx = prev_ctx;
            self.scopes.pop_scope();
        }

        Ty::Function(FunctionTy {
            type_params: Vec::new(),
            params: param_tys,
            return_type: Box::new(return_ty),
        })
    }
}
