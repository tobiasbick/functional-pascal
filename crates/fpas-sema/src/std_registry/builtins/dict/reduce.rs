//! Type checking for `Std.Dictionaries.Reduce`.
//!
//! **Documentation:** `docs/pascal/std/collections/dict.md` (from the repository root).

use crate::check::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::super::callbacks::{TernaryFunctionCallbackSpec, expect_ternary_function_callback};
use super::super::check_argument_count;
use super::dict_kv_types;

pub(super) fn check_reduce(c: &mut Checker, args: &[&Expr], span: Span) -> Ty {
    if !check_argument_count(
        c,
        s::STD_DICT_REDUCE,
        3,
        args,
        "Pass Reduce(D, Init, function(Acc: U; Key: K; Value: V): U).",
        span,
    ) {
        return Ty::Error;
    }
    let dictionary = c.check_expr(&args[0]);
    let initial = c.check_expr(&args[1]);
    let callback = c.check_expr(&args[2]);
    if dictionary == Ty::Error || initial == Ty::Error || callback == Ty::Error {
        return Ty::Error;
    }
    let Some((key, value)) = dict_kv_types(&dictionary) else {
        c.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`{}` first argument must be a dict", s::STD_DICT_REDUCE),
            "Pass a dict of K to V as the first argument.",
            span,
        );
        return Ty::Error;
    };
    expect_ternary_function_callback(
        c,
        s::STD_DICT_REDUCE,
        &callback,
        TernaryFunctionCallbackSpec {
            parameter_types: [&initial, &key, &value],
            return_ty: &initial,
            hint: "Pass a function(Acc: U; Key: K; Value: V): U.",
        },
        span,
    )
    .unwrap_or(Ty::Error)
}
