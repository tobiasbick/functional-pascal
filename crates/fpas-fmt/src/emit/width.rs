//! Width measurement for structural expression wrapping.

use super::{Emitter, expr::emit_expr};
use crate::comments::CommentMap;
use fpas_parser::Expr;
use unicode_width::UnicodeWidthStr;

/// Returns the display width of text, including wide and combining characters.
pub(super) fn text_width(text: &str) -> usize {
    UnicodeWidthStr::width(text)
}

/// Measures compact emission without recursively applying automatic width wrapping.
pub(super) fn measure(emit: impl FnOnce(&mut Emitter)) -> Option<usize> {
    let mut emitter = Emitter::for_measurement();
    emit(&mut emitter);
    let rendered = emitter.finish();
    (!rendered.contains('\n')).then(|| text_width(&rendered))
}

/// Measures the opening line of an item whose source layout may already span lines.
pub(super) fn first_line(comments: &CommentMap, emit: impl FnOnce(&mut Emitter)) -> usize {
    let mut emitter = Emitter::for_source_measurement(comments);
    emit(&mut emitter);
    text_width(emitter.finish().split('\n').next().unwrap_or_default())
}

/// Measures an operand with the parentheses required by its surrounding operator.
pub(super) fn expression_width(expr: &Expr, min_prec: u8) -> Option<usize> {
    measure(|emitter| emit_expr(emitter, expr, min_prec, &CommentMap::default()))
}
