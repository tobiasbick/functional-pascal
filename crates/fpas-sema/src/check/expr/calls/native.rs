//! Argument mapping and factory checking for fixed built-in type operations.

use super::super::super::Checker;
use crate::check::calls::CallTarget;
use crate::std_registry::{NativeLowering, NativeOperation};
use crate::types::{FunctionTy, GenericParamDef, Ty};
use fpas_lexer::Span;
use fpas_parser::{Designator, DesignatorPart, Expr};
use std::collections::{HashMap, HashSet};

impl Checker {
    /// Checks a keyword-owned factory without adding a lexical callable symbol.
    pub(in crate::check) fn try_check_native_factory(
        &mut self,
        key: usize,
        designator: &Designator,
        args: &[Expr],
        span: Span,
        expected: Option<&Ty>,
    ) -> Option<Ty> {
        let [
            DesignatorPart::Ident(owner, _),
            DesignatorPart::Ident(name, _),
        ] = designator.parts.as_slice()
        else {
            return None;
        };
        if !owner.eq_ignore_ascii_case("string") && !owner.eq_ignore_ascii_case("array") {
            return None;
        }
        let Some(operation) = crate::std_registry::native_factory(owner, name) else {
            self.error_with_code(
                fpas_diagnostics::codes::SEMA_UNKNOWN_NAME,
                format!("Type `{owner}` has no factory `{name}`"),
                "The factories are `string.Chr(N)` and `array.Fill(Value, Count)`.",
                span,
            );
            self.check_args_only(args);
            return Some(Ty::Error);
        };
        self.native_capabilities.remove(&key);
        let result = self.check_native_arguments(operation, None, args, span, expected);
        let capability = self.native_result_capability(
            operation,
            crate::TransferCapability::Transferable,
            args,
            &result,
        );
        self.native_capabilities.insert(key, capability);
        self.intrinsic_calls
            .insert(key, operation.implementation.into());
        Some(result)
    }

    /// Maps explicit parameters independently of the implicit receiver and checks the existing implementation.
    pub(in crate::check) fn check_native_arguments(
        &mut self,
        operation: &NativeOperation,
        receiver: Option<&Expr>,
        args: &[Expr],
        span: Span,
        expected: Option<&Ty>,
    ) -> Ty {
        if matches!(operation.lowering, NativeLowering::InPlace(_)) {
            let Some(receiver) = receiver else {
                return Ty::Error;
            };
            let ty = self.check_expr(receiver);
            let Expr::Designator(designator) = receiver else {
                self.error_with_code(
                    fpas_diagnostics::codes::SEMA_INVALID_VAR_ARGUMENT,
                    format!("`{}` requires a writable receiver", operation.name),
                    format!("Use a var variable, field or array element, or use the value operation without InPlace: `Value := Value.{}(…)`.", operation.name.trim_end_matches("InPlace")),
                    receiver.span(),
                );
                return Ty::Error;
            };
            let (_, root) = self.check_var_storage(designator, ty);
            let Some(root) = root else {
                return Ty::Error;
            };
            self.reject_var_argument_aliases(&[root], span);
        }
        let signature = self.native_call_signature(operation, receiver, span);
        if !signature.variadic
            && args.iter().all(|arg| arg.argument_name().is_none())
            && args.len() != signature.params.len()
        {
            self.error_with_code(
                fpas_diagnostics::codes::SEMA_WRONG_ARGUMENT_COUNT,
                format!(
                    "Native operation `{}` expects {} explicit argument(s), got {}",
                    operation.name,
                    signature.params.len(),
                    args.len()
                ),
                "Pass the catalog's explicit arguments; an instance receiver is implicit.",
                span,
            );
            self.check_args_only(args);
            return Ty::Error;
        }
        let Some(ordered) = self.order_call_arguments(
            operation.name,
            CallTarget::Routine,
            &signature.params,
            signature.variadic,
            &args.iter().collect::<Vec<_>>(),
        ) else {
            return Ty::Error;
        };
        let inferred = self.check_function_call_refs(
            operation.name,
            &signature,
            &args.iter().collect::<Vec<_>>(),
            span,
            expected,
        );
        let checked_result = self.check_call_result_type(
            &signature.return_type,
            &signature.type_params,
            &inferred,
            span,
        );
        // Existing operation-specific validation consumes the checked original expressions.
        let mut cached = Vec::new();
        for arg in args {
            let value = arg.argument_value();
            if matches!(value, Expr::VarArgument { .. }) {
                continue;
            }
            let key = Self::expr_lookup_key(value);
            if let Some(ty) = self.expr_types.get(&key).cloned() {
                self.expr_types
                    .insert(Self::expr_lookup_key(arg), ty.clone());
                self.prechecked_receivers.insert(key, ty);
                cached.push(key);
            }
        }
        let mut all_args = Vec::with_capacity(ordered.len() + usize::from(receiver.is_some()));
        if let Some(receiver) = receiver {
            all_args.push(receiver);
        }
        all_args.extend(ordered);
        let dispatch = match operation.lowering {
            NativeLowering::IsEmpty => operation.implementation.replace("IsEmpty", "Length"),
            NativeLowering::InPlace(base) => base.into(),
            _ => operation.implementation.into(),
        };
        let result = if operation.polymorphic() {
            let validated = crate::std_registry::check_builtin_std_call_refs(
                self,
                &dispatch,
                &all_args,
                receiver.is_some(),
                span,
            );
            if validated.is_error() {
                Ty::Error
            } else {
                if matches!(operation.lowering, NativeLowering::InPlace(_))
                    && matches!(operation.name, "MapInPlace" | "FlatMapInPlace")
                    && receiver.is_some_and(|receiver| {
                        !validated.compatible_with(&self.check_expr(receiver))
                    })
                {
                    self.error_with_code(
                        fpas_diagnostics::codes::SEMA_TYPE_MISMATCH,
                        "InPlace callbacks must preserve the receiver's exact type",
                        "Use Map or FlatMap without InPlace when changing the element or value type.",
                        span,
                    );
                    return Ty::Error;
                }
                checked_result
            }
        } else {
            checked_result
        };
        for key in cached {
            self.prechecked_receivers.remove(&key);
        }
        if operation.lowering == NativeLowering::IsEmpty && result != Ty::Error {
            Ty::Boolean
        } else {
            result
        }
    }

    fn native_call_signature(
        &mut self,
        operation: &NativeOperation,
        receiver: Option<&Expr>,
        span: Span,
    ) -> FunctionTy {
        let receiver_ty = receiver.map(|value| self.check_expr(value));
        self.native_signature_for_type(operation, receiver_ty.as_ref(), span)
    }

    pub(in crate::check) fn native_signature_for_type(
        &mut self,
        operation: &NativeOperation,
        receiver_ty: Option<&Ty>,
        span: Span,
    ) -> FunctionTy {
        let mut signature = operation.signature_for(receiver_ty);
        let raw = Ty::Function((operation.signature)());
        let mut parameters = HashMap::new();
        self.collect_type_param_bindings(&raw, &raw, &mut parameters, &mut HashSet::new(), span);
        let fixed: &[&str] = match receiver_ty {
            Some(Ty::Array(_) | Ty::Option(_)) => &["t"],
            Some(Ty::Dict(_, _)) => &["k", "v"],
            Some(Ty::Result(_, _)) => &["t", "e"],
            _ => &[],
        };
        signature.type_params = parameters
            .into_iter()
            .filter_map(|(key, ty)| {
                if fixed.contains(&key.as_str()) {
                    return None;
                }
                let Ty::GenericParam(name, constraint) = ty else {
                    return None;
                };
                Some(GenericParamDef { name, constraint })
            })
            .collect();
        signature
            .type_params
            .sort_by(|left, right| left.name.cmp(&right.name));
        signature
    }
}
