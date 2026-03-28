use crate::check::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::{SEMA_TYPE_MISMATCH, SEMA_WRONG_ARGUMENT_COUNT};
use fpas_lexer::Span;
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

pub(super) fn check_dict_builtin_std_call(
    c: &mut Checker,
    name: &str,
    args: &[Expr],
    span: Span,
) -> Option<Ty> {
    let ty = match name {
        s::STD_DICT_LENGTH => check_dict_length(c, args, span),
        s::STD_DICT_CONTAINS_KEY => check_dict_contains_key(c, args, span),
        s::STD_DICT_KEYS => check_dict_keys(c, args, span),
        s::STD_DICT_VALUES => check_dict_values(c, args, span),
        s::STD_DICT_REMOVE => check_dict_remove(c, args, span),
        _ => return None,
    };
    Some(ty)
}

fn dict_kv_types(ty: &Ty) -> Option<(Ty, Ty)> {
    if let Ty::Dict(k, v) = ty {
        Some((*k.clone(), *v.clone()))
    } else {
        None
    }
}

fn expect_dict_arg(
    c: &mut Checker,
    func_name: &str,
    args: &[Expr],
    expected: usize,
    span: Span,
) -> Option<Ty> {
    if args.len() != expected {
        c.error_with_code(
            SEMA_WRONG_ARGUMENT_COUNT,
            format!(
                "`{func_name}` expects {expected} argument(s), got {}",
                args.len()
            ),
            format!("Example: {func_name}(D)."),
            span,
        );
        return None;
    }
    Some(c.check_expr(&args[0]))
}

fn check_dict_length(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    let Some(ty) = expect_dict_arg(c, s::STD_DICT_LENGTH, args, 1, span) else {
        return Ty::Error;
    };
    if dict_kv_types(&ty).is_some() {
        Ty::Integer
    } else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{}` expects a dict", s::STD_DICT_LENGTH),
            "Pass `dict of K to V`.",
            span,
        );
        Ty::Error
    }
}

fn check_dict_contains_key(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if args.len() != 2 {
        c.error_with_code(
            SEMA_WRONG_ARGUMENT_COUNT,
            format!(
                "`{}` expects 2 arguments, got {}",
                s::STD_DICT_CONTAINS_KEY,
                args.len()
            ),
            "Example: Std.Dict.ContainsKey(D, Key).",
            span,
        );
        return Ty::Error;
    }
    let dict_ty = c.check_expr(&args[0]);
    let _key_ty = c.check_expr(&args[1]);
    if dict_kv_types(&dict_ty).is_some() {
        Ty::Boolean
    } else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!(
                "`{}` expects a dict as first argument",
                s::STD_DICT_CONTAINS_KEY
            ),
            "Pass `dict of K to V`.",
            span,
        );
        Ty::Error
    }
}

fn check_dict_keys(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    let Some(ty) = expect_dict_arg(c, s::STD_DICT_KEYS, args, 1, span) else {
        return Ty::Error;
    };
    if let Some((k, _)) = dict_kv_types(&ty) {
        Ty::Array(Box::new(k))
    } else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{}` expects a dict", s::STD_DICT_KEYS),
            "Pass `dict of K to V`.",
            span,
        );
        Ty::Error
    }
}

fn check_dict_values(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    let Some(ty) = expect_dict_arg(c, s::STD_DICT_VALUES, args, 1, span) else {
        return Ty::Error;
    };
    if let Some((_, v)) = dict_kv_types(&ty) {
        Ty::Array(Box::new(v))
    } else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{}` expects a dict", s::STD_DICT_VALUES),
            "Pass `dict of K to V`.",
            span,
        );
        Ty::Error
    }
}

fn check_dict_remove(c: &mut Checker, args: &[Expr], span: Span) -> Ty {
    if args.len() != 2 {
        c.error_with_code(
            SEMA_WRONG_ARGUMENT_COUNT,
            format!(
                "`{}` expects 2 arguments, got {}",
                s::STD_DICT_REMOVE,
                args.len()
            ),
            "Example: Std.Dict.Remove(D, Key).",
            span,
        );
        return Ty::Error;
    }
    let dict_ty = c.check_expr(&args[0]);
    let _key_ty = c.check_expr(&args[1]);
    if let Some((k, v)) = dict_kv_types(&dict_ty) {
        Ty::Dict(Box::new(k), Box::new(v))
    } else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{}` expects a dict as first argument", s::STD_DICT_REMOVE),
            "Pass `dict of K to V`.",
            span,
        );
        Ty::Error
    }
}
