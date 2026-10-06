//! Named case endings and scoped arm-list emission.
//!
//! **Documentation:** `docs/pascal/language/control-flow/case-of-intro.md`.

use super::super::Emitter;
use super::super::expr::emit_expr;
use super::line::write_indented;
use crate::comments::{CommentMap, emit_leading_comments, stmt_start};
use fpas_parser::{CaseArm, CaseLabel, DestructureVariant, Stmt};

/// Emits `when` arms, an optional scoped catch-all, and `end case`.
pub(super) fn emit_case(emitter: &mut Emitter, stmt: &Stmt, comments: &CommentMap) {
    let Stmt::Case {
        expr,
        arms,
        else_body,
        ..
    } = stmt
    else {
        return;
    };

    write_indented(emitter);
    emitter.write("case ");
    emit_expr(emitter, expr, 0, comments);
    emitter.write(" of\n");

    emitter.with_indent(|inner| {
        for arm in arms {
            emit_case_arm(inner, arm, comments);
        }

        if let Some(else_stmts) = else_body {
            if let Some(anchor) = comments.body_anchor(stmt_start(stmt)) {
                emit_leading_comments(inner, comments, anchor, false);
            }
            inner.writeln("else");
            inner.with_indent(|body| super::emit_stmts_in_block(body, else_stmts, comments));
        }
    });

    super::conditionals::emit_control_end(emitter, stmt, "end case", comments);
}

/// Emits a case arm header and its indented statements, preserving explicit inner blocks.
pub(super) fn emit_case_arm(emitter: &mut Emitter, arm: &CaseArm, comments: &CommentMap) {
    emit_leading_comments(emitter, comments, arm.span.offset, false);
    write_indented(emitter);
    emitter.write("when ");
    emit_case_labels(emitter, &arm.labels, comments);
    if let Some(guard) = &arm.guard {
        emitter.write(" if ");
        emit_expr(emitter, guard, 0, comments);
    }
    emitter.write(":\n");
    super::conditionals::emit_control_body(emitter, &arm.body, comments);
}

/// Emits labels separated by commas in one arm header.
fn emit_case_labels(emitter: &mut Emitter, labels: &[CaseLabel], comments: &CommentMap) {
    for (index, label) in labels.iter().enumerate() {
        if index > 0 {
            emitter.write(", ");
        }
        emit_case_label(emitter, label, comments);
    }
}

fn emit_case_label(emitter: &mut Emitter, label: &CaseLabel, comments: &CommentMap) {
    match label {
        CaseLabel::Value { start, end, .. } => {
            emit_expr(emitter, start, 0, comments);
            if let Some(end_expr) = end {
                emitter.write("..");
                emit_expr(emitter, end_expr, 0, comments);
            }
        }
        CaseLabel::Destructure {
            variant, binding, ..
        } => {
            let name = match variant {
                DestructureVariant::Ok => "Ok",
                DestructureVariant::Error => "Error",
                DestructureVariant::Some => "Some",
                DestructureVariant::None => "None",
            };
            emitter.write(name);
            if *variant == DestructureVariant::None {
                return;
            }
            emitter.write("(");
            emitter.write(binding.as_deref().unwrap_or("_"));
            emitter.write(")");
        }
    }
}
