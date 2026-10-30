//! Callable values for standard routines and built-in receiver operations.
//! See docs/pascal/language/functions/first-class.md.

use super::Checker;
use crate::{
    scope::SymbolKind,
    std_registry::{NativeLowering, NativeOperation},
    types::{FunctionTy, ParamMode, ParamTy, ProcedureTy, Ty, needs_expected_type},
};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::{Designator, DesignatorPart, Expr};
use std::collections::HashMap;

/// A checked native function value, optionally capturing one receiver by value.
#[derive(Debug, Clone, PartialEq)]
pub struct NativeCallable {
    /// Canonical intrinsic or native-catalog implementation name.
    pub name: String,
    /// Captured receiver type; absent for qualified routines and factories.
    pub receiver_ty: Option<Ty>,
    /// Designator parts evaluated to capture the receiver; zero for postfix fields.
    pub receiver_part_count: usize,
}

/// Native callable values keyed by designator or postfix-field identity.
pub type NativeCallableMap = HashMap<usize, NativeCallable>;

impl Checker {
    pub(in crate::check) fn try_native_callable_designator(
        &mut self,
        designator: &Designator,
        expected: Option<&Ty>,
    ) -> Option<Ty> {
        let name = Self::resolve_designator_name(designator);
        let symbol = self.scopes.lookup(&name).cloned();
        if let Some(symbol) = symbol
            && matches!(
                symbol.kind,
                SymbolKind::Function | SymbolKind::Procedure | SymbolKind::BuiltinStd
            )
            && let Some(canonical) = self.scopes.imports.original_source_name(&name)
            && canonical.rsplit_once('.').is_some_and(|(unit, _)| {
                fpas_builtins::STD_UNITS_INTRINSIC
                    .iter()
                    .any(|known| known.eq_ignore_ascii_case(unit))
            })
        {
            let canonical = canonical.to_owned();
            self.check_designator_expr(designator);
            let mut ty = self.native_callable_signature(
                symbol.ty,
                expected,
                symbol.kind == SymbolKind::BuiltinStd,
                designator.span,
            );
            if !ty.is_error() {
                if symbol.kind == SymbolKind::BuiltinStd {
                    self.validate_native_callable(&canonical, &mut ty, None, designator.span);
                }
                self.native_values.insert(
                    crate::designator_lookup_key(designator),
                    NativeCallable {
                        name: canonical,
                        receiver_ty: None,
                        receiver_part_count: 0,
                    },
                );
                if let Some(DesignatorPart::Ident(_, span)) = designator.parts.last() {
                    self.path_types
                        .insert((span.source_id, span.offset), ty.clone());
                }
            }
            return Some(ty);
        }
        if let [
            DesignatorPart::Ident(owner, _),
            DesignatorPart::Ident(member, _),
        ] = designator.parts.as_slice()
            && let Some(operation) = crate::native_factory(owner, member)
        {
            let signature = self.native_signature_for_type(operation, None, designator.span);
            let mut ty = self.native_callable_signature(
                Ty::Function(signature),
                expected,
                false,
                designator.span,
            );
            if operation.polymorphic() {
                self.validate_native_callable(
                    operation.implementation,
                    &mut ty,
                    None,
                    designator.span,
                );
            }
            self.native_values.insert(
                crate::designator_lookup_key(designator),
                NativeCallable {
                    name: operation.implementation.into(),
                    receiver_ty: None,
                    receiver_part_count: 0,
                },
            );
            return Some(ty);
        }
        let (DesignatorPart::Ident(member, member_span), prefix) = designator.parts.split_last()?
        else {
            return None;
        };
        if prefix.is_empty()
            || self.designator_has_unit_prefix(designator)
            || self.designator_parts_denote_type(prefix)
        {
            return None;
        }
        if let Some(DesignatorPart::Ident(root, _)) = prefix.first()
            && self.scopes.lookup(root).is_none()
        {
            return None;
        }
        let receiver = self.check_designator_prefix_expr(designator, prefix.len());
        let receiver = self.resolve_visible_type(&receiver);
        let operation = crate::native_operation(&receiver, member)?;
        self.member_receivers.insert(
            (member_span.source_id, member_span.offset),
            receiver.clone(),
        );
        let ty = self.check_native_callable_receiver(
            operation,
            receiver,
            crate::designator_lookup_key(designator),
            prefix.len(),
            designator.span,
            expected,
        );
        self.path_types
            .insert((member_span.source_id, member_span.offset), ty.clone());
        Some(ty)
    }

    pub(in crate::check) fn check_native_callable_receiver(
        &mut self,
        operation: &NativeOperation,
        receiver: Ty,
        key: usize,
        receiver_part_count: usize,
        span: Span,
        expected: Option<&Ty>,
    ) -> Ty {
        if matches!(operation.lowering, NativeLowering::InPlace(_)) {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                "An InPlace operation cannot capture its receiver by value",
                "Call InPlace directly on writable storage, or bind the value operation without InPlace.",
                span,
            );
            return Ty::Error;
        }
        let signature = self.native_signature_for_type(operation, Some(&receiver), span);
        let mut ty = self.native_callable_signature(Ty::Function(signature), expected, false, span);
        if operation.polymorphic() {
            let dispatch = if operation.lowering == NativeLowering::IsEmpty {
                operation.implementation.replace("IsEmpty", "Length")
            } else {
                operation.implementation.into()
            };
            self.validate_native_callable(&dispatch, &mut ty, Some(&receiver), span);
        }
        self.native_values.insert(
            key,
            NativeCallable {
                name: operation.implementation.into(),
                receiver_ty: Some(receiver),
                receiver_part_count,
            },
        );
        ty
    }

    fn native_callable_signature(
        &mut self,
        mut declared: Ty,
        expected: Option<&Ty>,
        polymorphic: bool,
        span: Span,
    ) -> Ty {
        if let Ty::Function(signature) = &declared
            && *signature.return_type == Ty::Unit
        {
            declared = Ty::Procedure(ProcedureTy {
                type_params: signature.type_params.clone(),
                params: signature.params.clone(),
                variadic: signature.variadic,
            });
        }
        let function = matches!(declared, Ty::Function(_));
        let (params, variadic) = match &mut declared {
            Ty::Function(signature) => (&mut signature.params, &mut signature.variadic),
            Ty::Procedure(signature) => (&mut signature.params, &mut signature.variadic),
            _ => return declared,
        };
        if polymorphic || *variadic {
            let context = match expected {
                Some(Ty::Function(signature)) if function => {
                    Some((&signature.params, Some(signature.return_type.as_ref())))
                }
                Some(Ty::Procedure(signature)) if !function => Some((&signature.params, None)),
                _ => None,
            };
            let Some((context, result)) = context else {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    "This native operation needs a concrete callable signature",
                    "Supply a typed function/procedure binding or pass it to a typed callback parameter.",
                    span,
                );
                return Ty::Error;
            };
            if *variadic {
                for parameter in context {
                    self.reject_implicit_distinct_unwrap("native callable", &parameter.ty, span);
                }
            }
            if (!*variadic && params.len() != context.len())
                || params.len() > context.len()
                || context
                    .iter()
                    .any(|parameter| parameter.mode == ParamMode::Var)
            {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    "Callable parameters do not match the native operation",
                    "Use the operation's published argument count and by-value parameter modes.",
                    span,
                );
                return Ty::Error;
            }
            *params = context
                .iter()
                .enumerate()
                .map(|(index, parameter)| {
                    ParamTy::value(
                        params
                            .get(index)
                            .map_or(parameter.name.as_str(), |p| p.name.as_str()),
                        parameter.ty.clone(),
                    )
                })
                .collect();
            *variadic = false;
            if let (Ty::Function(signature), Some(result)) = (&mut declared, result) {
                *signature.return_type = result.clone();
            }
        }
        if let Some(expected) = expected {
            self.specialize_expected_callable(declared, expected, span)
        } else {
            declared
        }
    }

    fn validate_native_callable(
        &mut self,
        name: &str,
        ty: &mut Ty,
        receiver: Option<&Ty>,
        span: Span,
    ) {
        if ty.is_error() {
            return;
        }
        let (params, result) = match &*ty {
            Ty::Function(FunctionTy {
                params,
                return_type,
                ..
            }) => (params, return_type.as_ref()),
            Ty::Procedure(ProcedureTy { params, .. }) => (params, &Ty::Unit),
            _ => return,
        };
        if params
            .iter()
            .any(|parameter| needs_expected_type(&parameter.ty))
        {
            return;
        }
        // Type-only probes share ordinary operation validation without evaluating source
        // expressions or creating source bindings. Remove their identity keys before drop.
        let types = receiver
            .into_iter()
            .chain(params.iter().map(|p| &p.ty))
            .collect::<Vec<_>>();
        let arguments = types.iter().map(|_| Expr::Error(span)).collect::<Vec<_>>();
        for (argument, ty) in arguments.iter().zip(types) {
            let key = Self::expr_lookup_key(argument);
            self.prechecked_receivers.insert(key, ty.clone());
            self.expr_types.insert(key, ty.clone());
        }
        let checked = crate::std_registry::check_builtin_std_call_refs(
            self,
            name,
            &arguments.iter().collect::<Vec<_>>(),
            receiver.is_some(),
            span,
        );
        if receiver.is_none() {
            self.check_type_compat(result, &checked, "native callable result", span);
        }
        for argument in &arguments {
            let key = Self::expr_lookup_key(argument);
            self.prechecked_receivers.remove(&key);
            self.expr_types.remove(&key);
            self.expr_capabilities.remove(&key);
        }
        if receiver.is_none()
            && let Ty::Function(signature) = ty
            && needs_expected_type(&signature.return_type)
            && !needs_expected_type(&checked)
        {
            *signature.return_type = checked;
        }
    }
}
