//! Statements.

use fpas_parser::{CaseArm, CaseLabel, DestructureVariant, ForDirection, Stmt, VarDef};

use super::Emitter;
use super::expr::{emit_arg_list, emit_designator, emit_expr};
use super::types::emit_type_expr;

/// Formats a statement list as it appears inside `begin` … `end`.
#[must_use]
pub(crate) fn format_block_stmts(stmts: &[Stmt]) -> String {
    let mut emitter = Emitter::new();
    emit_stmts_in_block(&mut emitter, stmts);
    emitter.finish()
}

/// Formats a single statement (no trailing block semicolon rules).
#[must_use]
pub(crate) fn format_stmt(stmt: &Stmt) -> String {
    let mut emitter = Emitter::new();
    emit_stmt_in_block(&mut emitter, stmt, true);
    emitter.finish()
}

fn emit_stmts_in_block(emitter: &mut Emitter, stmts: &[Stmt]) {
    for (index, stmt) in stmts.iter().enumerate() {
        let is_last = index + 1 == stmts.len();
        emit_stmt_in_block(emitter, stmt, is_last);
    }
}

fn emit_stmt_in_block(emitter: &mut Emitter, stmt: &Stmt, is_last: bool) {
    match stmt {
        Stmt::Block(stmts, ..) => {
            emitter.writeln("begin");
            emitter.with_indent(|inner| emit_stmts_in_block(inner, stmts));
            emitter.writeln("end");
            emit_trailing_semicolon(emitter, is_last);
        }
        Stmt::Var(var) => emit_var_stmt(emitter, "var", var, is_last),
        Stmt::MutableVar(var) => emit_var_stmt(emitter, "mutable var", var, is_last),
        Stmt::Assign { target, value, .. } => {
            write_indented(emitter);
            emit_designator(emitter, target);
            emitter.write(" := ");
            emit_expr(emitter, value, 0);
            emit_trailing_semicolon(emitter, is_last);
        }
        Stmt::Return(expr, ..) => {
            write_indented(emitter);
            emitter.write("return");
            if let Some(value) = expr {
                emitter.write(" ");
                emit_expr(emitter, value, 0);
            }
            emit_trailing_semicolon(emitter, is_last);
        }
        Stmt::Panic(expr, ..) => {
            write_indented(emitter);
            emitter.write("panic(");
            emit_expr(emitter, expr, 0);
            emitter.write(")");
            emit_trailing_semicolon(emitter, is_last);
        }
        Stmt::If { .. } => {
            emit_if(emitter, stmt, "");
            emit_trailing_semicolon(emitter, is_last);
        }
        Stmt::Case { .. } => {
            emit_case(emitter, stmt);
            emit_trailing_semicolon(emitter, is_last);
        }
        Stmt::For { .. } | Stmt::ForIn { .. } => {
            emit_for(emitter, stmt);
            emit_trailing_semicolon(emitter, is_last);
        }
        Stmt::While { .. } => {
            emit_while(emitter, stmt);
            emit_trailing_semicolon(emitter, is_last);
        }
        Stmt::Repeat { .. } => {
            emit_repeat(emitter, stmt);
            emit_trailing_semicolon(emitter, is_last);
        }
        Stmt::Break(..) => {
            write_indented(emitter);
            emitter.write("break");
            emit_trailing_semicolon(emitter, is_last);
        }
        Stmt::Continue(..) => {
            write_indented(emitter);
            emitter.write("continue");
            emit_trailing_semicolon(emitter, is_last);
        }
        Stmt::Call {
            designator, args, ..
        } => {
            write_indented(emitter);
            emit_designator(emitter, designator);
            emitter.write("(");
            emit_arg_list(emitter, args);
            emitter.write(")");
            emit_trailing_semicolon(emitter, is_last);
        }
        Stmt::Go { expr, .. } => {
            write_indented(emitter);
            emitter.write("go ");
            emit_expr(emitter, expr, 0);
            emit_trailing_semicolon(emitter, is_last);
        }
    }
}

fn emit_var_stmt(emitter: &mut Emitter, keyword: &str, var: &VarDef, is_last: bool) {
    write_indented(emitter);
    emitter.write(keyword);
    emitter.write(" ");
    emitter.write(&var.name);
    emitter.write(": ");
    emit_type_expr(emitter, &var.type_expr);
    emitter.write(" := ");
    emit_expr(emitter, &var.value, 0);
    emit_trailing_semicolon(emitter, is_last);
}

fn emit_if(emitter: &mut Emitter, stmt: &Stmt, prefix: &str) {
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
    emit_expr(emitter, condition, 0);
    emitter.write(" then\n");
    emit_wrapped_branch(emitter, then_branch);

    match else_branch {
        Some(else_branch) if matches!(else_branch.as_ref(), Stmt::If { .. }) => {
            emit_if(emitter, else_branch, "else ");
        }
        Some(else_branch) => {
            emitter.writeln("else");
            emit_wrapped_branch(emitter, else_branch);
        }
        None => {}
    }
}

fn emit_wrapped_branch(emitter: &mut Emitter, branch: &Stmt) {
    emit_wrapped_branch_with_semicolon(emitter, branch, false);
}

fn emit_wrapped_branch_with_semicolon(
    emitter: &mut Emitter,
    branch: &Stmt,
    semicolon_after_end: bool,
) {
    emitter.writeln("begin");
    emitter.with_indent(|inner| match branch {
        Stmt::Block(stmts, ..) => emit_stmts_in_block(inner, stmts),
        other => emit_stmt_in_block(inner, other, true),
    });
    write_indented(emitter);
    emitter.write("end");
    if semicolon_after_end {
        emitter.write(";");
    }
    emitter.write("\n");
}

fn emit_case(emitter: &mut Emitter, stmt: &Stmt) {
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
    emit_expr(emitter, expr, 0);
    emitter.write(" of\n");

    emitter.with_indent(|inner| {
        for (index, arm) in arms.iter().enumerate() {
            let is_last_arm = index + 1 == arms.len();
            emit_case_arm(inner, arm, is_last_arm);
        }

        if let Some(else_stmts) = else_body {
            inner.writeln("else");
            inner.writeln("begin");
            inner.with_indent(|body| emit_stmts_in_block(body, else_stmts));
            inner.writeln("end");
        }
    });

    write_indented(emitter);
    emitter.write("end");
}

fn emit_case_arm(emitter: &mut Emitter, arm: &CaseArm, is_last_arm: bool) {
    write_indented(emitter);
    emit_case_labels(emitter, &arm.labels);
    if let Some(guard) = &arm.guard {
        emitter.write(" if ");
        emit_expr(emitter, guard, 0);
    }
    emitter.write(":\n");
    emit_wrapped_branch_with_semicolon(emitter, &arm.body, !is_last_arm);
}

fn emit_case_labels(emitter: &mut Emitter, labels: &[CaseLabel]) {
    for (index, label) in labels.iter().enumerate() {
        if index > 0 {
            emitter.write(", ");
        }
        emit_case_label(emitter, label);
    }
}

fn emit_case_label(emitter: &mut Emitter, label: &CaseLabel) {
    match label {
        CaseLabel::Value { start, end, .. } => {
            emit_expr(emitter, start, 0);
            if let Some(end_expr) = end {
                emitter.write("..");
                emit_expr(emitter, end_expr, 0);
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

fn emit_for(emitter: &mut Emitter, stmt: &Stmt) {
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
            emit_expr(emitter, start, 0);
            emitter.write(" ");
            emitter.write(match direction {
                ForDirection::To => "to",
                ForDirection::Downto => "downto",
            });
            emitter.write(" ");
            emit_expr(emitter, end, 0);
            emitter.write(" do\n");
            emit_wrapped_branch(emitter, body);
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
            emit_expr(emitter, iterable, 0);
            emitter.write(" do\n");
            emit_wrapped_branch(emitter, body);
        }
        _ => {}
    }
}

fn emit_while(emitter: &mut Emitter, stmt: &Stmt) {
    let Stmt::While {
        condition, body, ..
    } = stmt
    else {
        return;
    };

    write_indented(emitter);
    emitter.write("while ");
    emit_expr(emitter, condition, 0);
    emitter.write(" do\n");
    emit_wrapped_branch(emitter, body);
}

fn emit_repeat(emitter: &mut Emitter, stmt: &Stmt) {
    let Stmt::Repeat {
        body, condition, ..
    } = stmt
    else {
        return;
    };

    emitter.writeln("repeat");
    emitter.with_indent(|inner| emit_stmts_in_block(inner, body));
    write_indented(emitter);
    emitter.write("until ");
    emit_expr(emitter, condition, 0);
}

fn write_indented(emitter: &mut Emitter) {
    for _ in 0..emitter.indent_level() {
        emitter.write(crate::style::INDENT);
    }
}

fn emit_trailing_semicolon(emitter: &mut Emitter, is_last: bool) {
    if !is_last {
        emitter.write(";");
    }
    emitter.write("\n");
}

#[cfg(test)]
mod tests {
    use super::format_block_stmts;
    use fpas_parser::parse;

    fn format_body(source: &str) -> String {
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "{errors:?}");
        format_block_stmts(&program.body)
    }

    #[test]
    fn if_else_with_branch_blocks() {
        let formatted = format_body(
            "program T; begin
  if X > 0 then WriteLn('positive')
  else if X = 0 then WriteLn('zero')
  else WriteLn('negative')
end.",
        );
        assert!(formatted.contains("if X > 0 then\nbegin\n"));
        assert!(formatted.contains("else if X = 0 then\nbegin\n"));
        assert!(formatted.contains("else\nbegin\n"));
    }

    #[test]
    fn case_for_while_repeat() {
        let formatted = format_body(
            "program T; begin
  case Value of
    1: WriteLn('one');
    2, 3: WriteLn('two or three')
  else
    WriteLn('other')
  end;
  for I: integer := 1 to 3 do WriteLn(I);
  while X < 10 do X := X + 1;
  repeat WriteLn(N); N := N + 1 until N >= 3
end.",
        );
        assert!(formatted.contains("case Value of\n"));
        assert!(formatted.contains("1:\n"));
        assert!(formatted.contains("WriteLn('one')"));
        assert!(formatted.contains("end;\n"));
        assert!(formatted.contains("2, 3:\n"));
        assert!(formatted.contains("for I: integer := 1 to 3 do\n"));
        assert!(formatted.contains("while X < 10 do\n"));
        assert!(formatted.contains("repeat\n"));
        assert!(!formatted.contains("repeat\nbegin\n"));
        assert!(formatted.ends_with("until N >= 3\n"));
    }

    #[test]
    fn var_assign_call_return() {
        let formatted = format_body(
            "program T; begin
  var X: integer := 1;
  X := 2;
  WriteLn('hi');
  return X
end.",
        );
        assert_eq!(
            formatted,
            "var X: integer := 1;\n\
             X := 2;\n\
             WriteLn('hi');\n\
             return X\n"
        );
    }
}
