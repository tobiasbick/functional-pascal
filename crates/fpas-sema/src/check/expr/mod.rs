//! Expression typing.
//!
//! **Documentation:** `docs/pascal/language/basics/README.md`, `docs/pascal/language/functions/README.md`,
//! `docs/pascal/language/error-handling/README.md`, and
//! `docs/pascal/language/types/channels.md` (from the repository root).

mod boolean;
mod bound_method;
mod calls;
mod closure;
mod collections;
mod construction_inference;
mod conversion_argument;
mod decision;
mod designator;
mod distinct_conversion;
mod distinct_operators;
mod enum_construction;
mod equality;
mod is_test;
mod operators;
mod postfix;
mod postfix_inference;
mod record_construction;
mod record_fields;
mod subrange_conversion;
mod subrange_membership;
mod task_bound;
mod tasks;

use super::Checker;
use crate::types::Ty;
use fpas_parser::*;

pub(in crate::check) use calls::MethodCallSite;

impl Checker {
    /// Check an expression without an expected result type.
    pub(crate) fn check_expr(&mut self, expr: &Expr) -> Ty {
        self.check_expr_with_expected(expr, None)
    }

    /// Checks an expression with context for incomplete generic and collection types.
    pub(crate) fn check_expr_with_expected(&mut self, expr: &Expr, expected: Option<&Ty>) -> Ty {
        if let Some(ty) = self.prechecked_receivers.get(&Self::expr_lookup_key(expr)) {
            return ty.clone();
        }
        let ty = match expr {
            Expr::Integer(_, _) => Ty::Integer,
            Expr::Real(_, _) => Ty::Real,
            Expr::Str(_, _) => Ty::String,
            Expr::Bool(_, _) => Ty::Boolean,
            Expr::Designator(designator) => {
                self.reject_var_parameter_routine_value(designator);
                let ty = self.check_value_designator_expr(designator);
                let name = Self::resolve_designator_name(designator);
                if self
                    .scopes
                    .lookup(&name)
                    .is_some_and(|symbol| symbol.kind == crate::scope::SymbolKind::EnumMember)
                {
                    self.check_enum_construction(&name, &ty, &[], designator.span, expected)
                } else {
                    ty
                }
            }
            Expr::Call {
                designator,
                args,
                span,
            } => self.check_call_expr(expr, designator, args, *span, expected),
            Expr::UnaryOp { op, operand, span } => self.check_unary_expr(*op, operand, *span),
            Expr::BinaryOp {
                op,
                left,
                right,
                span,
            } => self.check_binary_expr(expr, *op, left, right, *span),
            Expr::Paren(inner, _) => self.check_expr_with_expected(inner, expected),
            Expr::ArrayLiteral(elements, _) => self.check_array_literal(elements, expected),
            Expr::DictLiteral(pairs, _) => self.check_dict_literal(pairs, expected),
            Expr::ResultOk(inner, _) => {
                let inner_expected = match expected {
                    Some(Ty::Result(ok, _)) => Some(ok.as_ref()),
                    _ => None,
                };
                let inner_ty = self.check_payload_argument("Ok", inner, inner_expected);
                let error = match expected {
                    Some(Ty::Result(_, error)) => *error.clone(),
                    _ => Ty::Error,
                };
                Ty::Result(Box::new(inner_ty), Box::new(error))
            }
            Expr::ResultError(inner, _) => {
                let inner_expected = match expected {
                    Some(Ty::Result(_, error)) => Some(error.as_ref()),
                    _ => None,
                };
                let inner_ty = self.check_payload_argument("Error", inner, inner_expected);
                let ok = match expected {
                    Some(Ty::Result(ok, _)) => *ok.clone(),
                    _ => Ty::Error,
                };
                Ty::Result(Box::new(ok), Box::new(inner_ty))
            }
            Expr::OptionSome(inner, _) => {
                let inner_expected = match expected {
                    Some(Ty::Option(value)) => Some(value.as_ref()),
                    _ => None,
                };
                let inner_ty = self.check_payload_argument("Some", inner, inner_expected);
                Ty::Option(Box::new(inner_ty))
            }
            Expr::OptionNone(_) => match expected {
                Some(expected @ Ty::Option(_)) => expected.clone(),
                _ => Ty::Option(Box::new(Ty::Error)),
            },
            Expr::Try(inner, span) => self.check_try_expr(inner, *span),
            Expr::Go(inner, span) => self.check_go_expr(inner, *span),
            Expr::RecordUpdate { base, fields, span } => {
                self.check_record_update(base, fields, *span)
            }
            Expr::Postfix {
                base, operations, ..
            } => self.check_postfix_expr(base, operations, expected),
            Expr::Closure(closure) => self.check_closure_expr(
                expr,
                closure.is_function,
                &closure.params,
                closure.return_type.as_ref(),
                &closure.body,
                closure.span,
            ),
            Expr::NamedArgument { .. } => self.check_misplaced_named_argument(expr),
            Expr::VarArgument { .. } => self.check_misplaced_var_argument(expr),
            Expr::Is { value, pattern, .. } => self.check_is_test(value, pattern),
            Expr::If {
                branches,
                else_value,
                ..
            } => self.check_if_expr(branches, else_value, expected),
            Expr::Case {
                selector,
                arms,
                else_arm,
                span,
            } => self.check_case_expr(selector, arms, else_arm.as_deref(), *span, expected),
            Expr::Error(_) => Ty::Error,
        };
        let ty = if let Some(expected) = expected {
            self.specialize_expected_callable(ty, expected, expr.span())
        } else {
            ty
        };
        if let Expr::Designator(designator) = expr {
            let parameters = match &ty {
                Ty::Function(function) => function.type_params.as_slice(),
                Ty::Procedure(procedure) => procedure.type_params.as_slice(),
                _ => &[],
            };
            let name = format!(
                "{} (callable value)",
                Self::resolve_designator_name(designator)
            );
            self.complete_or_defer_type_arguments(&name,
                parameters.iter().map(|parameter| parameter.display_name().to_owned()).collect(),
                "Supply an explicit callable signature, for example `const F: function(Value: integer): integer := Identity;`. Every declared parameter needs evidence; remove parameters unused by the signature.",
                expr.span(), true);
        }
        let key = Self::expr_lookup_key(expr);
        self.expr_types.insert(key, ty.clone());
        self.propagate_task_bound_expr(expr, key);
        ty
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
                let value_ty = self.check_expr_with_expected(&field_init.value, Some(field_ty));
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
                "Wrap the expression in a function that returns `Result of (T, E)` or `Option of T`.",
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
                "Use `try` inside a function that returns `Result of (T, E)` or `Option of T`.",
                span,
            );
            return;
        };

        if return_ty.is_error() {
            return;
        }

        match (inner_ty, &return_ty) {
            (Ty::Result(_, inner_err), Ty::Result(_, outer_err)) => {
                if !outer_err.propagation_compatible_with(inner_err) {
                    self.error_with_code(
                        fpas_diagnostics::codes::SEMA_TYPE_MISMATCH,
                        format!(
                            "`try` propagates `{inner_ty}`, but function `{}` returns `{return_ty}`",
                            function_ctx.name
                        ),
                        "Make the enclosing function return `Result of (ValueType, ErrorType)` with the same error type.",
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
                    "Use `try` on `Result` only inside a function that returns `Result of (T, E)` with a compatible error type.",
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
