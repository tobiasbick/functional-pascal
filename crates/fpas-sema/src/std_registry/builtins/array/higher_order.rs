use crate::check::Checker;
use crate::types::{FunctionTy, ProcedureTy, Ty};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::{array_elem_ty, check_argument_count};

/// `Std.Array.Map(Arr, F)` -> `array of U` where `F: function(V: T): U`.
pub(super) fn check_map(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !check_argument_count(
        c,
        s::STD_ARRAY_MAP,
        2,
        args,
        "Example: Std.Array.Map(Arr, function(X: integer): integer begin return X * 2 end).",
        span,
    ) {
        return Ty::Error;
    }

    let arr_ty = c.check_expr(&args[0]);
    let func_ty = c.check_expr(&args[1]);
    let Some(elem_ty) = array_elem_ty(&arr_ty) else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{}` first argument must be an array", s::STD_ARRAY_MAP),
            "Pass `array of T`.",
            span,
        );
        return Ty::Error;
    };

    match expect_unary_function_callback(
        c,
        s::STD_ARRAY_MAP,
        &func_ty,
        &elem_ty,
        None,
        span,
        "Pass a function(X: T): U.",
    ) {
        Some(return_type) => Ty::Array(Box::new(return_type)),
        _ => Ty::Error,
    }
}

/// `Std.Array.Filter(Arr, F)` -> `array of T` where `F: function(V: T): boolean`.
pub(super) fn check_filter(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !check_argument_count(
        c,
        s::STD_ARRAY_FILTER,
        2,
        args,
        "Example: Std.Array.Filter(Arr, function(X: integer): boolean begin return X > 0 end).",
        span,
    ) {
        return Ty::Error;
    }

    let arr_ty = c.check_expr(&args[0]);
    let func_ty = c.check_expr(&args[1]);
    let Some(elem_ty) = array_elem_ty(&arr_ty) else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{}` first argument must be an array", s::STD_ARRAY_FILTER),
            "Pass `array of T`.",
            span,
        );
        return Ty::Error;
    };
    if expect_unary_function_callback(
        c,
        s::STD_ARRAY_FILTER,
        &func_ty,
        &elem_ty,
        Some(&Ty::Boolean),
        span,
        "Pass a function(V: T): boolean.",
    )
    .is_none()
    {
        return Ty::Error;
    }

    arr_ty
}

/// `Std.Array.Reduce(Arr, Init, F)` -> `U` where `F: function(Acc: U; V: T): U`.
pub(super) fn check_reduce(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !check_argument_count(
        c,
        s::STD_ARRAY_REDUCE,
        3,
        args,
        "Example: Std.Array.Reduce(Arr, 0, function(Acc: integer; V: integer): integer begin return Acc + V end).",
        span,
    ) {
        return Ty::Error;
    }

    let arr_ty = c.check_expr(&args[0]);
    let init_ty = c.check_expr(&args[1]);
    let func_ty = c.check_expr(&args[2]);
    let Some(elem_ty) = array_elem_ty(&arr_ty) else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{}` first argument must be an array", s::STD_ARRAY_REDUCE),
            "Pass `array of T`.",
            span,
        );
        return Ty::Error;
    };

    match expect_binary_function_callback(
        c,
        s::STD_ARRAY_REDUCE,
        &func_ty,
        BinaryFunctionCallbackSpec {
            first_param_ty: &init_ty,
            second_param_ty: &elem_ty,
            return_ty: Some(&init_ty),
            hint: "Pass a function(Acc: U; V: T): U.",
        },
        span,
    ) {
        Some(return_ty) => return_ty,
        None => Ty::Error,
    }
}

/// `Std.Array.Find(Arr, Pred)` -> `Option of T` where `Pred: function(V: T): boolean`.
pub(super) fn check_find(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !check_argument_count(
        c,
        s::STD_ARRAY_FIND,
        2,
        args,
        "Example: Std.Array.Find(Arr, function(X: integer): boolean begin return X > 0 end).",
        span,
    ) {
        return Ty::Error;
    }

    let arr_ty = c.check_expr(&args[0]);
    let func_ty = c.check_expr(&args[1]);
    let Some(elem_ty) = array_elem_ty(&arr_ty) else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{}` first argument must be an array", s::STD_ARRAY_FIND),
            "Pass `array of T`.",
            span,
        );
        return Ty::Error;
    };
    if expect_unary_function_callback(
        c,
        s::STD_ARRAY_FIND,
        &func_ty,
        &elem_ty,
        Some(&Ty::Boolean),
        span,
        "Pass a function(V: T): boolean.",
    )
    .is_none()
    {
        return Ty::Error;
    }
    Ty::Option(Box::new(elem_ty))
}

/// `Std.Array.FindIndex(Arr, Pred)` -> `integer`.
pub(super) fn check_find_index(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !check_argument_count(
        c,
        s::STD_ARRAY_FIND_INDEX,
        2,
        args,
        "Example: Std.Array.FindIndex(Arr, function(X: integer): boolean begin return X > 0 end).",
        span,
    ) {
        return Ty::Error;
    }

    let arr_ty = c.check_expr(&args[0]);
    let func_ty = c.check_expr(&args[1]);
    let Some(elem_ty) = array_elem_ty(&arr_ty) else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!(
                "`{}` first argument must be an array",
                s::STD_ARRAY_FIND_INDEX
            ),
            "Pass `array of T`.",
            span,
        );
        return Ty::Error;
    };
    if expect_unary_function_callback(
        c,
        s::STD_ARRAY_FIND_INDEX,
        &func_ty,
        &elem_ty,
        Some(&Ty::Boolean),
        span,
        "Pass a function(V: T): boolean.",
    )
    .is_none()
    {
        return Ty::Error;
    }
    Ty::Integer
}

/// `Std.Array.Any(Arr, Pred)` -> `boolean`.
pub(super) fn check_any(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !check_argument_count(
        c,
        s::STD_ARRAY_ANY,
        2,
        args,
        "Example: Std.Array.Any(Arr, function(X: integer): boolean begin return X > 0 end).",
        span,
    ) {
        return Ty::Error;
    }

    let arr_ty = c.check_expr(&args[0]);
    let func_ty = c.check_expr(&args[1]);
    let Some(elem_ty) = array_elem_ty(&arr_ty) else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{}` first argument must be an array", s::STD_ARRAY_ANY),
            "Pass `array of T`.",
            span,
        );
        return Ty::Error;
    };
    if expect_unary_function_callback(
        c,
        s::STD_ARRAY_ANY,
        &func_ty,
        &elem_ty,
        Some(&Ty::Boolean),
        span,
        "Pass a function(V: T): boolean.",
    )
    .is_none()
    {
        return Ty::Error;
    }
    Ty::Boolean
}

/// `Std.Array.All(Arr, Pred)` -> `boolean`.
pub(super) fn check_all(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !check_argument_count(
        c,
        s::STD_ARRAY_ALL,
        2,
        args,
        "Example: Std.Array.All(Arr, function(X: integer): boolean begin return X > 0 end).",
        span,
    ) {
        return Ty::Error;
    }

    let arr_ty = c.check_expr(&args[0]);
    let func_ty = c.check_expr(&args[1]);
    let Some(elem_ty) = array_elem_ty(&arr_ty) else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{}` first argument must be an array", s::STD_ARRAY_ALL),
            "Pass `array of T`.",
            span,
        );
        return Ty::Error;
    };
    if expect_unary_function_callback(
        c,
        s::STD_ARRAY_ALL,
        &func_ty,
        &elem_ty,
        Some(&Ty::Boolean),
        span,
        "Pass a function(V: T): boolean.",
    )
    .is_none()
    {
        return Ty::Error;
    }
    Ty::Boolean
}

/// `Std.Array.FlatMap(Arr, F)` -> `array of U` where `F: function(V: T): array of U`.
pub(super) fn check_flat_map(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !check_argument_count(
        c,
        s::STD_ARRAY_FLAT_MAP,
        2,
        args,
        "Example: Std.Array.FlatMap(Arr, function(X: integer): array of integer begin ... end).",
        span,
    ) {
        return Ty::Error;
    }

    let arr_ty = c.check_expr(&args[0]);
    let func_ty = c.check_expr(&args[1]);
    let Some(elem_ty) = array_elem_ty(&arr_ty) else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!(
                "`{}` first argument must be an array",
                s::STD_ARRAY_FLAT_MAP
            ),
            "Pass `array of T`.",
            span,
        );
        return Ty::Error;
    };
    match expect_unary_function_callback(
        c,
        s::STD_ARRAY_FLAT_MAP,
        &func_ty,
        &elem_ty,
        None,
        span,
        "Pass a function(V: T): array of U.",
    ) {
        Some(return_ty) if return_ty.is_error() => Ty::Error,
        Some(return_ty) if array_elem_ty(&return_ty).is_some() => return_ty,
        Some(_) => {
            c.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("`{}` mapper must return an array", s::STD_ARRAY_FLAT_MAP),
                "Pass a function(V: T): array of U.",
                span,
            );
            Ty::Error
        }
        None => Ty::Error,
    }
}

/// `Std.Array.ForEach(Arr, F)` -> `unit`.
pub(super) fn check_for_each(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !check_argument_count(
        c,
        s::STD_ARRAY_FOR_EACH,
        2,
        args,
        "Example: Std.Array.ForEach(Arr, procedure(X: integer) begin ... end).",
        span,
    ) {
        return Ty::Error;
    }

    let arr_ty = c.check_expr(&args[0]);
    let func_ty = c.check_expr(&args[1]);
    let Some(elem_ty) = array_elem_ty(&arr_ty) else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!(
                "`{}` first argument must be an array",
                s::STD_ARRAY_FOR_EACH
            ),
            "Pass `array of T`.",
            span,
        );
        return Ty::Error;
    };
    if expect_unary_procedure_callback(
        c,
        s::STD_ARRAY_FOR_EACH,
        &func_ty,
        &elem_ty,
        span,
        "Pass a procedure(V: T).",
    )
    .is_none()
    {
        return Ty::Error;
    }
    Ty::Unit
}

fn expect_unary_function_callback(
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

fn expect_binary_function_callback(
    c: &mut Checker,
    std_name: &str,
    callback_ty: &Ty,
    spec: BinaryFunctionCallbackSpec<'_>,
    span: Span,
) -> Option<Ty> {
    let Ty::Function(FunctionTy {
        params,
        return_type,
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

struct BinaryFunctionCallbackSpec<'a> {
    first_param_ty: &'a Ty,
    second_param_ty: &'a Ty,
    return_ty: Option<&'a Ty>,
    hint: &'a str,
}

fn expect_unary_procedure_callback(
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
