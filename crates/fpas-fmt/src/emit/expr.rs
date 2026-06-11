//! Expressions and designators.

use fpas_parser::{BinaryOp, Designator, DesignatorPart, Expr, FieldInit, UnaryOp};

use super::Emitter;

/// Formats an expression.
#[must_use]
pub(crate) fn format_expr(expr: &Expr) -> String {
    let mut emitter = Emitter::new();
    emit_expr(&mut emitter, expr, 0);
    emitter.finish()
}

pub(crate) fn emit_expr(emitter: &mut Emitter, expr: &Expr, min_prec: u8) {
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
                emit_expr(emitter, expr, 0);
                emitter.write(")");
                return;
            }
            match op {
                UnaryOp::Not => {
                    emitter.write("not ");
                    emit_expr(emitter, operand, prec);
                }
                UnaryOp::Negate => {
                    emitter.write("-");
                    if needs_space_after_negate(operand) {
                        emitter.write(" ");
                    }
                    emit_expr(emitter, operand, prec);
                }
            }
        }
        Expr::BinaryOp {
            op, left, right, ..
        } => {
            let prec = binary_prec(*op);
            if prec < min_prec {
                emitter.write("(");
                emit_expr(emitter, expr, 0);
                emitter.write(")");
                return;
            }
            emit_expr(emitter, left, prec + 1);
            emitter.write(binary_op_spaced(*op));
            emit_expr(emitter, right, prec);
        }
        Expr::Paren(inner, ..) => {
            emitter.write("(");
            emit_expr(emitter, inner, 0);
            emitter.write(")");
        }
        Expr::ArrayLiteral(elements, ..) => {
            emitter.write("[");
            for (index, element) in elements.iter().enumerate() {
                if index > 0 {
                    emitter.write(", ");
                }
                emit_expr(emitter, element, 0);
            }
            emitter.write("]");
        }
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
    emitter.write("record ");
    emit_record_field_inits(emitter, fields);
    emitter.write(record_literal_end(fields));
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
}
