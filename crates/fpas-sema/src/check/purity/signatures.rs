//! Pure callable signatures retain guarantees through concrete instantiation.

use crate::check::Checker;
use crate::types::{FunctionTy, ParamTy, Ty, TypeArguments};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;

impl Checker {
    pub(in crate::check) fn has_valid_pure_signatures(&self, ty: &Ty) -> bool {
        ty.has_valid_pure_signatures_with(|ty| {
            let resolved = self.resolve_visible_type(ty);
            if let Ty::GenericParam(parameter) = &resolved
                && self.pure_parameters.contains(&parameter.identity)
            {
                return Ty::Integer;
            }
            resolved
        })
    }

    pub(in crate::check) fn check_nested_pure_signatures(&mut self, ty: &Ty, span: Span) {
        self.record_default_purity_requirement(ty, true);
        if !self.has_valid_pure_signatures(ty) {
            self.error_with_code(SEMA_TYPE_MISMATCH,
                format!("Type `{ty}` contains a pure signature with forbidden parameter or result types"),
                "Pure signatures require value parameters and resource-free data or pure callable components.", span);
        }
    }

    pub(in crate::check) fn check_pure_signature(
        &mut self,
        params: &[ParamTy],
        result: &Ty,
        span: Span,
    ) {
        for parameter in params {
            if parameter.mutable || !self.is_pure_data(&parameter.ty) {
                self.error_with_code(SEMA_TYPE_MISMATCH,
                    format!("Pure function parameter `{}` must be resource-free and cannot use var", parameter.name),
                    "Use an ordinary value parameter containing only resource-free data or pure callables.", span);
            }
        }
        if !self.is_pure_data(result) {
            self.error_with_code(SEMA_TYPE_MISMATCH,
                format!("Pure function result `{result}` is not resource-free data or a pure callable"),
                "Return data or a pure callable; resource handles and ordinary callable components are forbidden.", span);
        }
    }

    pub(in crate::check) fn check_pure_instantiation(
        &mut self,
        function: &FunctionTy,
        arguments: &TypeArguments,
        span: Span,
    ) {
        if !function.pure {
            self.check_nested_pure_signatures(
                &Ty::Function(function.clone()).substitute(arguments),
                span,
            );
            return;
        }
        for parameter in &function.type_params {
            if let Some(argument) = arguments.get(&parameter.identity)
                && !self.is_pure_data(argument)
            {
                self.error_with_code(SEMA_TYPE_MISMATCH,
                    format!("Type argument `{argument}` for pure function parameter `{}` is not resource-free", parameter.name),
                    "Instantiate pure functions only with resource-free data or pure callable types.", span);
            }
        }
        let params = function
            .params
            .iter()
            .map(|parameter| ParamTy {
                ty: parameter.ty.substitute(arguments),
                ..parameter.clone()
            })
            .collect::<Vec<_>>();
        self.check_pure_signature(&params, &function.return_type.substitute(arguments), span);
    }
}
