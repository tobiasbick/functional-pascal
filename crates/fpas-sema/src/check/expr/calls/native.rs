//! Argument mapping and factory checking for fixed built-in type operations.

use super::super::super::Checker;
use crate::check::calls::CallTarget;
use crate::std_registry::{NativeLowering, NativeOperation};
use crate::types::{ParamTy, Ty};
use fpas_lexer::Span;
use fpas_parser::{Designator, DesignatorPart, Expr};

impl Checker {
    /// Checks a keyword-owned factory without adding a lexical callable symbol.
    pub(in crate::check) fn try_check_native_factory(
        &mut self,
        key: usize,
        designator: &Designator,
        args: &[Expr],
        span: Span,
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
        let result = self.check_native_arguments(operation, None, args, span);
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
    ) -> Ty {
        let mut signature = (operation.signature)();
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
        // Type-check named argument values in written order before the implementation
        // inspects them in parameter order. Retain their original AST identities.
        let named = args.iter().any(|arg| arg.argument_name().is_some());
        let mut cached = Vec::new();
        if named {
            for arg in args {
                let value = arg.argument_value();
                if matches!(value, Expr::VarArgument { .. }) {
                    continue;
                }
                let ty = self.check_expr(value);
                let key = Self::expr_lookup_key(value);
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
        let dispatch = if operation.lowering == NativeLowering::IsEmpty {
            operation.implementation.replace("IsEmpty", "Length")
        } else {
            operation.implementation.into()
        };
        let result = if operation.polymorphic() {
            crate::std_registry::check_builtin_std_call_refs(
                self,
                &dispatch,
                &all_args,
                receiver.is_some(),
                span,
            )
        } else {
            if let Some(receiver) = receiver {
                let receiver_ty = self.check_expr(receiver);
                signature
                    .params
                    .insert(0, ParamTy::value("Self", receiver_ty));
            }
            // IsEmpty reuses Length with an integer result before returning boolean.
            if operation.lowering == NativeLowering::IsEmpty {
                *signature.return_type = Ty::Integer;
            }
            let inferred =
                self.check_function_call_refs(operation.name, &signature, &all_args, span);
            Self::substitute_type_params(&signature.return_type, &inferred)
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
}
