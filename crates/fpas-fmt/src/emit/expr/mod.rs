//! Expressions and designators.

mod binary;
mod closure;
mod literal;
mod postfix;

use fpas_parser::{Designator, DesignatorPart, Expr, UnaryOp};

use crate::comments::CommentMap;

use super::Emitter;
use super::wrap::{exceeds_width, measure_emit, text_width};
use binary::{binary_op_spaced, binary_prec, emit_binary_with_break};
use literal::{
    emit_array_literal, emit_record_field_inits, emit_record_fields, format_real, format_string,
    needs_space_after_negate, record_literal_end,
};
use postfix::emit_postfix;

/// Formats an expression.
#[must_use]
pub(crate) fn format_expr(expr: &Expr) -> String {
    let mut emitter = Emitter::new();
    emit_expr(&mut emitter, expr, 0, &CommentMap::default());
    emitter.finish()
}

pub(crate) fn emit_expr(emitter: &mut Emitter, expr: &Expr, min_prec: u8, comments: &CommentMap) {
    emit_expr_impl(emitter, expr, min_prec, min_prec == 0, comments);
}

pub(super) fn emit_expr_impl(
    emitter: &mut Emitter,
    expr: &Expr,
    min_prec: u8,
    allow_wrap: bool,
    comments: &CommentMap,
) {
    if allow_wrap {
        let base_column = emitter.column();
        if matches!(expr, Expr::BinaryOp { .. }) {
            let rendered = measure_emit(|inner| emit_expr_impl(inner, expr, 0, false, comments));
            if exceeds_width(base_column, text_width(&rendered)) {
                // The break emitter must receive the complete expression so no surrounding
                // operators or parentheses are discarded when a nested operator has lower precedence.
                emit_binary_with_break(emitter, expr, base_column, comments);
                return;
            }
        }
    }

    match expr {
        Expr::Integer(value, ..) => emitter.write(&value.to_string()),
        Expr::Real(value, ..) => emitter.write(&format_real(*value)),
        Expr::Str(value, ..) => emitter.write(&format_string(value)),
        Expr::Bool(value, ..) => emitter.write(if *value { "true" } else { "false" }),
        Expr::Designator(designator) => emit_designator(emitter, designator, comments),
        Expr::Call {
            designator, args, ..
        } => {
            emit_designator(emitter, designator, comments);
            emitter.write("(");
            emit_arg_list(emitter, args, comments);
            emitter.write(")");
        }
        Expr::UnaryOp { op, operand, .. } => {
            let prec = 4;
            if prec < min_prec {
                emitter.write("(");
                emit_expr_impl(emitter, expr, 0, false, comments);
                emitter.write(")");
                return;
            }
            match op {
                UnaryOp::Not => {
                    emitter.write("not ");
                    emit_expr_impl(emitter, operand, prec, false, comments);
                }
                UnaryOp::Negate => {
                    emitter.write("-");
                    if needs_space_after_negate(operand) {
                        emitter.write(" ");
                    }
                    emit_expr_impl(emitter, operand, prec, false, comments);
                }
            }
        }
        Expr::BinaryOp {
            op, left, right, ..
        } => {
            let prec = binary_prec(*op);
            if prec < min_prec {
                emitter.write("(");
                emit_expr_impl(emitter, expr, 0, false, comments);
                emitter.write(")");
                return;
            }
            emit_expr_impl(emitter, left, prec + 1, false, comments);
            emitter.write(binary_op_spaced(*op));
            emit_expr_impl(emitter, right, prec, false, comments);
        }
        Expr::Paren(inner, ..) => {
            emitter.write("(");
            emit_expr_impl(emitter, inner, 0, false, comments);
            emitter.write(")");
        }
        Expr::ArrayLiteral(elements, ..) => emit_array_literal(emitter, elements, comments),
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
                emit_expr(emitter, key, 0, comments);
                emitter.write(": ");
                emit_expr(emitter, value, 0, comments);
            }
            emitter.write("]");
        }
        Expr::RecordLiteral { fields, .. } => emit_record_fields(emitter, fields, comments),
        Expr::RecordUpdate { base, fields, .. } => {
            emit_expr(emitter, base, 0, comments);
            emitter.write(" with ");
            emit_record_field_inits(emitter, fields, comments);
            emitter.write(record_literal_end(fields));
        }
        Expr::ResultOk(inner, ..) => {
            emitter.write("Ok(");
            emit_expr(emitter, inner, 0, comments);
            emitter.write(")");
        }
        Expr::ResultError(inner, ..) => {
            emitter.write("Error(");
            emit_expr(emitter, inner, 0, comments);
            emitter.write(")");
        }
        Expr::OptionSome(inner, ..) => {
            emitter.write("Some(");
            emit_expr(emitter, inner, 0, comments);
            emitter.write(")");
        }
        Expr::OptionNone(..) => emitter.write("None"),
        Expr::Nil(..) => emitter.write("nil"),
        Expr::Try(inner, ..) => {
            emitter.write("try ");
            emit_expr(emitter, inner, 4, comments);
        }
        Expr::Go(inner, ..) => {
            emitter.write("go ");
            emit_expr(emitter, inner, 0, comments);
        }
        Expr::Postfix {
            base, operations, ..
        } => emit_postfix(emitter, base, operations, allow_wrap, comments),
        Expr::Closure(closure) => closure::emit_closure(
            emitter,
            closure.is_function,
            &closure.params,
            &closure.return_type,
            &closure.body,
            closure.span.offset,
            comments,
        ),
        Expr::Error(..) => emitter.write("<error>"),
    }
}

pub(crate) fn emit_designator(
    emitter: &mut Emitter,
    designator: &Designator,
    comments: &CommentMap,
) {
    for (index, part) in designator.parts.iter().enumerate() {
        match part {
            DesignatorPart::Ident(name, ..) => {
                if index > 0 {
                    match &designator.parts[index - 1] {
                        DesignatorPart::Ident(..) => emitter.write("."),
                        DesignatorPart::Index(..) => emitter.write("."),
                    }
                }
                emitter.write(name);
            }
            DesignatorPart::Index(index_expr, ..) => {
                emitter.write("[");
                emit_expr(emitter, index_expr, 0, comments);
                emitter.write("]");
            }
        }
    }
}

pub(crate) fn emit_arg_list(emitter: &mut Emitter, args: &[Expr], comments: &CommentMap) {
    for (index, arg) in args.iter().enumerate() {
        if index > 0 {
            emitter.write(", ");
        }
        emit_expr(emitter, arg, 0, comments);
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
        assert_eq!(
            expr_from_body("program T; begin var X: integer := Scene[0].resolved.rect.x; end."),
            "Scene[0].resolved.rect.x"
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
            "record\n  X := 1;\n  Y := 2;\nend"
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
    fn nonempty_record_literal_is_multiline() {
        let formatted = expr_from_body(
            "program T; type Point = record X: integer; end; begin var Value: Point := record X := 1; end; end.",
        );
        assert_eq!(formatted, "record\n  X := 1;\nend");
    }

    #[test]
    fn empty_record_literal_has_one_space() {
        let formatted = expr_from_body(
            "program T; type Empty = record end; begin var Value: Empty := record  end; end.",
        );
        assert_eq!(formatted, "record end");
    }

    #[test]
    fn empty_record_update_has_one_space() {
        let formatted = expr_from_body(
            "program T; type Empty = record end; begin var Value: Empty := Base with  end; end.",
        );
        assert_eq!(formatted, "Base with end");
    }

    #[test]
    fn nested_record_literal_indents_from_its_field() {
        let formatted = expr_from_body(
            "program T; type Inner = record X: integer; end; Outer = record Item: Inner; end; begin var Value: Outer := record Item := record X := 1; end; end; end.",
        );
        assert_eq!(
            formatted,
            "record\n  Item := record\n    X := 1;\n  end;\nend"
        );
    }

    #[test]
    fn long_binary_chain_wraps() {
        let formatted = expr_from_body(
            "program T; begin var X: boolean := VeryLongIdentifierAlpha + VeryLongIdentifierBeta + VeryLongIdentifierGamma + VeryLongIdentifierDelta + VeryLongIdentifierEpsilon; end.",
        );
        assert!(formatted.contains(" +\n"), "formatted: {formatted}");
    }
}
