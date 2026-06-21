//! Expressions and designators.

use fpas_parser::{BinaryOp, Designator, DesignatorPart, Expr, FieldInit, UnaryOp};

use crate::style::INDENT_WIDTH;

use super::Emitter;
use super::wrap::{exceeds_width, measure_emit, text_width};

/// Formats an expression.
#[must_use]
pub(crate) fn format_expr(expr: &Expr) -> String {
    let mut emitter = Emitter::new();
    emit_expr(&mut emitter, expr, 0);
    emitter.finish()
}

pub(crate) fn emit_expr(emitter: &mut Emitter, expr: &Expr, min_prec: u8) {
    emit_expr_impl(emitter, expr, min_prec, min_prec == 0);
}

fn emit_expr_impl(emitter: &mut Emitter, expr: &Expr, min_prec: u8, allow_wrap: bool) {
    if allow_wrap {
        let base_column = emitter.column();
        if matches!(expr, Expr::BinaryOp { .. }) {
            let rendered = measure_emit(|inner| emit_expr_impl(inner, expr, 0, false));
            if exceeds_width(base_column, text_width(&rendered)) {
                // The break emitter must receive the complete expression so no surrounding
                // operators or parentheses are discarded when a nested operator has lower precedence.
                emit_binary_with_break(emitter, expr, base_column);
                return;
            }
        }
    }

    match expr {
        Expr::Integer(value, ..) => emitter.write(&value.to_string()),
        Expr::Real(value, ..) => emitter.write(&format_real(*value)),
        Expr::Str(value, ..) => emitter.write(&format_string(value)),
        Expr::Bool(value, ..) => emitter.write(if *value { "true" } else { "false" }),
        Expr::Designator(designator) => emit_designator(emitter, designator),
        Expr::Call {
            designator, args, ..
        } => {
            emit_designator(emitter, designator);
            emitter.write("(");
            emit_arg_list(emitter, args);
            emitter.write(")");
        }
        Expr::UnaryOp { op, operand, .. } => {
            let prec = 4;
            if prec < min_prec {
                emitter.write("(");
                emit_expr_impl(emitter, expr, 0, false);
                emitter.write(")");
                return;
            }
            match op {
                UnaryOp::Not => {
                    emitter.write("not ");
                    emit_expr_impl(emitter, operand, prec, false);
                }
                UnaryOp::Negate => {
                    emitter.write("-");
                    if needs_space_after_negate(operand) {
                        emitter.write(" ");
                    }
                    emit_expr_impl(emitter, operand, prec, false);
                }
            }
        }
        Expr::BinaryOp {
            op, left, right, ..
        } => {
            let prec = binary_prec(*op);
            if prec < min_prec {
                emitter.write("(");
                emit_expr_impl(emitter, expr, 0, false);
                emitter.write(")");
                return;
            }
            emit_expr_impl(emitter, left, prec + 1, false);
            emitter.write(binary_op_spaced(*op));
            emit_expr_impl(emitter, right, prec, false);
        }
        Expr::Paren(inner, ..) => {
            emitter.write("(");
            emit_expr_impl(emitter, inner, 0, false);
            emitter.write(")");
        }
        Expr::ArrayLiteral(elements, ..) => emit_array_literal(emitter, elements),
        Expr::DictLiteral(pairs, ..) => {
            if pairs.is_empty() {
                emitter.write("[:]");
                return;
            }
            emitter.write("[");
            for (index, (key, value)) in pairs.iter().enumerate() {
                if index > 0 {
                    emitter.write(", ");
                }
                emit_expr(emitter, key, 0);
                emitter.write(": ");
                emit_expr(emitter, value, 0);
            }
            emitter.write("]");
        }
        Expr::RecordLiteral { fields, .. } => emit_record_fields(emitter, fields),
        Expr::RecordUpdate { base, fields, .. } => {
            emit_expr(emitter, base, 0);
            emitter.write(" with ");
            emit_record_field_inits(emitter, fields);
            emitter.write(record_literal_end(fields));
        }
        Expr::ResultOk(inner, ..) => {
            emitter.write("Ok(");
            emit_expr(emitter, inner, 0);
            emitter.write(")");
        }
        Expr::ResultError(inner, ..) => {
            emitter.write("Error(");
            emit_expr(emitter, inner, 0);
            emitter.write(")");
        }
        Expr::OptionSome(inner, ..) => {
            emitter.write("Some(");
            emit_expr(emitter, inner, 0);
            emitter.write(")");
        }
        Expr::OptionNone(..) => emitter.write("None"),
        Expr::Try(inner, ..) => {
            emitter.write("try ");
            emit_expr(emitter, inner, 4);
        }
        Expr::Go(inner, ..) => {
            emitter.write("go ");
            emit_expr(emitter, inner, 0);
        }
        Expr::Error(..) => emitter.write("<error>"),
    }
}

pub(crate) fn emit_designator(emitter: &mut Emitter, designator: &Designator) {
    for (index, part) in designator.parts.iter().enumerate() {
        match part {
            DesignatorPart::Ident(name, ..) => {
                if index > 0 {
                    match &designator.parts[index - 1] {
                        DesignatorPart::Ident(..) => emitter.write("."),
                        DesignatorPart::Index(..) => {}
                    }
                }
                emitter.write(name);
            }
            DesignatorPart::Index(index_expr, ..) => {
                emitter.write("[");
                emit_expr(emitter, index_expr, 0);
                emitter.write("]");
            }
        }
    }
}

pub(crate) fn emit_arg_list(emitter: &mut Emitter, args: &[Expr]) {
    for (index, arg) in args.iter().enumerate() {
        if index > 0 {
            emitter.write(", ");
        }
        emit_expr(emitter, arg, 0);
    }
}

fn emit_record_fields(emitter: &mut Emitter, fields: &[FieldInit]) {
    let inline = measure_emit(|inner| {
        inner.write("record ");
        emit_record_field_inits(inner, fields);
        inner.write(record_literal_end(fields));
    });
    if !exceeds_width(emitter.column(), text_width(&inline)) {
        emitter.write("record ");
        emit_record_field_inits(emitter, fields);
        emitter.write(record_literal_end(fields));
        return;
    }

    let base_column = emitter.indent_level() * INDENT_WIDTH;
    let field_column = base_column + INDENT_WIDTH;
    emitter.write("record\n");
    for (index, field) in fields.iter().enumerate() {
        if index > 0 {
            emitter.write("\n");
        }
        write_to_column(emitter, field_column);
        emitter.write(&field.name);
        emitter.write(" := ");
        emit_expr_impl(emitter, &field.value, 0, false);
        emitter.write(";");
    }
    emitter.write("\n");
    write_to_column(emitter, base_column);
    emitter.write("end");
}

fn write_to_column(emitter: &mut Emitter, column: usize) {
    let pad = column.saturating_sub(emitter.column());
    emitter.write(&" ".repeat(pad));
}

fn emit_array_literal(emitter: &mut Emitter, elements: &[Expr]) {
    let items: Vec<String> = elements
        .iter()
        .map(|element| measure_emit(|inner| emit_expr_impl(inner, element, 0, false)))
        .collect();
    if items.is_empty() {
        emitter.write("[]");
        return;
    }

    let single_line = format!("[{}]", items.join(", "));
    if !exceeds_width(emitter.column(), text_width(&single_line)) {
        emitter.write(&single_line);
        return;
    }

    let base_column = emitter.column();
    let item_column = base_column + INDENT_WIDTH;
    emitter.write("[\n");
    for (index, item) in items.iter().enumerate() {
        write_block_at_column(emitter, item_column, item);
        if index + 1 < items.len() {
            emitter.write(",");
        }
        emitter.write("\n");
    }
    emitter.newline_to_column(base_column);
    emitter.write("]");
}

fn write_block_at_column(emitter: &mut Emitter, column: usize, text: &str) {
    for (index, line) in text.split('\n').enumerate() {
        if index > 0 {
            emitter.write("\n");
            emitter.newline_to_column(column);
        } else {
            let pad = column.saturating_sub(emitter.column());
            emitter.write(&" ".repeat(pad));
        }
        emitter.write(line);
    }
}

fn emit_binary_with_break(emitter: &mut Emitter, expr: &Expr, base_column: usize) {
    let Expr::BinaryOp {
        op, left, right, ..
    } = expr
    else {
        emit_expr_impl(emitter, expr, 0, false);
        return;
    };
    let prec = binary_prec(*op);
    emit_expr_impl(emitter, left, prec + 1, false);
    let op_token = binary_op_spaced(*op).trim();
    emitter.write(" ");
    emitter.write(op_token);
    emitter.newline_to_column(base_column);
    emit_expr_impl(emitter, right, prec, false);
}

fn emit_record_field_inits(emitter: &mut Emitter, fields: &[FieldInit]) {
    for (index, field) in fields.iter().enumerate() {
        if index > 0 {
            emitter.write("; ");
        }
        emitter.write(&field.name);
        emitter.write(" := ");
        emit_expr(emitter, &field.value, 0);
    }
}

fn record_literal_end(fields: &[FieldInit]) -> &'static str {
    if fields.is_empty() { " end" } else { "; end" }
}

fn format_string(value: &str) -> String {
    let mut out = String::from("'");
    for ch in value.chars() {
        if ch == '\'' {
            out.push_str("''");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    out
}

fn format_real(value: f64) -> String {
    if value.is_finite() && value.fract() == 0.0 && value.abs() < 1e15 {
        return format!("{value:.1}");
    }
    let text = format!("{value}");
    if text.contains('.') || text.contains('e') || text.contains('E') {
        return text;
    }
    format!("{text}.0")
}

fn needs_space_after_negate(operand: &Expr) -> bool {
    !matches!(
        operand,
        Expr::Integer(..) | Expr::Real(..) | Expr::Paren(..)
    )
}

fn binary_prec(op: BinaryOp) -> u8 {
    match op {
        BinaryOp::Mul
        | BinaryOp::RealDiv
        | BinaryOp::IntDiv
        | BinaryOp::Mod
        | BinaryOp::And
        | BinaryOp::Shl
        | BinaryOp::Shr => 3,
        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Or | BinaryOp::Xor => 2,
        BinaryOp::Eq
        | BinaryOp::NotEq
        | BinaryOp::Lt
        | BinaryOp::Gt
        | BinaryOp::LtEq
        | BinaryOp::GtEq
        | BinaryOp::In => 1,
    }
}

fn binary_op_spaced(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Mul => " * ",
        BinaryOp::RealDiv => " / ",
        BinaryOp::IntDiv => " div ",
        BinaryOp::Mod => " mod ",
        BinaryOp::And => " and ",
        BinaryOp::Shl => " shl ",
        BinaryOp::Shr => " shr ",
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

#[cfg(test)]
mod tests {
    use super::format_expr;
    use fpas_parser::{Stmt, parse};

    fn expr_from_body(source: &str) -> String {
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "{errors:?}");
        let Stmt::Var(var) = &program.body[0] else {
            panic!("expected var stmt");
        };
        format_expr(&var.value)
    }

    #[test]
    fn literals_and_designators() {
        assert_eq!(
            expr_from_body("program T; begin var X: integer := 42; end."),
            "42"
        );
        assert_eq!(
            expr_from_body("program T; begin var X: real := 3.14; end."),
            "3.14"
        );
        assert_eq!(
            expr_from_body("program T; begin var X: string := 'hi'; end."),
            "'hi'"
        );
        assert_eq!(
            expr_from_body("program T; begin var X: boolean := true; end."),
            "true"
        );
        assert_eq!(
            expr_from_body(
                "program T; begin var X: procedure(Msg: string) := Std.Console.WriteLn; end."
            ),
            "Std.Console.WriteLn"
        );
    }

    #[test]
    fn operators_and_calls() {
        assert_eq!(
            expr_from_body("program T; begin var X: integer := 1 + 2 * 3; end."),
            "1 + 2 * 3"
        );
        assert_eq!(
            expr_from_body("program T; begin var X: string := IntToStr(42); end."),
            "IntToStr(42)"
        );
        assert_eq!(
            expr_from_body("program T; begin var X: boolean := not true; end."),
            "not true"
        );
    }

    #[test]
    fn aggregates_and_wrappers() {
        assert_eq!(
            expr_from_body("program T; begin var X: array of integer := [1, 2, 3]; end."),
            "[1, 2, 3]"
        );
        assert_eq!(
            expr_from_body("program T; begin var X: dict of string to integer := ['a': 1]; end."),
            "['a': 1]"
        );
        assert_eq!(
            expr_from_body(
                "program T; type Point = record X: integer; Y: integer; end; begin var X: Point := record X := 1; Y := 2; end; end."
            ),
            "record X := 1; Y := 2; end"
        );
        assert_eq!(
            expr_from_body("program T; begin var X: result of integer, string := Ok(42); end."),
            "Ok(42)"
        );
        assert_eq!(
            expr_from_body("program T; begin var X: option of integer := None; end."),
            "None"
        );
    }

    #[test]
    fn wrapped_record_literal() {
        let formatted = expr_from_body(
            "program T; type Config = record Host: string; Port: integer; Retries: integer; TimeoutSeconds: integer; Extra: string; end; begin var X: Config := record Host := 'https://api.example.com/v2/production'; Port := 443; Retries := 5; TimeoutSeconds := 30; Extra := 'metadata'; end; end.",
        );
        assert!(formatted.contains("record\n  Host := 'https://api.example.com/v2/production';"));
    }

    #[test]
    fn long_binary_chain_wraps() {
        let formatted = expr_from_body(
            "program T; begin var X: boolean := VeryLongIdentifierAlpha + VeryLongIdentifierBeta + VeryLongIdentifierGamma + VeryLongIdentifierDelta + VeryLongIdentifierEpsilon; end.",
        );
        assert!(formatted.contains(" +\n"), "formatted: {formatted}");
    }
}
