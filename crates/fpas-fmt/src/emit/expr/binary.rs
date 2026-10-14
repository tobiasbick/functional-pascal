//! Binary operand grouping, source line breaks, and operator comments.
//!
//! **Documentation:** `docs/pascal/tools/fmt-style.md#line-width-v2`.

use super::super::width;
use super::precedence::{binary_prec, operand_prec};
use super::{Emitter, emit_expr};
use crate::comments::{CommentMap, emit_leading_comments, emit_trailing_comments};
use crate::style::INDENT_WIDTH;
use fpas_parser::{BinaryOp, Expr};

/// Emits a binary expression without changing its operand grouping.
pub(super) fn emit_binary(
    emitter: &mut Emitter,
    op: BinaryOp,
    left: &Expr,
    right: &Expr,
    min_prec: u8,
    comments: &CommentMap,
) {
    let parens = binary_prec(op) < min_prec;
    if parens {
        emitter.write("(");
    }
    let previous_continuation = emitter.continuation_indent;
    let continuation_indent = previous_continuation.unwrap_or(emitter.line_indent() + INDENT_WIDTH);
    emitter.continuation_indent = Some(continuation_indent);
    emit_expr(emitter, left, operand_prec(op, left, false), comments);
    let left_end = left.span().offset.saturating_add(left.span().length);
    let operator = emitter
        .layout
        .operator_between(left_end, right.span().offset);
    let has_comments = !comments.leading_at(right.span().offset).is_empty()
        || operator.is_some_and(|span| !comments.trailing_at(span.offset).is_empty());
    let right_width = if emitter.measuring {
        None
    } else {
        width::expression_width(right, operand_prec(op, right, true))
    };
    if has_comments
        || emitter.would_exceed_width(right_width, binary_op_spaced(op).len() + 1)
        || emitter
            .layout
            .breaks_between(left_end.saturating_sub(1), right.span().offset)
    {
        if matches!(op, BinaryOp::And | BinaryOp::Or | BinaryOp::Xor) {
            emitter.at_indent(continuation_indent, Emitter::start_line);
        } else {
            emitter.write(" ");
        }
        emitter.write(binary_op_spaced(op).trim());
        if let Some(operator) = operator {
            emit_trailing_comments(emitter, comments, operator.offset);
        }
        if !emitter.ends_with_newline() {
            emitter.write_line_end();
        }
        emitter.at_indent(continuation_indent, |inner| {
            emit_leading_comments(inner, comments, right.span().offset, false);
            inner.write_current_indent();
        });
    } else {
        emitter.write(binary_op_spaced(op));
    }
    emit_expr(emitter, right, operand_prec(op, right, true), comments);
    emitter.continuation_indent = previous_continuation;
    if parens {
        emitter.write(")");
    }
}

/// Returns an operator surrounded by its canonical single spaces.
pub(super) fn binary_op_spaced(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Mul => " * ",
        BinaryOp::RealDiv => " / ",
        BinaryOp::IntDiv => " div ",
        BinaryOp::Mod => " mod ",
        BinaryOp::And => " and ",
        BinaryOp::Add => " + ",
        BinaryOp::Sub => " - ",
        BinaryOp::Or => " or ",
        BinaryOp::Xor => " xor ",
        BinaryOp::Eq => " = ",
        BinaryOp::NotEq => " <> ",
        BinaryOp::Lt => " < ",
        BinaryOp::Gt => " > ",
        BinaryOp::LtEq => " <= ",
        BinaryOp::GtEq => " >= ",
        BinaryOp::In => " in ",
    }
}
