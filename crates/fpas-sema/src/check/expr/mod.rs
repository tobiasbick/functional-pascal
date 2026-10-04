//! Expression typing.
//!
//! **Documentation:** `docs/pascal/language/basics/README.md`, `docs/pascal/language/functions/README.md`,
//! `docs/pascal/language/error-handling/README.md`, and
//! `docs/pascal/language/types/channels.md` (from the repository root).

mod bound_method;
mod calls;
mod closure;
mod construction;
mod decisions;
mod designator;
mod dictionaries;
mod equality;
mod event_access;
mod expected;
mod literals;
mod obsolete_records;
mod operators;
mod postfix;
mod record_fields;
mod task_bound;
mod try_values;

use super::Checker;
use crate::types::Ty;
use fpas_parser::*;

pub(in crate::check) use calls::MethodCallSite;

impl Checker {
    pub(crate) fn check_expr(&mut self, expr: &Expr) -> Ty {
        if let Some(ty) = self.prechecked_receivers.get(&Self::expr_lookup_key(expr)) {
            return ty.clone();
        }
        if let Some(ty) = self.try_check_enum_construction(expr, None) {
            return ty;
        }
        let ty = match expr {
            Expr::If(decision) => self.check_if_expression(decision, None),
            Expr::Case(decision) => self.check_case_expression(decision, None),
            Expr::RecordConstruction { .. } => self
                .try_check_record_construction(expr, None)
                .unwrap_or(Ty::Error),
            Expr::Integer(_, _) => Ty::Integer,
            Expr::Real(_, _) => Ty::Real,
            Expr::Str(_, _) => Ty::String,
            Expr::Bool(_, _) => Ty::Boolean,
            Expr::Designator(designator) => self.check_designator_value_expr(designator),
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
            Expr::InvalidRecord(span) => self.reject_obsolete_record(*span, None),
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
                    "Write `Event := nil` to clear a handler, or use `Option.None` for an `Option`.",
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

    /// Check a spawned call using the same target rules in both go positions.
    pub(in crate::check) fn check_go_expr(&mut self, inner: &Expr, span: fpas_lexer::Span) -> Ty {
        let inner_ty = match inner {
            Expr::Call {
                designator,
                args,
                span: call_span,
            } => {
                use calls::CallResolution;
                match self.resolve_call_target(inner, designator, args, *call_span, true) {
                    CallResolution::Symbol { kind, ty } => {
                        let name = self.resolve_designator_name(designator);
                        let dispatch = self.builtin_std_dispatch_name(&name);
                        if dispatch.starts_with("Std.")
                            && kind != crate::scope::SymbolKind::EnumVariantConstructor
                        {
                            self.intrinsic_calls
                                .insert(Self::expr_lookup_key(inner), dispatch);
                        }
                        self.check_known_go_call_symbol(&name, kind, ty, args, *call_span)
                    }
                    CallResolution::MethodResult(ty) => ty,
                    CallResolution::Failed => Ty::Error,
                }
            }
            Expr::Postfix {
                base, operations, ..
            } => self.check_postfix_go(base, operations, span),
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

    pub(crate) fn callable_expr_is_task_bound(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Call { designator, .. } => {
                self.fluent_calls
                    .get(&Self::expr_lookup_key(expr))
                    .and_then(|target| self.scopes.lookup(&target.name))
                    .is_some_and(|symbol| symbol.task_bound)
                    || self.designator_refers_to_task_bound(designator)
            }
            Expr::Postfix { operations, .. } => {
                operations.last().is_some_and(|operation| {
                    self.expr_is_task_bound(Self::postfix_operation_lookup_key(operation))
                }) || operations
                    .last()
                    .and_then(|operation| {
                        self.fluent_calls
                            .get(&Self::postfix_operation_lookup_key(operation))
                    })
                    .and_then(|target| self.scopes.lookup(&target.name))
                    .is_some_and(|symbol| symbol.task_bound)
                    || self.expr_is_task_bound(Self::expr_lookup_key(expr))
            }
            other => self.expr_is_task_bound(Self::expr_lookup_key(other)),
        }
    }

    pub(crate) fn designator_refers_to_task_bound(&self, designator: &Designator) -> bool {
        let name = self.qualified_import_name(&Self::designator_name(designator));
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
        false
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
        self.validate_unique_record_fields(fields, "record update");
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
            if self.reject_private_record_member(&record_ty, &field_init.name, span) {
                let _ = self.check_expr(&field_init.value);
                continue;
            }
            if let Some((_, field_ty)) = record_ty
                .fields
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(&field_init.name))
            {
                let value_ty = self.check_expr_with_expected(&field_init.value, field_ty);
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

        // Preserve the declared record identity of the base.
        base_ty
    }
}
