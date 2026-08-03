//! Safe source-span anchors used while attaching comments.

use fpas_lexer::{Span, Token, lex_with_comments};
use fpas_parser::Stmt;

/// Start/end byte offsets of a formattable construct in source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EmissionAnchor {
    pub start: usize,
    pub end: usize,
}

/// Returns the byte offset one past `span`, saturating malformed external AST spans.
#[must_use]
pub(crate) fn span_end(span: Span) -> usize {
    span.offset.saturating_add(span.length)
}

/// Returns the lexer start offset for `stmt`.
#[must_use]
pub(crate) fn stmt_start(stmt: &Stmt) -> usize {
    match stmt {
        Stmt::Block(_, span) => span.offset,
        Stmt::Var(var) | Stmt::MutableVar(var) => var.span.offset,
        Stmt::Assign { span, .. }
        | Stmt::Return(_, span)
        | Stmt::Panic(_, span)
        | Stmt::If { span, .. }
        | Stmt::Case { span, .. }
        | Stmt::For { span, .. }
        | Stmt::ForIn { span, .. }
        | Stmt::While { span, .. }
        | Stmt::Repeat { span, .. }
        | Stmt::Break(span)
        | Stmt::Continue(span)
        | Stmt::Call { span, .. }
        | Stmt::Expression { span, .. }
        | Stmt::Go { span, .. } => span.offset,
    }
}

/// Returns the lexer end offset for `stmt`.
#[must_use]
pub(crate) fn stmt_end(stmt: &Stmt) -> usize {
    match stmt {
        Stmt::Block(_, span) => span_end(*span),
        Stmt::Var(var) | Stmt::MutableVar(var) => span_end(var.span),
        Stmt::Assign { span, .. }
        | Stmt::Return(_, span)
        | Stmt::Panic(_, span)
        | Stmt::If { span, .. }
        | Stmt::Case { span, .. }
        | Stmt::For { span, .. }
        | Stmt::ForIn { span, .. }
        | Stmt::While { span, .. }
        | Stmt::Repeat { span, .. }
        | Stmt::Break(span)
        | Stmt::Continue(span)
        | Stmt::Call { span, .. }
        | Stmt::Expression { span, .. }
        | Stmt::Go { span, .. } => span_end(*span),
    }
}

/// Byte offset of the compilation unit's `uses` keyword when present.
#[must_use]
pub(crate) fn uses_keyword_offset(source: &str) -> Option<usize> {
    let (tokens, _, _) = lex_with_comments(source);
    tokens
        .iter()
        .find(|token| token.token == Token::Uses)
        .map(|token| token.span.offset)
}

/// Returns `true` when `comment` may trail the construct ending at `anchor_end`.
#[must_use]
pub(crate) fn trailing_gap_allows(source: &str, anchor_end: usize, comment_start: usize) -> bool {
    if comment_start < anchor_end || !same_line(source, anchor_end, comment_start) {
        return false;
    }
    source
        .get(anchor_end..comment_start)
        .is_some_and(|gap| gap.chars().all(|c| c == ';' || c.is_whitespace()))
}

/// Returns `true` when `left` and `right` are on the same source line.
#[must_use]
pub(crate) fn same_line(source: &str, left: usize, right: usize) -> bool {
    let Some(before_left) = source.get(..left) else {
        return false;
    };
    let Some(before_right) = source.get(..right) else {
        return false;
    };
    line_start(before_left) == line_start(before_right)
}

fn line_start(text: &str) -> usize {
    text.rfind(['\n', '\r'])
        .map_or(0, |index| index.saturating_add(1))
}
