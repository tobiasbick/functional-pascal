//! Record routine argument validation.

use super::Checker;
use crate::types::{GenericParamDef, ParamTy, Ty};
use fpas_diagnostics::codes::SEMA_WRONG_ARGUMENT_COUNT;
use fpas_lexer::Span;
use fpas_parser::Expr;
use std::collections::HashMap;

impl Checker {
    /// Check arguments against the visible record routine parameters.
    pub(in crate::check) fn check_method_call_args(
        &mut self,
        name: &str,
        type_params: &[GenericParamDef],
        visible_params: &[ParamTy],
        args: &[Expr],
        span: Span,
    ) -> HashMap<String, Ty> {
        self.check_method_call_args_with_hint(
            name,
            type_params,
            visible_params,
            args,
            span,
            "Check the number of arguments (Self is implicit).",
        )
    }

    /// Check arguments against the visible record routine parameters.
    pub(in crate::check) fn check_static_call_args(
        &mut self,
        name: &str,
        type_params: &[GenericParamDef],
        params: &[ParamTy],
        args: &[Expr],
        span: Span,
    ) -> HashMap<String, Ty> {
        self.check_method_call_args_with_hint(
            name,
            type_params,
            params,
            args,
            span,
            "Check the number of arguments.",
        )
    }

    fn check_method_call_args_with_hint(
        &mut self,
        name: &str,
        type_params: &[GenericParamDef],
        visible_params: &[ParamTy],
        args: &[Expr],
        span: Span,
        arity_hint: &str,
    ) -> HashMap<String, Ty> {
        if visible_params.len() != args.len() {
            self.error_with_code(
                SEMA_WRONG_ARGUMENT_COUNT,
                format!(
                    "Method `{name}` expects {} arguments, got {}",
                    visible_params.len(),
                    args.len()
                ),
                arity_hint,
                span,
            );
        }

        let mut arg_types = Vec::with_capacity(args.len());
        for (index, arg) in args.iter().enumerate() {
            let arg_ty = if let Some(param) = visible_params.get(index) {
                self.check_expr_with_expected_record_literals(arg, &param.ty)
            } else {
                self.check_expr(arg)
            };
            arg_types.push(arg_ty);
        }

        let inferred =
            self.validate_routine_constraints(type_params, visible_params, &arg_types, span);
        for (index, (param, arg_ty)) in visible_params.iter().zip(&arg_types).enumerate() {
            let expected = Self::substitute_type_params(&param.ty, &inferred);
            self.check_type_compat(&expected, arg_ty, &format!("argument {}", index + 1), span);
        }
        inferred
    }
}
