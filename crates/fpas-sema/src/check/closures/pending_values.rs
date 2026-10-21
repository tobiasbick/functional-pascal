//! Deferred callable provenance follows the inputs actually retained by native values.
//! See `docs/pascal/language/functions/closures.md`.

use crate::{check::Checker, std_registry::NativeOperation, types::Ty};
use fpas_parser::{Expr, PostfixOperation};

impl Checker {
    pub(super) fn pending_native_captures(
        &self,
        operation: &NativeOperation,
        result: &Ty,
        receiver: Vec<usize>,
        args: &[Expr],
    ) -> Vec<usize> {
        let (retains_receiver, parameters) = self.native_retained_inputs(operation, result);
        let mut targets = if retains_receiver {
            receiver
        } else {
            Vec::new()
        };
        for &(name, position) in parameters {
            let argument = if args.iter().any(|arg| arg.argument_name().is_some()) {
                args.iter().find(|arg| {
                    arg.argument_name()
                        .is_some_and(|actual| actual.eq_ignore_ascii_case(name))
                })
            } else {
                args.get(position)
            };
            if let Some(argument) = argument {
                targets.extend(self.pending_expr_captures(argument.argument_value()));
            }
        }
        targets.sort_unstable();
        targets.dedup();
        targets
    }

    pub(super) fn pending_postfix_captures(
        &self,
        base: &Expr,
        operations: &[PostfixOperation],
    ) -> Vec<usize> {
        let mut targets = self.pending_expr_captures(base);
        for operation in operations {
            match operation {
                PostfixOperation::Field { span, .. } | PostfixOperation::Index { span, .. } => {
                    if self
                        .path_types
                        .get(&(span.source_id, span.offset))
                        .is_some_and(|ty| !self.type_can_contain_callable(ty))
                    {
                        targets.clear();
                    }
                }
                PostfixOperation::MethodCall { span, args, .. } => {
                    let key = Self::postfix_operation_lookup_key(operation);
                    if let Some(native) = self.fluent_calls.get(&key).and_then(|target| {
                        crate::std_registry::native_operation_by_implementation(&target.name)
                    }) && let Some(result) =
                        self.postfix_types.get(&(span.source_id, span.offset))
                    {
                        targets = self.pending_native_captures(native, result, targets, args);
                    } else {
                        // Ordinary calls create a new result whose captured values are not
                        // determined by the callee or its receiver's captured values.
                        targets.clear();
                    }
                }
            }
        }
        targets
    }
}
