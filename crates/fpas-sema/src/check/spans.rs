use fpas_lexer::Span;
use fpas_parser::Expr;

pub(crate) fn expr_span(expr: &Expr) -> Span {
    match expr {
        Expr::Integer(_, span) => *span,
        Expr::Real(_, span) => *span,
        Expr::Str(_, span) => *span,
        Expr::Bool(_, span) => *span,
        Expr::Designator(designator) => designator.span,
        Expr::Call { span, .. } => *span,
        Expr::UnaryOp { span, .. } => *span,
        Expr::BinaryOp { span, .. } => *span,
        Expr::Paren(_, span) => *span,
        Expr::ArrayLiteral(_, span) => *span,
        Expr::DictLiteral(_, span) => *span,
        Expr::RecordLiteral { span, .. } => *span,
        Expr::New { span, .. } => *span,
        Expr::ResultOk(_, span) => *span,
        Expr::ResultError(_, span) => *span,
        Expr::OptionSome(_, span) => *span,
        Expr::OptionNone(span) => *span,
        Expr::Try(_, span) => *span,
        Expr::Function { span, .. } => *span,
        Expr::Go(_, span) => *span,
    }
}
