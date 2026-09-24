//! Shared callback-type validation for higher-order standard-library operations.

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

/// Expected parameter and return types for a three-argument function callback.
pub(super) struct TernaryFunctionCallbackSpec<'a> {
    pub(super) parameter_types: [&'a Ty; 3],
    pub(super) return_ty: &'a Ty,
    pub(super) hint: &'a str,
}

/// Validates a three-argument function callback and returns its result type.
pub(super) fn expect_ternary_function_callback(
    c: &mut Checker,
    std_name: &str,
    callback_ty: &Ty,
    spec: TernaryFunctionCallbackSpec<'_>,
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
    if params.len() != 3 {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{std_name}` callback must take exactly 3 arguments"),
            spec.hint,
            span,
        );
        return None;
    }

    let mut valid = true;
    for (index, (expected, actual)) in spec.parameter_types.iter().zip(params).enumerate() {
        c.check_type_compat(
            expected,
            &actual.ty,
            &format!("callback argument {}", index + 1),
            span,
        );
        valid &= expected.compatible_with(&actual.ty);
    }
    c.check_type_compat(spec.return_ty, return_type, "callback return type", span);
    valid &= spec.return_ty.compatible_with(return_type);
    valid.then(|| (**return_type).clone())
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
