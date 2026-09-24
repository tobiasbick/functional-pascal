//! Synthetic task entry functions for receiver calls to standard intrinsics.
//!
//! **Documentation:** `docs/pascal/language/functions/fluent-calls.md`

use std::collections::{BTreeMap, BTreeSet};

use fpas_ir::{Constant, Function, FunctionId, IntrinsicId, Operation, Terminator};
use fpas_parser::{Expr, PostfixOperation};
use fpas_sema::AnalysisMetadata;

use crate::CompileError;
use crate::lowering::context::{
    BoundMethodTarget, FunctionInput, GlobalBinding, LoweringContext, ParameterInput, unsupported,
};
use crate::lowering::types;

use super::{ClosureRegistry, IntrinsicTaskRoutine};

impl ClosureRegistry<'_> {
    /// Registers an intrinsic receiver call that is the body of a `go` expression.
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
        let Some(target) = metadata.fluent_calls.get(&key) else {
            return Ok(());
        };
        let Some(intrinsic) =
            crate::intrinsic_catalog::resolve(&target.name, Some(&target.receiver_ty))
        else {
            return Ok(());
        };
        let span = expression.span();
        let mut parameters = vec![types.intern(&target.receiver_ty, span.line, span.column)?];
        for argument in args {
            let ty = metadata
                .expr_types
                .get(&fpas_sema::expr_lookup_key(argument))
                .ok_or_else(|| unsupported(argument.span(), "intrinsic task argument type"))?;
            parameters.push(types.intern(ty, span.line, span.column)?);
        }
        let result = types.intern(&target.result_ty, span.line, span.column)?;
        let value_type = types.function_type(parameters.clone(), result, span)?;
        let id = FunctionId::new(self.next_id);
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or_else(|| unsupported(span, "function identifier overflow"))?;
        self.intrinsic_task_targets.insert(
            key,
            BoundMethodTarget {
                function: id,
                value_type,
            },
        );
        self.intrinsic_task_routines.push(IntrinsicTaskRoutine {
            id,
            name: format!("$intrinsic_task_{}", id.get()),
            intrinsic,
            parameters,
            result,
            span,
            owner,
        });
        Ok(())
    }

    /// Lowers an intrinsic task entry with the original call arguments as parameters.
    pub(in crate::lowering) fn lower_intrinsic_task(
        &self,
        routine: &IntrinsicTaskRoutine,
        metadata: &AnalysisMetadata,
        types: &mut types::TypeTable,
        globals: &BTreeMap<String, GlobalBinding>,
        constants: &BTreeMap<String, Constant>,
    ) -> Result<(Function, types::TypeTable), CompileError> {
        let parameters = routine
            .parameters
            .iter()
            .enumerate()
            .map(|(index, ty)| ParameterInput {
                name: format!("argument{index}"),
                ty: *ty,
                declaration: None,
            })
            .collect::<Vec<_>>();
        let mut context = LoweringContext::new(FunctionInput {
            name: &routine.name,
            source_name: &self.source_name,
            id: routine.id,
            result: routine.result,
            parameters: &parameters,
            captures: &[],
            globals: globals.clone(),
            constants: constants.clone(),
            metadata,
            callables: self.callables.clone(),
            closure_targets: self.targets.clone(),
            bound_method_targets: self.bound_targets.clone(),
            intrinsic_task_targets: self.intrinsic_task_targets.clone(),
            cell_names: BTreeSet::new(),
            type_table: types.clone(),
        })?;
        let mut arguments = parameters
            .iter()
            .map(|parameter| context.read_named_local(&parameter.name, routine.span))
            .collect::<Result<Vec<_>, _>>()?;
        if matches!(
            routine.intrinsic,
            fpas_bytecode::Intrinsic::Str(fpas_bytecode::StrIntrinsic::Format)
        ) {
            let count = i64::try_from(arguments.len().saturating_sub(1))
                .map_err(|_| unsupported(routine.span, "format argument count overflow"))?;
            arguments.push(context.emit_value(
                Operation::Const(Constant::Integer(count)),
                types::INTEGER,
                routine.span,
            )?);
        }
        context.record_call_arguments(arguments.len(), routine.span)?;
        let result = context.emit_value(
            Operation::Intrinsic {
                intrinsic: IntrinsicId::new(u32::from(u16::from(routine.intrinsic))),
                arguments,
            },
            routine.result,
            routine.span,
        )?;
        if routine.result == types::UNIT {
            context.terminate(Terminator::Return(None))?;
        } else {
            context.terminate(Terminator::Return(Some(result)))?;
        }
        context.finish(routine.span)
    }
}
