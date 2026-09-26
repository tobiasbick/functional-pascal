//! Receiver-based lookup for free routines and first-class callables.
//!
//! **Documentation:** `docs/pascal/language/functions/fluent-calls.md`

use super::super::super::Checker;
use crate::check::FluentCallTarget;
use crate::scope::{Symbol, SymbolKind, canonical_symbol_name};
use crate::types::Ty;
use fpas_diagnostics::codes::{
    SEMA_AMBIGUOUS_IMPORTED_NAME, SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME,
};
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
                    .any(|(member, _)| member.eq_ignore_ascii_case(name))
                || record
                    .properties
                    .iter()
                    .any(|(member, _)| member.eq_ignore_ascii_case(name))
                || record
                    .events
                    .iter()
                    .any(|(member, _)| member.eq_ignore_ascii_case(name));
            if has_member {
                if record
                    .fields
                    .iter()
                    .any(|(member, _)| member.eq_ignore_ascii_case(name))
                    || record
                        .properties
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
        let receiver_reads = self
            .property_reads
            .remove(&crate::designator_lookup_key(designator))
            .unwrap_or_default();
        Some(self.check_fluent_call(FluentCall {
            call_key,
            receiver: &receiver,
            receiver_ty: &receiver_ty,
            name,
            args,
            span,
            call_span: *name_span,
            allow_procedure_result,
            receiver_reads,
        }))
    }

    /// Checks a call through a callable record field or property.
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
                let inferred = self.check_function_call_args(name, signature, args, span);
                Self::substitute_type_params(&signature.return_type, &inferred)
            }
            Ty::Procedure(signature) => {
                self.check_procedure_call_args(name, signature, args, span);
                Ty::Unit
            }
            _ => {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!("Record member `{name}` is not callable"),
                    "Use a callable field or property, or call a qualified free routine.",
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

    /// Selects and checks a receiver call using its first explicit parameter.
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
            receiver_reads,
        } = call;
        if receiver_ty.is_error() {
            self.check_args_only(args);
            return Ty::Error;
        }
        let Some((target_name, symbol)) = self.select_fluent_target(name, receiver_ty, span) else {
            self.check_args_only(args);
            return Ty::Error;
        };
        let mut all_args = Vec::with_capacity(args.len() + 1);
        all_args.push(receiver);
        all_args.extend(args.iter());
        self.prechecked_receivers
            .insert(Self::expr_lookup_key(receiver), receiver_ty.clone());

        let result = if symbol.kind == SymbolKind::BuiltinStd {
            crate::std_registry::check_builtin_std_call_refs(self, &target_name, &all_args, span)
        } else {
            match &symbol.ty {
                Ty::Function(signature) => {
                    let inferred = self.check_fluent_function_call_args(
                        &target_name,
                        signature,
                        &all_args,
                        span,
                    );
                    Self::substitute_type_params(&signature.return_type, &inferred)
                }
                Ty::Procedure(signature) => {
                    self.check_fluent_procedure_call_args(&target_name, signature, &all_args, span);
                    Ty::Unit
                }
                _ => {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!("`{target_name}` is not callable"),
                        "Choose a function, procedure, or callable value with a first parameter.",
                        span,
                    );
                    self.check_args_only(args);
                    Ty::Error
                }
            }
        };
        self.prechecked_receivers
            .remove(&Self::expr_lookup_key(receiver));

        if result == Ty::Unit && !allow_procedure_result {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("`{target_name}` does not return a value"),
                "Use this call as the final operation of a statement.",
                span,
            );
            return Ty::Error;
        }
        self.fluent_calls.insert(
            call_key,
            FluentCallTarget {
                name: target_name,
                receiver_reads,
                receiver_ty: receiver_ty.clone(),
                result_ty: result.clone(),
                call_span,
            },
        );
        result
    }

    fn select_fluent_target(
        &mut self,
        name: &str,
        receiver_ty: &Ty,
        span: Span,
    ) -> Option<(String, Symbol)> {
        let key = canonical_symbol_name(name);
        if let Some((scope, symbol)) = self.scopes.lookup_with_scope(name)
            && (scope > 0
                || !self.std_short_alias_keys.contains(&key)
                    && !self.source_short_alias_keys.contains(&key))
        {
            let symbol = symbol.clone();
            if self.fluent_symbol_accepts(name, &symbol, receiver_ty) {
                return Some((name.to_string(), symbol));
            }
            self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!("`{name}` cannot be called with receiver type `{receiver_ty}`"),
                    "The nearest binding must be callable and accept the receiver as its first parameter.",
                    span,
                );
            return None;
        }

        let mut matches = self
            .imported_candidates
            .get(&key)
            .into_iter()
            .flatten()
            .filter_map(|qualified| {
                let symbol = self.scopes.lookup(qualified)?.clone();
                self.fluent_symbol_accepts(qualified, &symbol, receiver_ty)
                    .then_some((qualified.clone(), symbol))
            })
            .collect::<Vec<_>>();
        matches.sort_by(|left, right| {
            left.0
                .to_ascii_lowercase()
                .cmp(&right.0.to_ascii_lowercase())
        });
        matches.dedup_by(|left, right| left.0.eq_ignore_ascii_case(&right.0));
        match matches.len() {
            1 => matches.pop(),
            0 => {
                self.error_with_code(
                    SEMA_UNKNOWN_NAME,
                    format!("No visible `{name}` accepts receiver type `{receiver_ty}`"),
                    "Import a unit that exports a matching callable, or call a qualified routine explicitly.",
                    span,
                );
                None
            }
            _ => {
                self.error_with_code(
                    SEMA_AMBIGUOUS_IMPORTED_NAME,
                    format!("Ambiguous receiver call `.{name}(...)` for `{receiver_ty}`"),
                    format!(
                        "Matching imported callables: {}. Use a qualified ordinary call.",
                        matches
                            .iter()
                            .map(|entry| entry.0.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    span,
                );
                None
            }
        }
    }

    fn fluent_symbol_accepts(&self, name: &str, symbol: &Symbol, receiver_ty: &Ty) -> bool {
        if symbol.kind == SymbolKind::BuiltinStd {
            return crate::std_registry::builtin_accepts_receiver(name, receiver_ty)
                .unwrap_or(false);
        }
        let first = match &symbol.ty {
            Ty::Function(signature) => signature.params.first(),
            Ty::Procedure(signature) => signature.params.first(),
            _ => None,
        };
        first.is_some_and(|param| first_param_accepts(&param.ty, receiver_ty))
    }
}

fn first_param_accepts(expected: &Ty, receiver: &Ty) -> bool {
    match (expected, receiver) {
        (Ty::GenericParam(_, constraint), actual) => {
            constraint.is_none_or(|constraint| constraint.satisfied_by(actual))
        }
        (Ty::Array(left), Ty::Array(right))
        | (Ty::Channel(left), Ty::Channel(right))
        | (Ty::Option(left), Ty::Option(right))
        | (Ty::Task(left), Ty::Task(right)) => first_param_accepts(left, right),
        (Ty::Dict(left_key, left_value), Ty::Dict(right_key, right_value))
        | (Ty::Result(left_key, left_value), Ty::Result(right_key, right_value)) => {
            first_param_accepts(left_key, right_key) && first_param_accepts(left_value, right_value)
        }
        _ => expected.compatible_with(receiver),
    }
}

/// One receiver call `Receiver.Name(Args)` resolved through a free routine.
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
    /// Property getter reads needed while evaluating the receiver.
    pub(in crate::check) receiver_reads: Vec<crate::check::PropertyReadInfo>,
}
