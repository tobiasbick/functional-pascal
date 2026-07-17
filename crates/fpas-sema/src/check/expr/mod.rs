//! Expression typing.
//!
//! **Documentation:** `docs/pascal/language/basics/README.md`, `docs/pascal/language/functions/README.md`,
//! and `docs/pascal/language/error-handling/README.md` (from the repository root).

mod bound_method;
mod calls;
mod closure;
mod designator;
mod event_access;
mod operators;
mod postfix;

use super::Checker;
use crate::scope::SymbolKind;
use crate::types::Ty;
use fpas_parser::*;

pub(crate) use event_access::EventRaiseRequest;

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
            Expr::Nil(span) => {
                self.error_with_code(
                    fpas_diagnostics::codes::SEMA_TYPE_MISMATCH,
                    "`nil` is only valid when clearing an event",
                    "Write `Event := nil` to clear a handler, or use `None` for an `Option`.",
                    *span,
                );
                Ty::Error
            }
            Expr::Try(inner, span) => self.check_try_expr(inner, *span),
            Expr::Go(inner, span) => self.check_go_expr(inner, *span),
            Expr::RecordUpdate { base, fields, span } => {
                self.check_record_update(base, fields, *span)
            }
            Expr::Postfix {
                base, operations, ..
            } => self.check_postfix_expr(base, operations),
            Expr::Closure(closure) => self.check_closure_expr(
                expr,
                closure.is_function,
                &closure.params,
                closure.return_type.as_ref(),
                &closure.body,
                closure.span,
            ),
            Expr::Error(_) => Ty::Error,
        };
        let key = Self::expr_lookup_key(expr);
        self.expr_types.insert(key, ty.clone());
        self.propagate_task_bound_expr(expr, key);
        ty
    }

    fn check_go_expr(&mut self, inner: &Expr, span: fpas_lexer::Span) -> Ty {
        let inner_ty = match inner {
            Expr::Call {
                designator,
                args,
                span: call_span,
            } => {
                use calls::CallResolution;
                match self.resolve_call_target(inner, designator, args, *call_span, true) {
                    CallResolution::Symbol { kind, ty } => {
                        let name = Self::resolve_designator_name(designator);
                        self.check_known_go_call_symbol(&name, kind, ty, args, *call_span)
                    }
                    CallResolution::MethodResult(ty) => ty,
                    CallResolution::Failed => Ty::Error,
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
        self.reject_spawned_event_raise(inner_key, span);
        if self.callable_expr_is_task_bound(inner) {
            self.error_with_code(
                fpas_diagnostics::codes::SEMA_TASK_BOUND_CALLABLE,
                "Cannot spawn a task-bound callable across a task boundary",
                "Mutable captures make a closure task-bound. Pass immutable values instead, or invoke the closure on the same task.",
                span,
            );
        }
        Ty::Task(Box::new(inner_ty))
    }

    fn propagate_task_bound_expr(&mut self, expr: &Expr, key: usize) {
        match expr {
            Expr::Paren(inner, _) if self.expr_is_task_bound(Self::expr_lookup_key(inner)) => {
                self.mark_expr_task_bound(key);
            }
            Expr::Designator(designator) if self.designator_refers_to_task_bound(designator) => {
                self.mark_expr_task_bound(key);
            }
            _ => {}
        }
    }

    pub(crate) fn callable_expr_is_task_bound(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Call { designator, .. } => self.designator_refers_to_task_bound(designator),
            other => self.expr_is_task_bound(Self::expr_lookup_key(other)),
        }
    }

    pub(crate) fn designator_refers_to_task_bound(&self, designator: &Designator) -> bool {
        let name = Self::resolve_designator_name(designator);
        let Some(symbol) = self.scopes.lookup(&name).or_else(|| {
            designator.parts.first().and_then(|part| match part {
                DesignatorPart::Ident(base, _) => self.scopes.lookup(base),
                _ => None,
            })
        }) else {
            return false;
        };
        if symbol.task_bound {
            return true;
        }
        if matches!(symbol.kind, SymbolKind::Function | SymbolKind::Procedure) {
            let canonical = crate::scope::canonical_symbol_name(&name);
            return self
                .nested_routine_captures
                .get(&canonical)
                .is_some_and(|info| info.task_bound);
        }
        false
    }

    fn check_array_literal(&mut self, elements: &[Expr]) -> Ty {
        if elements.is_empty() {
            return Ty::Array(Box::new(Ty::Error));
        }

        let first_ty = self.check_expr(&elements[0]);
        for element in &elements[1..] {
            let element_ty = self.check_expr(element);
            self.check_type_compat(&first_ty, &element_ty, "array element", element.span());
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
            self.check_type_compat(&first_key_ty, &key_ty, "dict key", key.span());
            let val_ty = self.check_expr(val);
            self.check_type_compat(&first_val_ty, &val_ty, "dict value", val.span());
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
            fields: field_types,
            methods: Vec::new(),
            static_functions: Vec::new(),
            properties: Vec::new(),
            events: Vec::new(),
        })
    }

    /// Type-check a record update expression: `base with Field := Value; … end`.
    ///
    /// The base must resolve to a record type. Each override field must exist in
    /// that record and have a compatible value type. The result has the same type
    /// as the base expression.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-update.md`
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
                    format!("`with` update requires a record value, found `{base_ty}`"),
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
            } else if record_ty
                .properties
                .iter()
                .any(|(name, _)| name.eq_ignore_ascii_case(&field_init.name))
            {
                self.error_with_code(
                    fpas_diagnostics::codes::SEMA_UNKNOWN_NAME,
                    format!(
                        "Record type `{}` property `{}` cannot be set in a `with` update",
                        record_ty.name, field_init.name
                    ),
                    "Properties are not record fields. Assign to the property with `Value.Prop := …` instead.",
                    span,
                );
                let _ = self.check_expr(&field_init.value);
            } else if self
                .find_record_event_on_type(&record_ty, &field_init.name)
                .is_some()
            {
                self.error_with_code(
                    fpas_diagnostics::codes::SEMA_UNKNOWN_NAME,
                    format!(
                        "Record type `{}` event `{}` cannot be set in a `with` update",
                        record_ty.name, field_init.name
                    ),
                    "Events are not record fields. Assign with `Value.Event := Handler` or `:= nil`.",
                    span,
                );
                let _ = self.check_expr(&field_init.value);
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
                    format!("try requires Result or Option, found `{inner_ty}`"),
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
                            "`try` propagates `{inner_ty}`, but function `{}` returns `{return_ty}`",
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
                        "`try` propagates `{inner_ty}`, but function `{}` returns `{return_ty}`",
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
                        "`try` propagates `{inner_ty}`, but function `{}` returns `{return_ty}`",
                        function_ctx.name
                    ),
                    "Use `try` on `Option` only inside a function that returns `Option of T`.",
                    span,
                );
            }
            _ => {}
        }
    }
}
