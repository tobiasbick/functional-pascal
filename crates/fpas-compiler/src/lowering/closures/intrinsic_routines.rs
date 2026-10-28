//! Shared lowering of resolved intrinsic task entries and conversion function values.
//!
//! **Documentation:** `docs/pascal/language/functions/fluent-calls.md`

use std::collections::{BTreeMap, BTreeSet};

use fpas_ir::{Constant, Function, IntrinsicId, Operation, Terminator};
use fpas_sema::AnalysisMetadata;

use crate::CompileError;
use crate::lowering::context::{
    FunctionInput, GlobalBinding, LoweringContext, ParameterInput, unsupported,
};
use crate::lowering::types;

use super::{ClosureRegistry, IntrinsicRoutine};

impl ClosureRegistry<'_> {
    /// Lowers a resolved intrinsic entry with explicit arguments as parameters.
    pub(in crate::lowering) fn lower_intrinsic_routine(
        &self,
        routine: &IntrinsicRoutine,
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
                type_name: None,
                ty: *ty,
                declaration: None,
                reference: false,
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
            intrinsic_targets: self.intrinsic_targets.clone(),
            cell_names: BTreeSet::new(),
            type_table: types.clone(),
        })?;
        let mut arguments = parameters
            .iter()
            .map(|parameter| context.read_named_local(&parameter.name, routine.span))
            .collect::<Result<Vec<_>, _>>()?;
        if let Some(native) = &routine.native {
            let count = arguments.len().saturating_sub(1);
            let result = context.lower_native_values(
                &native.name,
                &native.receiver_ty,
                arguments,
                routine.result,
                count,
                routine.span,
            )?;
            context.terminate(if routine.result == types::UNIT {
                Terminator::Return(None)
            } else {
                Terminator::Return(Some(result))
            })?;
            return context.finish(routine.span);
        }
        let selected = routine
            .intrinsic
            .ok_or_else(|| unsupported(routine.span, "resolved intrinsic entry"))?;
        if matches!(
            selected,
            fpas_code::Intrinsic::Console(
                fpas_code::ConsoleIntrinsic::Write | fpas_code::ConsoleIntrinsic::WriteLn
            )
        ) {
            let write = fpas_code::Intrinsic::Console(fpas_code::ConsoleIntrinsic::Write);
            let write_ln = fpas_code::Intrinsic::Console(fpas_code::ConsoleIntrinsic::WriteLn);
            if arguments.is_empty() && selected == write_ln {
                arguments.push(context.emit_value(
                    Operation::Const(Constant::String(String::new())),
                    types::STRING,
                    routine.span,
                )?);
            }
            let last = arguments.len().saturating_sub(1);
            for (index, argument) in arguments.into_iter().enumerate() {
                let intrinsic = if index == last && selected == write_ln {
                    write_ln
                } else {
                    write
                };
                context.record_call_arguments(1, routine.span)?;
                context.emit_value(
                    Operation::Intrinsic {
                        intrinsic: IntrinsicId::new(u32::from(u16::from(intrinsic))),
                        arguments: vec![argument],
                    },
                    types::UNIT,
                    routine.span,
                )?;
            }
            context.terminate(Terminator::Return(None))?;
            return context.finish(routine.span);
        }
        context.can_spawn_tasks |= selected.starts_task();
        if matches!(
            selected,
            fpas_code::Intrinsic::Str(fpas_code::StrIntrinsic::Format)
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
                intrinsic: IntrinsicId::new(u32::from(u16::from(selected))),
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
