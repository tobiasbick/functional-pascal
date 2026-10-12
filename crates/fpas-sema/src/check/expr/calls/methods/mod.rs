//! Record instance and static call checking.
//! Documentation: docs/pascal/language/types/record-methods.md.

mod lookup;

use super::super::super::Checker;
use crate::check::MethodCallTarget;
use crate::types::{MethodKind, Ty};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::{Designator, DesignatorPart, Expr};

impl Checker {
    /// Try to resolve `designator(args)` as a record instance or static member call.
    /// Returns `Some(return_ty)` when the call is a record member invocation.
    pub(super) fn try_check_method_call(
        &mut self,
        call_expr: &Expr,
        designator: &Designator,
        args: &[Expr],
        span: Span,
        expected: Option<&Ty>,
    ) -> Option<Ty> {
        self.try_check_method_call_like(
            MethodCallSite::Expression {
                call_expr,
                allow_procedure_result: false,
                expected,
            },
            designator,
            args,
            span,
        )
    }

    /// Checks a spawned method without an expected result context.
    pub(in crate::check::expr) fn try_check_method_go_call(
        &mut self,
        call_expr: &Expr,
        designator: &Designator,
        args: &[Expr],
        span: Span,
    ) -> Option<Ty> {
        self.try_check_method_call_like(
            MethodCallSite::Expression {
                call_expr,
                allow_procedure_result: true,
                expected: None,
            },
            designator,
            args,
            span,
        )
    }

    /// Try to resolve a call expression or call statement as a record instance or
    /// static member call. Returns `Some(result_ty)` when the call is handled here.
    pub(in crate::check) fn try_check_method_call_like(
        &mut self,
        site: MethodCallSite<'_>,
        designator: &Designator,
        args: &[Expr],
        span: Span,
    ) -> Option<Ty> {
        let (call_key, allow_procedure_result, expected) = match site {
            MethodCallSite::Expression {
                call_expr,
                allow_procedure_result,
                expected,
            } => (
                Self::expr_lookup_key(call_expr),
                allow_procedure_result,
                expected,
            ),
            MethodCallSite::Statement => (crate::designator_lookup_key(designator), true, None),
        };
        if designator.parts.len() < 2 {
            return None;
        }

        let method_name = match designator.parts.last()? {
            DesignatorPart::Ident(name, _) => name.clone(),
            _ => return None,
        };

        let receiver_designator = Designator {
            parts: designator.parts[..designator.parts.len() - 1].to_vec(),
            span: designator.span,
        };

        let through_type = self.designator_denotes_type(&receiver_designator);
        // Resolve the full receiver path (supports qualified aliases like `Geom.Api.Point`).
        let receiver_ty = self.check_designator_prefix_expr(designator, designator.parts.len() - 1);
        let resolved_receiver_ty = self.resolve_visible_type(&receiver_ty);

        let record_ty = match &resolved_receiver_ty {
            Ty::Record(record_ty) => record_ty.clone(),
            _ => return None,
        };

        if self.reject_private_record_member(&record_ty, &method_name, span) {
            self.check_args_only(args);
            return Some(Ty::Error);
        }

        if !through_type
            && (record_ty
                .fields
                .iter()
                .any(|(name, _)| name.eq_ignore_ascii_case(&method_name)))
        {
            let member_ty = self.check_designator_expr(designator);
            return Some(self.check_member_value_call(
                call_key,
                &method_name,
                &member_ty,
                args,
                span,
                allow_procedure_result,
            ));
        }

        let qualified = format!("{}.{}", record_ty.name, method_name);

        if through_type {
            if let Some(func_ty) =
                self.resolve_static_function(&record_ty, &method_name, &qualified)
            {
                self.method_calls
                    .insert(call_key, MethodCallTarget::Static(qualified.clone()));
                let inferred = self.check_static_call_args(
                    &qualified,
                    &func_ty.type_params,
                    &func_ty.params,
                    args,
                    span,
                    Some((&func_ty.return_type, expected)),
                );
                return Some(self.check_call_result_type(
                    &func_ty.return_type,
                    &func_ty.type_params,
                    &inferred,
                    span,
                ));
            }

            if let Some(proc_ty) = self.resolve_static_procedure(&record_ty, &method_name) {
                self.method_calls
                    .insert(call_key, MethodCallTarget::Static(qualified.clone()));
                self.check_static_call_args(
                    &qualified,
                    &proc_ty.type_params,
                    &proc_ty.params,
                    args,
                    span,
                    None,
                );
                if allow_procedure_result {
                    return Some(Ty::Unit);
                }
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!("Static procedure `{qualified}` does not return a value"),
                    "Use a static function instead if you need a return value.",
                    span,
                );
                return Some(Ty::Error);
            }

            if self
                .resolve_method_kind(&record_ty, &method_name, &qualified)
                .is_some()
            {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!(
                        "`{method_name}` is an instance method and must be called through a `{}.…` value",
                        record_ty.name
                    ),
                    format!(
                        "Call it as `Value.{method_name}(...)` where `Value` has type `{}`.",
                        record_ty.name
                    ),
                    span,
                );
                self.check_args_only(args);
                return Some(Ty::Error);
            }
            return None;
        }

        // Value receiver: instance methods only.
        if let Some(routine_kind) = self.static_routine_kind_on_record(&record_ty, &method_name) {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "`{method_name}` is a static {routine_kind} and must be called through the type `{}.{}`",
                    record_ty.name, method_name
                ),
                format!(
                    "Write `{}.{}(...)` instead of calling it on a value.",
                    record_ty.name, method_name
                ),
                span,
            );
            self.check_args_only(args);
            return Some(Ty::Error);
        }

        let method_kind = self.resolve_method_kind(&record_ty, &method_name, &qualified)?;

        self.method_calls.insert(
            call_key,
            MethodCallTarget::Instance {
                qualified_name: qualified.clone(),
            },
        );

        match &method_kind {
            MethodKind::Function(func_ty) => {
                let Some(visible_params) = func_ty.params.get(1..) else {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!(
                            "{} `{qualified}` must declare `Self` as its first parameter",
                            site.method_label()
                        ),
                        "Declare the method as `function Name(Self: RecordType; ...)`.",
                        span,
                    );
                    return Some(Ty::Error);
                };
                let inferred = self.check_method_call_args(
                    &qualified,
                    &func_ty.type_params,
                    visible_params,
                    args,
                    span,
                    Some((&func_ty.return_type, expected)),
                );
                Some(self.check_call_result_type(
                    &func_ty.return_type,
                    &func_ty.type_params,
                    &inferred,
                    span,
                ))
            }
            MethodKind::Procedure(proc_ty) => {
                let Some(visible_params) = proc_ty.params.get(1..) else {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!(
                            "{} `{qualified}` must declare `Self` as its first parameter",
                            site.method_label()
                        ),
                        "Declare the method as `procedure Name(Self: RecordType; ...)`.",
                        span,
                    );
                    return Some(Ty::Error);
                };
                self.check_method_call_args(
                    &qualified,
                    &proc_ty.type_params,
                    visible_params,
                    args,
                    span,
                    None,
                );
                if allow_procedure_result {
                    Some(Ty::Unit)
                } else {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!("Method procedure `{qualified}` does not return a value"),
                        "Use a method function instead if you need a return value.",
                        span,
                    );
                    Some(Ty::Error)
                }
            }
        }
    }
}

/// Syntactic position of a record method call.
#[derive(Clone, Copy)]
pub(in crate::check) enum MethodCallSite<'a> {
    /// Call expression; `allow_procedure_result` accepts procedures (e.g. `go` calls).
    Expression {
        call_expr: &'a Expr,
        allow_procedure_result: bool,
        expected: Option<&'a Ty>,
    },
    /// Call statement keyed by its designator.
    Statement,
}

impl MethodCallSite<'_> {
    /// Diagnostic label for a method that lacks its `Self` receiver.
    fn method_label(self) -> &'static str {
        match self {
            Self::Expression { .. } => "Record method",
            Self::Statement => "Method",
        }
    }
}
