use fpas_lexer::Span;
use fpas_parser::Expr;

pub(crate) fn expr_span(expr: &Expr) -> Span {
    expr.span()
}
