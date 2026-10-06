//! Case statement and compound arm emission.
//!
//! **Documentation:** `docs/pascal/language/control-flow/case-of-intro.md`.

use super::super::Emitter;
use super::super::expr::emit_expr;
use super::line::write_indented;
use crate::comments::{CommentMap, emit_leading_comments, stmt_start};
use fpas_parser::{CaseArm, CaseLabel, DestructureVariant, Stmt};

/// Wraps a branch body and optionally emits the compound statement's terminator.
pub(super) fn emit_wrapped_branch_with_semicolon(
    emitter: &mut Emitter,
    branch: &Stmt,
    semicolon_after_end: bool,
    comments: &CommentMap,
) {
    if matches!(branch, Stmt::Block(..)) {
        emit_leading_comments(emitter, comments, stmt_start(branch), false);
    }
    emitter.writeln("begin");
    emitter.with_indent(|inner| match branch {
        Stmt::Block(stmts, ..) => super::emit_stmts_in_block(inner, stmts, comments),
        other => {
            emit_leading_comments(inner, comments, stmt_start(other), false);
            super::emit_stmt_in_block(inner, other, comments);
        }
    });
    write_indented(emitter);
    emitter.write("end");
    if semicolon_after_end {
        emitter.write(";");
    }
    emitter.write("\n");
}

/// Emits a case statement with terminated arm bodies.
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
            inner.writeln("else");
            if else_stmts.len() == 1 {
                emit_wrapped_branch_with_semicolon(inner, &else_stmts[0], true, comments);
            } else {
                inner.writeln("begin");
                inner.with_indent(|body| super::emit_stmts_in_block(body, else_stmts, comments));
                inner.writeln("end;");
            }
        }
    });

    write_indented(emitter);
    emitter.write("end");
}

/// Emits a case arm with a terminated compound statement body.
pub(super) fn emit_case_arm(emitter: &mut Emitter, arm: &CaseArm, comments: &CommentMap) {
    write_indented(emitter);
    emit_case_labels(emitter, &arm.labels, comments);
    if let Some(guard) = &arm.guard {
        emitter.write(" if ");
        emit_expr(emitter, guard, 0, comments);
    }
    emitter.write(":\n");
    emit_wrapped_branch_with_semicolon(emitter, &arm.body, true, comments);
}

pub(super) fn emit_case_labels(emitter: &mut Emitter, labels: &[CaseLabel], comments: &CommentMap) {
    for (index, label) in labels.iter().enumerate() {
        if index > 0 {
            emitter.write(", ");
        }
        emit_case_label(emitter, label, comments);
    }
}

pub(super) fn emit_case_label(emitter: &mut Emitter, label: &CaseLabel, comments: &CommentMap) {
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
