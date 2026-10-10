//! Fixed catalog lookup for built-in dot operations.
//!
//! **Documentation:** `docs/pascal/language/functions/fluent-calls.md`

use super::super::super::Checker;
use crate::check::FluentCallTarget;
use crate::check::calls::CallTarget;
use crate::types::Ty;
use fpas_diagnostics::codes::{SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME};
use fpas_lexer::Span;
use fpas_parser::{Designator, DesignatorPart, Expr};

impl Checker {
    /// Distinguishes a qualified imported-unit call from a value receiver call.
    pub(in crate::check) fn designator_has_unit_prefix(&self, designator: &Designator) -> bool {
        let Some((_, prefix)) = designator.parts.split_last() else {
            return false;
        };
        let names = prefix
            .iter()
            .map(|part| match part {
                DesignatorPart::Ident(name, _) => Some(name.as_str()),
                DesignatorPart::Index(_, _) => None,
            })
            .collect::<Option<Vec<_>>>();
        names.is_some_and(|names| {
            if names
                .first()
                .is_some_and(|root| self.scopes.imports.is_alias(root))
            {
                return names.len() == 1;
            }
            self.used_unit_names
                .contains(&names.join(".").to_ascii_lowercase())
                && names
                    .first()
                    .is_some_and(|root| self.scopes.lookup(root).is_none())
        })
    }

    /// Checks a designator call as a receiver call when it is not a member call.
    pub(in crate::check) fn try_check_fluent_designator(
        &mut self,
        call_key: usize,
        designator: &Designator,
        args: &[Expr],
        span: Span,
        allow_procedure_result: bool,
    ) -> Option<Ty> {
        let (last, prefix_parts) = designator.parts.split_last()?;
        let DesignatorPart::Ident(name, name_span) = last else {
            return None;
        };
        if prefix_parts.is_empty() {
            return None;
        }
        let prefix = Designator {
            parts: prefix_parts.to_vec(),
            span: designator.span,
        };
        if self.designator_has_unit_prefix(designator) {
            return None;
        }
        if self.designator_denotes_type(&prefix) {
            return None;
        }
        let receiver_ty = self.check_designator_prefix_expr(designator, prefix_parts.len());
        let receiver_ty = self.resolve_visible_type(&receiver_ty);
        if let Ty::Record(record) = &receiver_ty {
            let has_member = record
                .fields
                .iter()
                .any(|(member, _)| member.eq_ignore_ascii_case(name))
                || record
                    .methods
                    .iter()
                    .any(|(member, _)| member.eq_ignore_ascii_case(name))
                || record
                    .static_functions
                    .iter()
                    .any(|(member, _)| member.eq_ignore_ascii_case(name))
                || record
                    .static_procedures
                    .iter()
                    .any(|(member, _)| member.eq_ignore_ascii_case(name));
            if has_member {
                if record
                    .fields
                    .iter()
                    .any(|(member, _)| member.eq_ignore_ascii_case(name))
                {
                    let member_ty = self.check_designator_expr(designator);
                    return Some(self.check_member_value_call(
                        call_key,
                        name,
                        &member_ty,
                        args,
                        span,
                        allow_procedure_result,
                    ));
                }
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!("Record member `{name}` takes priority over receiver-call lookup"),
                    "Call the member with its declared arguments or use a qualified free routine.",
                    span,
                );
                self.check_args_only(args);
                return Some(Ty::Error);
            }
        }
        let receiver = Expr::Designator(prefix);
        Some(self.check_fluent_call(FluentCall {
            call_key,
            receiver: &receiver,
            receiver_ty: &receiver_ty,
            name,
            args,
            span,
            call_span: *name_span,
            allow_procedure_result,
        }))
    }

    /// Checks a call through a callable record field or indexed value.
    ///
    /// **Documentation:** `docs/pascal/language/functions/first-class.md`.
    pub(in crate::check) fn check_member_value_call(
        &mut self,
        call_key: usize,
        name: &str,
        member_ty: &Ty,
        args: &[Expr],
        span: Span,
        allow_procedure_result: bool,
    ) -> Ty {
        let result = match member_ty {
            Ty::Function(signature) => {
                let inferred = self.check_function_call_args(
                    name,
                    signature,
                    CallTarget::FunctionValue,
                    args,
                    span,
                );
                Self::substitute_type_params(&signature.return_type, &inferred)
            }
            Ty::Procedure(signature) => {
                self.check_procedure_call_args(
                    name,
                    signature,
                    CallTarget::FunctionValue,
                    args,
                    span,
                );
                Ty::Unit
            }
            _ => {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!("Record member `{name}` is not callable"),
                    "Use a callable field, or call a qualified free routine.",
                    span,
                );
                self.check_args_only(args);
                return Ty::Error;
            }
        };
        if result == Ty::Unit && !allow_procedure_result {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Record member `{name}` does not return a value"),
                "Use the call as the final operation of a statement.",
                span,
            );
            return Ty::Error;
        }
        self.member_value_calls.insert(call_key, result.clone());
        result
    }

    /// Checks a built-in dot operation selected only by static receiver and name.
    pub(in crate::check) fn check_fluent_call(&mut self, call: FluentCall<'_>) -> Ty {
        let FluentCall {
            call_key,
            receiver,
            receiver_ty,
            name,
            args,
            span,
            call_span,
            allow_procedure_result,
        } = call;
        if receiver_ty.is_error() {
            self.check_args_only(args);
            return Ty::Error;
        }
        let Some(operation) = crate::std_registry::native_operation(receiver_ty, name) else {
            let hint = if receiver_ty == &Ty::String && name.eq_ignore_ascii_case("Substring") {
                "Use `.Slice(Start, Len)` for a checked Unicode-scalar range.".to_string()
            } else if let Ty::Distinct(distinct) = receiver_ty {
                // Documentation: docs/pascal/language/types/distinct-types.md
                format!(
                    "Distinct types do not inherit operations of `{0}`. Unwrap explicitly, for example `{0}(Value).{name}()`.",
                    distinct.underlying
                )
            } else {
                "Use a declared record member or a built-in catalog operation. Call your own free functions ordinarily, for example `Name(Value, …)`.".to_string()
            };
            self.error_with_code(
                SEMA_UNKNOWN_NAME,
                format!("Type `{receiver_ty}` has no dot operation `{name}`"),
                hint,
                span,
            );
            self.check_args_only(args);
            return Ty::Error;
        };
        self.prechecked_receivers
            .insert(Self::expr_lookup_key(receiver), receiver_ty.clone());
        let result = self.check_native_arguments(operation, Some(receiver), args, span);
        self.prechecked_receivers
            .remove(&Self::expr_lookup_key(receiver));
        if result == Ty::Unit && !allow_procedure_result {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("`{name}` does not return a value"),
                "Use the operation as the final step of a statement.",
                span,
            );
            return Ty::Error;
        }
        self.fluent_calls.insert(
            call_key,
            FluentCallTarget {
                name: operation.implementation.to_string(),
                receiver_ty: receiver_ty.clone(),
                result_ty: result.clone(),
                call_span,
            },
        );
        result
    }
}

/// One built-in dot call `Receiver.Name(Args)` resolved through the fixed catalog.
pub(in crate::check) struct FluentCall<'a> {
    /// Lookup key recording the resolved target.
    pub(in crate::check) call_key: usize,
    /// Receiver expression passed as the first argument.
    pub(in crate::check) receiver: &'a Expr,
    /// Already checked receiver type.
    pub(in crate::check) receiver_ty: &'a Ty,
    /// Called routine name.
    pub(in crate::check) name: &'a str,
    /// Explicit arguments after the receiver.
    pub(in crate::check) args: &'a [Expr],
    /// Diagnostic span of the call.
    pub(in crate::check) span: Span,
    /// Span recorded for the call target.
    pub(in crate::check) call_span: Span,
    /// Accept procedures (statement position or `go`).
    pub(in crate::check) allow_procedure_result: bool,
}
