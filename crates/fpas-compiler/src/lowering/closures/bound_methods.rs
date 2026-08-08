//! Bound-method thunk discovery and lowering.

use std::collections::{BTreeMap, BTreeSet};

use fpas_ir::{CaptureKind, Function, FunctionId};
use fpas_sema::AnalysisMetadata;

use crate::CompileError;

use super::{BoundMethodRoutine, ClosureRegistry};
use crate::lowering::context::{
    BoundMethodTarget, CaptureInput, FunctionInput, GlobalBinding, LoweringContext, unsupported,
};
use crate::lowering::types;

impl ClosureRegistry<'_> {
    pub(in crate::lowering) fn lower_bound(
        &self,
        routine: &BoundMethodRoutine,
        metadata: &AnalysisMetadata,
        types: &mut types::TypeTable,
        globals: &BTreeMap<String, GlobalBinding>,
        constants: &BTreeMap<String, fpas_ir::Constant>,
    ) -> Result<(Function, types::TypeTable), CompileError> {
        let parameters = routine
            .parameters
            .iter()
            .enumerate()
            .map(|(index, ty)| (format!("argument{index}"), *ty))
            .collect::<Vec<_>>();
        let captures = vec![CaptureInput {
            name: "__bound_self".to_string(),
            ty: routine.receiver_ty,
            storage_ty: routine.receiver_ty,
            kind: CaptureKind::Value,
        }];
        let mut context = LoweringContext::new(FunctionInput {
            name: &routine.name,
            id: routine.id,
            result: routine.result,
            parameters: &parameters,
            captures: &captures,
            globals: globals.clone(),
            constants: constants.clone(),
            metadata,
            callables: self.callables.clone(),
            closure_targets: self.targets.clone(),
            bound_method_targets: self.bound_targets.clone(),
            cell_names: BTreeSet::new(),
            type_table: types.clone(),
        })?;
        let receiver = context.read_capture("__bound_self", routine.span)?;
        let mut arguments = vec![receiver];
        for (name, _) in &parameters {
            arguments.push(context.read_named_local(name, routine.span)?);
        }
        context.record_call_arguments(arguments.len(), routine.span)?;
        let result = context.emit_value(
            fpas_ir::Operation::CallDirect {
                function: routine.target,
                arguments,
            },
            routine.result,
            routine.span,
        )?;
        if matches!(types.kind(routine.result), Some(fpas_ir::IrType::Unit)) {
            context.terminate(fpas_ir::Terminator::Return(None))?;
        } else {
            context.terminate(fpas_ir::Terminator::Return(Some(result)))?;
        }
        context.finish(routine.span)
    }

    pub(super) fn register_bound_method(
        &mut self,
        key: usize,
        info: &fpas_sema::BoundMethodInfo,
        span: fpas_lexer::Span,
        types: &mut types::TypeTable,
    ) -> Result<(), CompileError> {
        if self.bound_targets.contains_key(&key) {
            return Ok(());
        }
        let callable = self
            .callables
            .get(&info.qualified_name.to_ascii_lowercase())
            .cloned()
            .ok_or_else(|| unsupported(span, "bound method target"))?;
        let Some((&receiver_ty, parameters)) = callable.parameters.split_first() else {
            return Err(unsupported(span, "bound method receiver"));
        };
        if parameters.len() != usize::from(info.visible_arity) {
            return Err(unsupported(span, "bound method arity"));
        }
        let value_type = types.function_type(parameters.to_vec(), callable.result, span)?;
        let id = FunctionId::new(self.next_id);
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or_else(|| unsupported(span, "function identifier overflow"))?;
        self.bound_targets.insert(
            key,
            BoundMethodTarget {
                function: id,
                value_type,
            },
        );
        self.bound_routines.push(BoundMethodRoutine {
            id,
            name: format!("$bound_{}_{}", info.qualified_name, id.get()),
            target: callable.function,
            receiver_ty,
            parameters: parameters.to_vec(),
            result: callable.result,
            span,
        });
        Ok(())
    }
}
