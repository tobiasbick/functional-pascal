//! Expressions and designators.

mod binary;
mod closure;
mod decisions;
mod literal;
mod postfix;

use fpas_parser::{Designator, DesignatorPart, Expr, UnaryOp};

use crate::comments::CommentMap;

use super::Emitter;
use super::wrap::{exceeds_width, measure_emit, text_width};
use binary::{binary_op_spaced, binary_prec, emit_binary_with_break};
use literal::{
    emit_array_literal, emit_record_field_inits, format_real, format_string,
    needs_space_after_negate,
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
        Expr::If(decision) => decisions::emit_if_expression(emitter, decision, comments),
        Expr::Case(decision) => decisions::emit_case_expression(emitter, decision, comments),
        Expr::Integer(value, ..) => emitter.write(&value.to_string()),
        Expr::Real(value, ..) => emitter.write(&format_real(*value)),
        Expr::Str(value, ..) => emitter.write(&format_string(value)),
        Expr::Bool(value, ..) => emitter.write(if *value { "true" } else { "false" }),
        Expr::Designator(designator) => emit_designator(emitter, designator, comments),
        Expr::VarArgument(designator, _) => {
            emitter.write("var ");
            emit_designator(emitter, designator, comments);
        }
        Expr::Call {
            designator, args, ..
        } => {
            emit_designator(emitter, designator, comments);
            emitter.write("(");
            emit_arg_list(emitter, args, comments);
            emitter.write(")");
        }
        Expr::RecordConstruction {
            type_name, fields, ..
        } => {
            super::types::emit_qualified_id(emitter, type_name);
            emitter.write("(");
            for (index, field) in fields.iter().enumerate() {
                if index > 0 {
                    emitter.write(", ");
                }
                emitter.write(&field.name);
                emitter.write(" := ");
                emit_expr(emitter, &field.value, 0, comments);
            }
            emitter.write(")");
        }
        Expr::UnaryOp { op, operand, .. } => {
            let prec = if *op == UnaryOp::Not { 2 } else { 6 };
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
            emit_expr_impl(
                emitter,
                left,
                binary::left_precedence(*op, left),
                false,
                comments,
            );
            emitter.write(binary_op_spaced(*op));
            emit_expr_impl(emitter, right, prec + 1, false, comments);
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
        Expr::RecordUpdate { base, fields, .. } => {
            emit_expr(emitter, base, 0, comments);
            emitter.write(" with ");
            emit_record_field_inits(emitter, fields, comments);
            emitter.write(if fields.is_empty() {
                "end with"
            } else {
                "; end with"
            });
        }
        Expr::ResultOk(inner, ..) => {
            emitter.write("Result.Ok(");
            emit_expr(emitter, inner, 0, comments);
            emitter.write(")");
        }
        Expr::ResultError(inner, ..) => {
            emitter.write("Result.Error(");
            emit_expr(emitter, inner, 0, comments);
            emitter.write(")");
        }
        Expr::OptionSome(inner, ..) => {
            emitter.write("Option.Some(");
            emit_expr(emitter, inner, 0, comments);
            emitter.write(")");
        }
        Expr::OptionNone(..) => emitter.write("Option.None"),
        Expr::Try(inner, ..) => {
            emitter.write("try ");
            emit_expr(emitter, inner, 6, comments);
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
            closure.pure,
            &closure.params,
            &closure.return_type,
            &closure.body,
            closure.span.offset,
            comments,
        ),
        Expr::InvalidRecord(..) | Expr::Error(..) => emitter.write("<error>"),
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
        let Stmt::Const(var) = &program.body[0] else {
            panic!("expected var stmt");
        };
        format_expr(&var.value)
    }

    #[test]
    fn literals_and_designators() {
        assert_eq!(
            expr_from_body(r#"program T; begin const X: integer := 42; end program;"#),
            "42"
        );
        assert_eq!(
            expr_from_body(r#"program T; begin const X: real := 3.14; end program;"#),
            "3.14"
        );
        assert_eq!(
            expr_from_body(r#"program T; begin const X: string := 'hi'; end program;"#),
            "'hi'"
        );
        assert_eq!(
            expr_from_body(r#"program T; begin const X: boolean := true; end program;"#),
            "true"
        );
        assert_eq!(
            expr_from_body(
                r#"program T; begin const X: procedure(Msg: string) := Std.Console.WriteLn; end program;"#
            ),
            "Std.Console.WriteLn"
        );
    }

    #[test]
    fn operators_and_calls() {
        assert_eq!(
            expr_from_body(r#"program T; begin const X: integer := 1 + 2 * 3; end program;"#),
            "1 + 2 * 3"
        );
        assert_eq!(
            expr_from_body(r#"program T; begin const X: string := IntToStr(42); end program;"#),
            "IntToStr(42)"
        );
        assert_eq!(
            expr_from_body(r#"program T; begin const X: boolean := not true; end program;"#),
            "not true"
        );
        assert_eq!(
            expr_from_body(
                r#"program T; begin const X: integer := Scene[0].resolved.rect.x; end program;"#
            ),
            "Scene[0].resolved.rect.x"
        );
    }

    #[test]
    fn aggregates_and_wrappers() {
        assert_eq!(
            expr_from_body(
                r#"program T; begin const X: array of (integer) := [1, 2, 3]; end program;"#
            ),
            "[1, 2, 3]"
        );
        assert_eq!(
            expr_from_body(
                r#"program T; begin const X: dict of (string, integer) := ['a': 1]; end program;"#
            ),
            "['a': 1]"
        );
        assert_eq!(
            expr_from_body(
                r#"program T;

type Point = record
  X: integer;
  Y: integer;
end record;

begin
  const X: Point := Point(X := 1, Y := 2);
end program;
"#
            ),
            "Point(X := 1, Y := 2)"
        );
        assert_eq!(
            expr_from_body(
                r#"program T; begin const X: Result of (integer, string) := Result.Ok(42); end program;"#
            ),
            "Result.Ok(42)"
        );
        assert_eq!(
            expr_from_body(
                r#"program T; begin const X: Option of (integer) := Option.None; end program;"#
            ),
            "Option.None"
        );
    }

    #[test]
    fn named_record_construction_formats_supplied_fields() {
        let formatted = expr_from_body(
            r#"program T;

type Point = record
  X: integer;
end record;

begin
  const Value: Point := Point(X := 1);
end program;
"#,
        );
        assert_eq!(formatted, "Point(X := 1)");
    }

    #[test]
    fn empty_record_construction_has_an_empty_argument_list() {
        let formatted = expr_from_body(
            r#"program T;

type Empty = record
end record;

begin
  const Value: Empty := Empty();
end program;
"#,
        );
        assert_eq!(formatted, "Empty()");
    }

    #[test]
    fn nonempty_record_update_formats_field_assignment() {
        let formatted = expr_from_body(
            r#"program T;  type Point = record X: integer; end record; begin const Value: Point := Base with X := 1; end with; end program;"#,
        );
        assert_eq!(formatted, "Base with X := 1; end with");
    }

    #[test]
    fn nested_record_construction_keeps_each_target() {
        let formatted = expr_from_body(
            r#"program T;

type Inner = record
  X: integer;
end record;

type Outer = record
  Item: Inner;
end record;

begin
  const Value: Outer := Outer(Item := Inner(X := 1));
end program;
"#,
        );
        assert_eq!(formatted, "Outer(Item := Inner(X := 1))");
    }

    #[test]
    fn record_construction_inside_array_preserves_its_argument_list() {
        let formatted = expr_from_body(
            r#"program T;

type Item = record
  Value: integer;
end record;

type Box = record
  Items: array of (Item);
end record;

begin
  const Value: Box := Box(Items := [Item(Value := 10)]);
end program;
"#,
        );
        assert_eq!(formatted, "Box(Items := [Item(Value := 10)])");
    }

    #[test]
    fn long_binary_chain_wraps() {
        let formatted = expr_from_body(
            r#"program T; begin const X: boolean := VeryLongIdentifierAlpha + VeryLongIdentifierBeta + VeryLongIdentifierGamma + VeryLongIdentifierDelta + VeryLongIdentifierEpsilon; end program;"#,
        );
        assert!(formatted.contains(" +\n"), "formatted: {formatted}");
    }
}
