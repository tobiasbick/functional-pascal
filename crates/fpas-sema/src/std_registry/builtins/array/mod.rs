use super::{array_elem_ty, mutable_array_elem_ty, simple_var_name};
use crate::check::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_WRONG_ARGUMENT_COUNT;
use fpas_lexer::Span;
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

mod higher_order;
mod mutation;
mod query;

pub(super) fn check_array_builtin_std_call(
    c: &mut Checker,
    name: &str,
    args: &[Expr],
    span: Span,
) -> Option<Ty> {
    let ty = match name {
        s::STD_ARRAY_LENGTH => query::check_length(c, args, span),
        s::STD_ARRAY_SORT | s::STD_ARRAY_REVERSE => {
            query::check_sort_or_reverse(c, name, args, span)
        }
        s::STD_ARRAY_CONTAINS | s::STD_ARRAY_INDEX_OF => {
            query::check_contains_or_index_of(c, name, args, span)
        }
        s::STD_ARRAY_SLICE => query::check_slice(c, args, span),
        s::STD_ARRAY_PUSH => mutation::check_push(c, args, span),
        s::STD_ARRAY_POP => mutation::check_pop(c, args, span),
        s::STD_ARRAY_MAP => higher_order::check_map(c, args, span),
        s::STD_ARRAY_FILTER => higher_order::check_filter(c, args, span),
        s::STD_ARRAY_REDUCE => higher_order::check_reduce(c, args, span),
        s::STD_ARRAY_CONCAT => query::check_concat(c, args, span),
        s::STD_ARRAY_FILL => query::check_fill(c, args, span),
        s::STD_ARRAY_FIND => higher_order::check_find(c, args, span),
        s::STD_ARRAY_FIND_INDEX => higher_order::check_find_index(c, args, span),
        s::STD_ARRAY_ANY => higher_order::check_any(c, args, span),
        s::STD_ARRAY_ALL => higher_order::check_all(c, args, span),
        s::STD_ARRAY_FLAT_MAP => higher_order::check_flat_map(c, args, span),
        s::STD_ARRAY_FOR_EACH => higher_order::check_for_each(c, args, span),
        _ => return None,
    };

    Some(ty)
}

fn check_argument_count(
    c: &mut Checker,
    name: &str,
    expected: usize,
    args: &[Expr],
    example: &str,
    span: Span,
) -> bool {
    if args.len() == expected {
        return true;
    }

    c.error_with_code(
        SEMA_WRONG_ARGUMENT_COUNT,
        format!("`{name}` expects {expected} arguments, got {}", args.len()),
        example,
        span,
    );
    false
}
