//! Typed native routine adapters, with an optional immutable receiver capture.

use fpas_ir::FunctionId;
use fpas_sema::{NativeCallable, Ty};

use super::{ClosureRegistry, IntrinsicRoutine};
use crate::{
    CompileError,
    lowering::{
        context::{BoundMethodTarget, unsupported},
        types,
    },
};

impl ClosureRegistry<'_> {
    pub(super) fn register_native_value(
        &mut self,
        key: usize,
        info: &NativeCallable,
        ty: &Ty,
        owner: FunctionId,
        span: fpas_lexer::Span,
        types: &mut types::TypeTable,
    ) -> Result<(), CompileError> {
        if self.intrinsic_targets.contains_key(&key) || self.bound_targets.contains_key(&key) {
            return Ok(());
        }
        let (params, semantic_result) = match ty {
            Ty::Function(signature) => (&signature.params, signature.return_type.as_ref()),
            Ty::Procedure(signature) => (&signature.params, &Ty::Unit),
            _ => return Err(unsupported(span, "native callable signature")),
        };
        let parameters = params
            .iter()
            .map(|parameter| types.intern_param(parameter, span.line, span.column))
            .collect::<Result<Vec<_>, _>>()?;
        let result = types.intern(semantic_result, span.line, span.column)?;
        let value_type = types.function_type(parameters.clone(), result, span)?;
        let receiver = info
            .receiver_ty
            .as_ref()
            .map(|ty| types.intern(ty, span.line, span.column))
            .transpose()?;
        let first = info
            .receiver_ty
            .as_ref()
            .or_else(|| params.first().map(|p| &p.ty));
        let intrinsic = crate::intrinsic_catalog::resolve(&info.name, first);
        if intrinsic.is_none() && receiver.is_none() {
            return Err(unsupported(span, "native callable implementation"));
        }
        let id = FunctionId::new(self.next_id);
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or_else(|| unsupported(span, "function identifier overflow"))?;
        let target = BoundMethodTarget {
            function: id,
            value_type,
        };
        if receiver.is_some() {
            self.bound_targets.insert(key, target);
        } else {
            self.intrinsic_targets.insert(key, target);
        }
        self.intrinsic_routines.push(IntrinsicRoutine {
            id,
            name: if receiver.is_some() {
                format!("$bound_native_{}", id.get())
            } else {
                format!("$native_value_{}", id.get())
            },
            intrinsic,
            native: info
                .receiver_ty
                .as_ref()
                .map(|ty| fpas_sema::FluentCallTarget {
                    name: info.name.clone(),
                    receiver_ty: ty.clone(),
                    result_ty: semantic_result.clone(),
                    call_span: span,
                }),
            receiver,
            parameters,
            result,
            span,
            owner,
        });
        Ok(())
    }
}
