//! Control-flow statements (`if`, `case`, loops).

use fpas_parser::{CaseArm, CaseLabel, DestructureVariant, ForDirection, Stmt};

use crate::comments::{CommentMap, emit_leading_comments, stmt_start};

use super::super::Emitter;
use super::super::expr::emit_expr;
use super::super::types::emit_type_expr;
use super::line::write_indented;

pub(super) fn emit_if(emitter: &mut Emitter, stmt: &Stmt, comments: &CommentMap) {
    let Stmt::If {
        condition,
        then_branch,
        elsif_branches,
        else_branch,
        ..
    } = stmt
    else {
        return;
    };

    write_indented(emitter);
    emitter.write("if ");
    emit_expr(emitter, condition, 0, comments);
    emitter.write(" then\n");
    emit_statement_body(emitter, then_branch, comments);

    for (condition, body) in elsif_branches {
        write_indented(emitter);
        emitter.write("elsif ");
        emit_expr(emitter, condition, 0, comments);
        emitter.write(" then\n");
        emit_statement_body(emitter, body, comments);
    }

    match else_branch {
        Some(else_branch) => {
            emitter.writeln("else");
            emit_statement_body(emitter, else_branch, comments);
        }
        None => {}
    }
    emitter.writeln("end if");
}

pub(super) fn emit_statement_body(emitter: &mut Emitter, branch: &Stmt, comments: &CommentMap) {
    emitter.with_indent(|inner| match branch {
        Stmt::StatementList(stmts, ..) => super::emit_stmts_in_block(inner, stmts, comments),
        other => {
            emit_leading_comments(inner, comments, stmt_start(other), false);
            super::emit_stmt_in_block(inner, other, comments);
        }
    });
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
        for arm in arms {
            emit_case_arm(inner, arm, comments);
        }

        if let Some(else_stmts) = else_body {
            inner.writeln("else");
            inner.with_indent(|body| super::emit_stmts_in_block(body, else_stmts, comments));
        }
    });

    write_indented(emitter);
    emitter.write("end case");
}

pub(super) fn emit_case_arm(emitter: &mut Emitter, arm: &CaseArm, comments: &CommentMap) {
    write_indented(emitter);
    emitter.write("when ");
    emit_case_labels(emitter, &arm.labels, comments);
    if let Some(guard) = &arm.guard {
        emitter.write(" if ");
        emit_expr(emitter, guard, 0, comments);
    }
    emitter.write(":\n");
    emit_statement_body(emitter, &arm.body, comments);
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
            emit_statement_body(emitter, body, comments);
            emitter.writeln("end for");
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
            emit_statement_body(emitter, body, comments);
            emitter.writeln("end for");
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
    emit_statement_body(emitter, body, comments);
    emitter.writeln("end while");
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
