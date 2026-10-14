//! Synthetic task entry functions for resolved standard intrinsic calls.
//!
//! **Documentation:** `docs/pascal/language/functions/fluent-calls.md`

use fpas_ir::FunctionId;
use fpas_parser::{Expr, PostfixOperation};
use fpas_sema::AnalysisMetadata;

use crate::CompileError;
use crate::lowering::context::{BoundMethodTarget, unsupported};
use crate::lowering::types;

use super::{ClosureRegistry, IntrinsicRoutine};

impl ClosureRegistry<'_> {
    /// Registers an intrinsic call that is the body of a `go` expression.
    pub(super) fn register_intrinsic_task(
        &mut self,
        expression: &Expr,
        owner: FunctionId,
        metadata: &AnalysisMetadata,
        types: &mut types::TypeTable,
    ) -> Result<(), CompileError> {
        let (key, args) = match expression {
            Expr::Call { args, .. } => (fpas_sema::expr_lookup_key(expression), args.as_slice()),
            Expr::Postfix { operations, .. } => {
                let Some(last) = operations.last() else {
                    return Ok(());
                };
                let PostfixOperation::MethodCall { args, .. } = last else {
                    return Ok(());
                };
                (
                    fpas_sema::postfix_operation_lookup_key(last),
                    args.as_slice(),
                )
            }
            _ => return Ok(()),
        };
        let span = expression.span();
        let ordered_args = if let Some(order) = args.first().and_then(|arg| {
            metadata
                .named_argument_orders
                .get(&fpas_sema::expr_lookup_key(arg))
        }) {
            order.iter().map(|&index| &args[index]).collect::<Vec<_>>()
        } else {
            args.iter().collect::<Vec<_>>()
        };
        let (intrinsic, result_ty, receiver_ty) =
            if let Some(target) = metadata.fluent_calls.get(&key) {
                let intrinsic =
                    crate::intrinsic_catalog::resolve(&target.name, Some(&target.receiver_ty));
                (intrinsic, &target.result_ty, Some(&target.receiver_ty))
            } else if let Some(name) = metadata.intrinsic_calls.get(&key) {
                let first_ty = ordered_args.first().and_then(|arg| {
                    metadata
                        .expr_types
                        .get(&fpas_sema::expr_lookup_key(arg.argument_value()))
                });
                let Some(intrinsic) = crate::intrinsic_catalog::resolve(name, first_ty) else {
                    return Ok(());
                };
                let result_ty = metadata
                    .expr_types
                    .get(&fpas_sema::expr_lookup_key(expression))
                    .ok_or_else(|| unsupported(span, "intrinsic task result type"))?;
                (Some(intrinsic), result_ty, None)
            } else {
                return Ok(());
            };
        let mut parameters = Vec::new();
        if let Some(receiver_ty) = receiver_ty {
            parameters.push(types.intern(receiver_ty, span.line, span.column)?);
        }
        for argument in ordered_args {
            let argument = argument.argument_value();
            let ty = metadata
                .expr_types
                .get(&fpas_sema::expr_lookup_key(argument))
                .ok_or_else(|| unsupported(argument.span(), "intrinsic task argument type"))?;
            parameters.push(types.intern(ty, span.line, span.column)?);
        }
        let result = types.intern(result_ty, span.line, span.column)?;
        let value_type = types.function_type(parameters.clone(), result, span)?;
        let id = FunctionId::new(self.next_id);
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or_else(|| unsupported(span, "function identifier overflow"))?;
        self.intrinsic_targets.insert(
            key,
            BoundMethodTarget {
                function: id,
                value_type,
            },
        );
        self.intrinsic_routines.push(IntrinsicRoutine {
            id,
            name: format!("$intrinsic_task_{}", id.get()),
            intrinsic,
            native: metadata.fluent_calls.get(&key).cloned(),
            parameters,
            result,
            span,
            owner,
        });
        Ok(())
    }
}
