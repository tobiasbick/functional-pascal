//! Control-flow statements (`if`, `case`, loops).

use fpas_parser::{CaseArm, CaseLabel, DestructureVariant, ForDirection, Stmt};

use crate::comments::{CommentMap, emit_leading_comments, stmt_start};

use super::super::Emitter;
use super::super::expr::emit_expr;
use super::super::types::emit_type_expr;
use super::line::write_indented;

pub(super) fn emit_if(emitter: &mut Emitter, stmt: &Stmt, prefix: &str, comments: &CommentMap) {
    let Stmt::If {
        condition,
        then_branch,
        else_branch,
        ..
    } = stmt
    else {
        return;
    };

    write_indented(emitter);
    emitter.write(prefix);
    emitter.write("if ");
    emit_expr(emitter, condition, 0, comments);
    emitter.write(" then\n");
    emit_wrapped_branch(emitter, then_branch, comments);

    match else_branch {
        Some(else_branch) if matches!(else_branch.as_ref(), Stmt::If { .. }) => {
            emit_leading_comments(emitter, comments, stmt_start(else_branch), false);
            emit_if(emitter, else_branch, "else ", comments);
        }
        Some(else_branch) => {
            emitter.writeln("else");
            emit_wrapped_branch(emitter, else_branch, comments);
        }
        None => {}
    }
}

pub(super) fn emit_wrapped_branch(emitter: &mut Emitter, branch: &Stmt, comments: &CommentMap) {
    emit_wrapped_branch_with_semicolon(emitter, branch, false, comments);
}

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
            super::emit_stmt_in_block(inner, other, true, comments);
        }
    });
    write_indented(emitter);
    emitter.write("end");
    if semicolon_after_end {
        emitter.write(";");
    }
    emitter.write("\n");
}

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
        for (index, arm) in arms.iter().enumerate() {
            let is_last_arm = index + 1 == arms.len();
            emit_case_arm(inner, arm, is_last_arm, comments);
        }

        if let Some(else_stmts) = else_body {
            inner.writeln("else");
            if else_stmts.len() == 1 {
                emit_wrapped_branch_with_semicolon(inner, &else_stmts[0], false, comments);
            } else {
                inner.writeln("begin");
                inner.with_indent(|body| super::emit_stmts_in_block(body, else_stmts, comments));
                inner.writeln("end");
            }
        }
    });

    write_indented(emitter);
    emitter.write("end");
}

pub(super) fn emit_case_arm(
    emitter: &mut Emitter,
    arm: &CaseArm,
    is_last_arm: bool,
    comments: &CommentMap,
) {
    write_indented(emitter);
    emit_case_labels(emitter, &arm.labels, comments);
    if let Some(guard) = &arm.guard {
        emitter.write(" if ");
        emit_expr(emitter, guard, 0, comments);
    }
    emitter.write(":\n");
    emit_wrapped_branch_with_semicolon(emitter, &arm.body, !is_last_arm, comments);
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

pub(super) fn emit_for(emitter: &mut Emitter, stmt: &Stmt, comments: &CommentMap) {
    match stmt {
        Stmt::For {
            var_name,
            var_type,
            start,
            direction,
            end,
            body,
            ..
        } => {
            write_indented(emitter);
            emitter.write("for ");
            emitter.write(var_name);
            emitter.write(": ");
            emit_type_expr(emitter, var_type);
            emitter.write(" := ");
            emit_expr(emitter, start, 0, comments);
            emitter.write(" ");
            emitter.write(match direction {
                ForDirection::To => "to",
                ForDirection::Downto => "downto",
            });
            emitter.write(" ");
            emit_expr(emitter, end, 0, comments);
            emitter.write(" do\n");
            emit_wrapped_branch(emitter, body, comments);
        }
        Stmt::ForIn {
            var_name,
            var_type,
            iterable,
            body,
            ..
        } => {
            write_indented(emitter);
            emitter.write("for ");
            emitter.write(var_name);
            emitter.write(": ");
            emit_type_expr(emitter, var_type);
            emitter.write(" in ");
            emit_expr(emitter, iterable, 0, comments);
            emitter.write(" do\n");
            emit_wrapped_branch(emitter, body, comments);
        }
        _ => {}
    }
}

pub(super) fn emit_while(emitter: &mut Emitter, stmt: &Stmt, comments: &CommentMap) {
    let Stmt::While {
        condition, body, ..
    } = stmt
    else {
        return;
    };

    write_indented(emitter);
    emitter.write("while ");
    emit_expr(emitter, condition, 0, comments);
    emitter.write(" do\n");
    emit_wrapped_branch(emitter, body, comments);
}

pub(super) fn emit_repeat(emitter: &mut Emitter, stmt: &Stmt, comments: &CommentMap) {
    let Stmt::Repeat {
        body, condition, ..
    } = stmt
    else {
        return;
    };

    emitter.writeln("repeat");
    emitter.with_indent(|inner| super::emit_stmts_in_block(inner, body, comments));
    write_indented(emitter);
    emitter.write("until ");
    emit_expr(emitter, condition, 0, comments);
}
