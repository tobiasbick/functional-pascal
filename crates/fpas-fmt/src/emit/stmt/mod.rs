//! Statement and block emission.

mod line;
mod loops;
mod spacing;

use fpas_parser::Stmt;

use crate::comments::{CommentMap, emit_leading_comments, stmt_start};

use super::Emitter;
use super::expr::{emit_arg_list, emit_designator, emit_expr};
use line::{finish_stmt_after_newline, finish_stmt_line, write_indented};

/// Formats a statement list as it appears inside `begin` … `end`.
#[must_use]
pub(crate) fn format_block_stmts(stmts: &[Stmt]) -> String {
    let mut emitter = Emitter::new();
    emit_stmts_in_block(&mut emitter, stmts, &CommentMap::default());
    emitter.finish()
}

pub(crate) fn emit_stmts_in_block(emitter: &mut Emitter, stmts: &[Stmt], comments: &CommentMap) {
    for (index, stmt) in stmts.iter().enumerate() {
        if index > 0 && spacing::needs_blank_line(&stmts[index - 1], stmt) {
            emitter.blank_line();
        }
        emit_leading_comments(emitter, comments, stmt_start(stmt), false);
        let is_last = index + 1 == stmts.len();
        emit_stmt_in_block(emitter, stmt, is_last, comments);
    }
}

fn emit_stmt_in_block(emitter: &mut Emitter, stmt: &Stmt, is_last: bool, comments: &CommentMap) {
    match stmt {
        Stmt::Block(stmts, ..) => {
            emitter.writeln("begin");
            emitter.with_indent(|inner| emit_stmts_in_block(inner, stmts, comments));
            emitter.writeln("end");
            finish_stmt_after_newline(emitter, comments, stmt, is_last);
        }
        Stmt::Var(var) => line::emit_var_stmt(emitter, "var", var, is_last, comments),
        Stmt::MutableVar(var) => {
            line::emit_var_stmt(emitter, "mutable var", var, is_last, comments)
        }
        Stmt::Assign { target, value, .. } => {
            write_indented(emitter);
            emit_designator(emitter, target, comments);
            emitter.write(" := ");
            emit_expr(emitter, value, 0, comments);
            finish_stmt_line(emitter, comments, stmt, is_last);
        }
        Stmt::Return(expr, ..) => {
            write_indented(emitter);
            emitter.write("return");
            if let Some(value) = expr {
                emitter.write(" ");
                emit_expr(emitter, value, 0, comments);
            }
            finish_stmt_line(emitter, comments, stmt, is_last);
        }
        Stmt::Panic(expr, ..) => {
            write_indented(emitter);
            emitter.write("panic(");
            emit_expr(emitter, expr, 0, comments);
            emitter.write(")");
            finish_stmt_line(emitter, comments, stmt, is_last);
        }
        Stmt::If { .. } => {
            loops::emit_if(emitter, stmt, "", comments);
            finish_stmt_after_newline(emitter, comments, stmt, is_last);
        }
        Stmt::Case { .. } => {
            loops::emit_case(emitter, stmt, comments);
            finish_stmt_line(emitter, comments, stmt, is_last);
        }
        Stmt::For { .. } | Stmt::ForIn { .. } => {
            loops::emit_for(emitter, stmt, comments);
            finish_stmt_after_newline(emitter, comments, stmt, is_last);
        }
        Stmt::While { .. } => {
            loops::emit_while(emitter, stmt, comments);
            finish_stmt_after_newline(emitter, comments, stmt, is_last);
        }
        Stmt::Repeat { .. } => {
            loops::emit_repeat(emitter, stmt, comments);
            finish_stmt_line(emitter, comments, stmt, is_last);
        }
        Stmt::Break(..) => {
            write_indented(emitter);
            emitter.write("break");
            finish_stmt_line(emitter, comments, stmt, is_last);
        }
        Stmt::Continue(..) => {
            write_indented(emitter);
            emitter.write("continue");
            finish_stmt_line(emitter, comments, stmt, is_last);
        }
        Stmt::Call {
            designator, args, ..
        } => {
            write_indented(emitter);
            emit_designator(emitter, designator, comments);
            emitter.write("(");
            emit_arg_list(emitter, args, comments);
            emitter.write(")");
            finish_stmt_line(emitter, comments, stmt, is_last);
        }
        Stmt::Expression { expr, .. } => {
            write_indented(emitter);
            emit_expr(emitter, expr, 0, comments);
            finish_stmt_line(emitter, comments, stmt, is_last);
        }
        Stmt::Go { expr, .. } => {
            write_indented(emitter);
            emitter.write("go ");
            emit_expr(emitter, expr, 0, comments);
            finish_stmt_line(emitter, comments, stmt, is_last);
        }
    }
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
    fn case_else_with_block_body_is_idempotent() {
        let source =
            "program T; begin case X of 1: WriteLn('one') else begin WriteLn('other') end end end.";
        let formatted_once = format_body(source);
        let formatted_twice = format_body(&format!(
            "program T; begin {} end.",
            formatted_once.trim_end()
        ));
        assert_eq!(
            formatted_once, formatted_twice,
            "once:\n{formatted_once}\ntwice:\n{formatted_twice}"
        );
        assert!(
            !formatted_once.contains("begin\n    begin\n    begin"),
            "formatted:\n{formatted_once}"
        );
    }

    #[test]
    fn multiline_string_literal_places_semicolon_after_closing_paren() {
        let formatted = format_body(
            "program T; begin
  WriteLn('line1
line2');
  WriteLn('after')
end.",
        );
        assert!(
            formatted.contains("line2');"),
            "semicolon must follow the closing paren, not an interior newline; formatted:\n{formatted}"
        );
        assert!(
            formatted.contains("WriteLn('after')"),
            "formatted:\n{formatted}"
        );
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

    #[test]
    fn postfix_procedure_statement_round_trips() {
        let source = "program T; begin Factory.Create().Transform().Destroy() end.";
        let formatted = format_body(source);
        assert_eq!(formatted, "Factory.Create().Transform().Destroy()\n");

        let reparsed = format!("program T; begin {formatted} end.");
        let (_, errors) = fpas_parser::parse(&reparsed);
        assert!(errors.is_empty(), "{errors:?}");
    }
}
