//! `if` expressions: one line when they fit and carry no comments, otherwise one
//! branch per line with `elsif`, `else`, and `end if` indented one level.
//!
//! **Documentation:** `docs/pascal/tools/fmt-style.md` and
//! `docs/pascal/language/control-flow/if-then-else.md`.

use fpas_lexer::Span;
use fpas_parser::{Expr, IfExprBranch};

use super::super::Emitter;
use super::super::wrap::{exceeds_width, measure_emit, text_width};
use super::{emit_expr, emit_expression_end};
use crate::comments::{CommentMap, emit_leading_comments, emit_trailing_comments};

/// Emits `if ... end if`, preserving comments attached to its branches and ending.
pub(super) fn emit_if_expression(
    emitter: &mut Emitter,
    branches: &[IfExprBranch],
    else_value: &Expr,
    else_span: Span,
    span: Span,
    comments: &CommentMap,
) {
    let closer = comments.closer_anchor(span.offset);
    let has_comments = branches
        .iter()
        .map(|branch| branch.span.offset)
        .chain(std::iter::once(else_span.offset))
        .chain(closer)
        .any(|anchor| {
            !comments.leading_at(anchor).is_empty() || !comments.trailing_at(anchor).is_empty()
        });
    if !has_comments {
        let compact = measure_emit(|inner| emit_compact(inner, branches, else_value, comments));
        if !compact.contains('\n') && !exceeds_width(emitter.column(), text_width(&compact)) {
            emit_compact(emitter, branches, else_value, comments);
            return;
        }
    }
    let Some((first, rest)) = branches.split_first() else {
        return;
    };
    emit_branch(emitter, "if ", first, comments);
    finish_line(emitter, first.span.offset, comments);
    emitter.with_indent(|inner| {
        for branch in rest {
            emit_leading_comments(inner, comments, branch.span.offset, false);
            inner.write_current_indent();
            emit_branch(inner, "elsif ", branch, comments);
            finish_line(inner, branch.span.offset, comments);
        }
        emit_leading_comments(inner, comments, else_span.offset, false);
        inner.write_current_indent();
        inner.write("else ");
        emit_expr(inner, else_value, 0, comments);
        finish_line(inner, else_span.offset, comments);
        emit_expression_end(inner, closer, "end if", comments);
    });
}

/// Ends a branch line after its trailing comments.
fn finish_line(emitter: &mut Emitter, anchor: usize, comments: &CommentMap) {
    emit_trailing_comments(emitter, comments, anchor);
    if !emitter.ends_with_newline() {
        emitter.write_line_end();
    }
}

fn emit_compact(
    emitter: &mut Emitter,
    branches: &[IfExprBranch],
    else_value: &Expr,
    comments: &CommentMap,
) {
    for (index, branch) in branches.iter().enumerate() {
        if index > 0 {
            emitter.write(" ");
        }
        emit_branch(
            emitter,
            if index == 0 { "if " } else { "elsif " },
            branch,
            comments,
        );
    }
    emitter.write(" else ");
    emit_expr(emitter, else_value, 0, comments);
    emitter.write(" end if");
}

fn emit_branch(emitter: &mut Emitter, keyword: &str, branch: &IfExprBranch, comments: &CommentMap) {
    emitter.write(keyword);
    emit_expr(emitter, &branch.condition, 0, comments);
    emitter.write(" then ");
    emit_expr(emitter, &branch.value, 0, comments);
}
