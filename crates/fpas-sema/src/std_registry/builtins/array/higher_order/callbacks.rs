//! Shared callback-type validation helpers for higher-order `Std.Array` checks.

use crate::check::Checker;
use crate::types::{FunctionTy, ProcedureTy, Ty};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;

/// Validates a unary function callback `function(V: T): R` and returns the return type.
pub(super) fn expect_unary_function_callback(
    c: &mut Checker,
    std_name: &str,
    callback_ty: &Ty,
    expected_param_ty: &Ty,
    expected_return_ty: Option<&Ty>,
    span: Span,
    hint: &str,
) -> Option<Ty> {
    let Ty::Function(FunctionTy {
        params,
        return_type,
        ..
    }) = callback_ty
    else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{std_name}` second argument must be a function"),
            hint,
            span,
        );
        return None;
    };

    if params.len() != 1 {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{std_name}` callback must take exactly 1 argument"),
            hint,
            span,
        );
        return None;
    }

    c.check_type_compat(
        expected_param_ty,
        &params[0].ty,
        "callback argument 1",
        span,
    );
    if !expected_param_ty.compatible_with(&params[0].ty) {
        return None;
    }

    if let Some(expected_return_ty) = expected_return_ty {
        c.check_type_compat(
            expected_return_ty,
            return_type,
            "callback return type",
            span,
        );
        if !expected_return_ty.compatible_with(return_type) {
            return None;
        }
    }

    Some((**return_type).clone())
}

/// Spec for [`expect_binary_function_callback`].
pub(super) struct BinaryFunctionCallbackSpec<'a> {
    pub(super) first_param_ty: &'a Ty,
    pub(super) second_param_ty: &'a Ty,
    pub(super) return_ty: Option<&'a Ty>,
    pub(super) hint: &'a str,
}

/// Validates a binary function callback `function(A: U; B: T): R` and returns the return type.
pub(super) fn expect_binary_function_callback(
    c: &mut Checker,
    std_name: &str,
    callback_ty: &Ty,
    spec: BinaryFunctionCallbackSpec<'_>,
    span: Span,
) -> Option<Ty> {
    let Ty::Function(FunctionTy {
        params,
        return_type,
        ..
    }) = callback_ty
    else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{std_name}` callback must be a function"),
            spec.hint,
            span,
        );
        return None;
    };

    if params.len() != 2 {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{std_name}` callback must take exactly 2 arguments"),
            spec.hint,
            span,
        );
        return None;
    }

    c.check_type_compat(
        spec.first_param_ty,
        &params[0].ty,
        "callback argument 1",
        span,
    );
    c.check_type_compat(
        spec.second_param_ty,
        &params[1].ty,
        "callback argument 2",
        span,
    );
    if !spec.first_param_ty.compatible_with(&params[0].ty)
        || !spec.second_param_ty.compatible_with(&params[1].ty)
    {
        return None;
    }

    if let Some(expected_return_ty) = spec.return_ty {
        c.check_type_compat(
            expected_return_ty,
            return_type,
            "callback return type",
            span,
        );
        if !expected_return_ty.compatible_with(return_type) {
            return None;
        }
    }

    Some((**return_type).clone())
}

/// Validates a unary procedure callback `procedure(V: T)`.
pub(super) fn expect_unary_procedure_callback(
    c: &mut Checker,
    std_name: &str,
    callback_ty: &Ty,
    expected_param_ty: &Ty,
    span: Span,
    hint: &str,
) -> Option<()> {
    let Ty::Procedure(ProcedureTy { params, .. }) = callback_ty else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{std_name}` second argument must be a procedure"),
            hint,
            span,
        );
        return None;
    };

    if params.len() != 1 {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{std_name}` callback must take exactly 1 argument"),
            hint,
            span,
        );
        return None;
    }

    c.check_type_compat(
        expected_param_ty,
        &params[0].ty,
        "callback argument 1",
        span,
    );
    if !expected_param_ty.compatible_with(&params[0].ty) {
        return None;
    }

    Some(())
}
