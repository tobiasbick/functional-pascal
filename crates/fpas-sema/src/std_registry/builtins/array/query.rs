use crate::check::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::{array_elem_ty, check_argument_count};

pub(super) fn check_length(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !check_argument_count(
        c,
        s::STD_ARRAY_LENGTH,
        1,
        args,
        "Example: Std.Array.Length(A).",
        span,
    ) {
        return Ty::Error;
    }

    let ty = c.check_expr(&args[0]);
    if array_elem_ty(&ty).is_some() {
        Ty::Integer
    } else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{}` expects an array", s::STD_ARRAY_LENGTH),
            "Pass `array of T` (or a variable of array type).",
            span,
        );
        Ty::Error
    }
}

pub(super) fn check_sort_or_reverse(c: &mut Checker, name: &str, args: &[Expr], span: Span) -> Ty {
    if !check_argument_count(c, name, 1, args, "Example: Std.Array.Sort(A).", span) {
        return Ty::Error;
    }

    let ty = c.check_expr(&args[0]);
    if array_elem_ty(&ty).is_some() {
        ty
    } else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{name}` expects an array"),
            "Pass `array of T`.",
            span,
        );
        Ty::Error
    }
}

pub(super) fn check_contains_or_index_of(
    c: &mut Checker,
    name: &str,
    args: &[Expr],
    span: Span,
) -> Ty {
    if !check_argument_count(c, name, 2, args, "Example: Std.Array.Contains(A, V).", span) {
        return Ty::Error;
    }

    let array_ty = c.check_expr(&args[0]);
    let Some(elem_ty) = array_elem_ty(&array_ty) else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{name}` first argument must be an array"),
            "Pass `array of T`.",
            span,
        );
        c.check_expr(&args[1]);
        return Ty::Error;
    };
    let value_ty = c.check_expr(&args[1]);
    c.check_type_compat(&elem_ty, &value_ty, "compared value", span);

    if name == s::STD_ARRAY_CONTAINS {
        Ty::Boolean
    } else {
        Ty::Integer
    }
}

pub(super) fn check_slice(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !check_argument_count(
        c,
        s::STD_ARRAY_SLICE,
        3,
        args,
        "Example: Std.Array.Slice(A, Start, Len).",
        span,
    ) {
        return Ty::Error;
    }

    let array_ty = c.check_expr(&args[0]);
    if array_elem_ty(&array_ty).is_none() {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{}` first argument must be an array", s::STD_ARRAY_SLICE),
            "Pass `array of T`.",
            span,
        );
        c.check_expr(&args[1]);
        c.check_expr(&args[2]);
        return Ty::Error;
    }

    let start_ty = c.check_expr(&args[1]);
    let len_ty = c.check_expr(&args[2]);
    c.check_type_compat(&Ty::Integer, &start_ty, "start index", span);
    c.check_type_compat(&Ty::Integer, &len_ty, "length", span);
    array_ty
}

pub(super) fn check_concat(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !check_argument_count(
        c,
        s::STD_ARRAY_CONCAT,
        2,
        args,
        "Example: Std.Array.Concat(A, B).",
        span,
    ) {
        return Ty::Error;
    }

    let a_ty = c.check_expr(&args[0]);
    let b_ty = c.check_expr(&args[1]);

    let Some(a_elem_ty) = array_elem_ty(&a_ty) else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{}` first argument must be an array", s::STD_ARRAY_CONCAT),
            "Pass `array of T` as the first argument.",
            span,
        );
        return Ty::Error;
    };

    let Some(b_elem_ty) = array_elem_ty(&b_ty) else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{}` second argument must be an array", s::STD_ARRAY_CONCAT),
            "Pass `array of T` as the second argument.",
            span,
        );
        return Ty::Error;
    };

    if !a_elem_ty.compatible_with(&b_elem_ty) {
        c.check_type_compat(&a_elem_ty, &b_elem_ty, "right array element", span);
        return Ty::Error;
    }

    if a_elem_ty.is_error() && !b_elem_ty.is_error() {
        b_ty
    } else {
        a_ty
    }
}

pub(super) fn check_fill(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if !check_argument_count(
        c,
        s::STD_ARRAY_FILL,
        2,
        args,
        "Example: Std.Array.Fill(Value, Count).",
        span,
    ) {
        return Ty::Error;
    }

    let elem_ty = c.check_expr(&args[0]);
    let count_ty = c.check_expr(&args[1]);
    c.check_type_compat(&Ty::Integer, &count_ty, "count", span);
    Ty::Array(Box::new(elem_ty))
}
