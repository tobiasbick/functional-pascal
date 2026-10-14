//! Expressions and designators.

mod binary;
mod case_expression;
mod closure;
mod conditional;
mod lists;
mod literal;
mod patterns;
mod postfix;
mod precedence;
mod record_update;

use fpas_parser::{Designator, DesignatorPart, Expr, UnaryOp};

use crate::comments::CommentMap;

use super::Emitter;
use binary::emit_binary;
pub(crate) use lists::emit_arguments;
use literal::{emit_array_literal, format_real, format_string, needs_space_after_negate};
pub(crate) use patterns::emit_pattern;
use postfix::emit_postfix;
use precedence::{PREFIX_PREC, unary_prec};

/// Formats an expression.
#[must_use]
pub(crate) fn format_expr(expr: &Expr) -> String {
    let mut emitter = Emitter::new();
    emit_expr(&mut emitter, expr, 0, &CommentMap::default());
    emitter.finish()
}

/// Emits an expression, leaving any terminator to its statement or declaration owner.
pub(crate) fn emit_expr(emitter: &mut Emitter, expr: &Expr, min_prec: u8, comments: &CommentMap) {
    match expr {
        Expr::Integer(value, ..) => emitter.write(&value.to_string()),
        Expr::Real(value, ..) => emitter.write(&format_real(*value)),
        Expr::Str(value, ..) => emitter.write(&format_string(value)),
        Expr::Bool(value, ..) => emitter.write(if *value { "true" } else { "false" }),
        Expr::Designator(designator) => emit_designator(emitter, designator, comments),
        Expr::Call {
            designator,
            args,
            span,
        } => {
            emit_designator(emitter, designator, comments);
            emit_arguments(emitter, args, *span, comments);
        }
        Expr::UnaryOp { op, operand, .. } => {
            let prec = unary_prec(*op);
            if prec < min_prec {
                emitter.write("(");
                emit_expr(emitter, expr, 0, comments);
                emitter.write(")");
                return;
            }
            match op {
                UnaryOp::Not => {
                    emitter.write("not ");
                    emit_expr(emitter, operand, prec, comments);
                }
                UnaryOp::Negate => {
                    emitter.write("-");
                    if needs_space_after_negate(operand) {
                        emitter.write(" ");
                    }
                    emit_expr(emitter, operand, prec, comments);
                }
            }
        }
        Expr::BinaryOp {
            op, left, right, ..
        } => emit_binary(emitter, *op, left, right, min_prec, comments),
        Expr::Paren(inner, span) => emit_arguments(
            emitter,
            std::slice::from_ref(inner.as_ref()),
            *span,
            comments,
        ),
        Expr::ArrayLiteral(elements, span) => {
            emit_array_literal(emitter, elements, *span, comments)
        }
        Expr::DictLiteral(pairs, span) => {
            lists::emit_list(
                emitter,
                pairs,
                lists::ListStyle {
                    span: *span,
                    open: if pairs.is_empty() { "[:" } else { "[" },
                    close: "]",
                    compact_collection: false,
                },
                |(key, _)| key.span(),
                pairs.iter().any(|(key, value)| {
                    lists::expression_multiline(key, comments)
                        || lists::expression_multiline(value, comments)
                }),
                comments,
                |inner, (key, value)| {
                    emit_expr(inner, key, 0, comments);
                    inner.write(": ");
                    emit_expr(inner, value, 0, comments);
                },
            );
        }
        Expr::RecordUpdate { base, fields, span } => {
            record_update::emit_record_update(emitter, base, fields, span.offset, comments);
        }
        Expr::ResultOk(inner, span) => {
            emitter.write("Ok");
            emit_arguments(
                emitter,
                std::slice::from_ref(inner.as_ref()),
                *span,
                comments,
            );
        }
        Expr::ResultError(inner, span) => {
            emitter.write("Error");
            emit_arguments(
                emitter,
                std::slice::from_ref(inner.as_ref()),
                *span,
                comments,
            );
        }
        Expr::OptionSome(inner, span) => {
            emitter.write("Some");
            emit_arguments(
                emitter,
                std::slice::from_ref(inner.as_ref()),
                *span,
                comments,
            );
        }
        Expr::OptionNone(..) => emitter.write("None"),
        Expr::Try(inner, ..) => {
            emitter.write("try ");
            emit_expr(emitter, inner, PREFIX_PREC, comments);
        }
        Expr::Go(inner, ..) => {
            emitter.write("go ");
            emit_expr(emitter, inner, 0, comments);
        }
        Expr::Postfix {
            base, operations, ..
        } => emit_postfix(emitter, base, operations, comments),
        Expr::Closure(closure) => closure::emit_closure(
            emitter,
            closure.is_function,
            &closure.params,
            &closure.return_type,
            &closure.body,
            closure.span.offset,
            comments,
        ),
        Expr::VarArgument { designator, .. } => {
            emitter.write("var ");
            emit_designator(emitter, designator, comments);
        }
        Expr::NamedArgument { name, value, .. } => {
            emitter.write(name);
            emitter.write(" := ");
            emit_expr(emitter, value, 0, comments);
        }
        Expr::Is { value, pattern, .. } => {
            // `is` binds like a comparison.
            const IS_PREC: u8 = 3;
            if IS_PREC < min_prec {
                emitter.write("(");
                emit_expr(emitter, expr, 0, comments);
                emitter.write(")");
                return;
            }
            emit_expr(emitter, value, IS_PREC + 1, comments);
            emitter.write(" is ");
            emit_pattern(emitter, pattern, comments);
        }
        Expr::If {
            branches,
            else_value,
            else_span,
            span,
        } => conditional::emit_if_expression(
            emitter, branches, else_value, *else_span, *span, comments,
        ),
        Expr::Case {
            selector,
            arms,
            else_arm,
            span,
        } => case_expression::emit_case_expression(
            emitter,
            selector,
            arms,
            else_arm.as_deref(),
            *span,
            comments,
        ),
        Expr::Error(..) => emitter.write("<error>"),
    }
}

fn emit_expression_end(
    emitter: &mut Emitter,
    anchor: Option<usize>,
    ending: &str,
    comments: &CommentMap,
) {
    if let Some(anchor) = anchor {
        crate::comments::emit_leading_comments(emitter, comments, anchor, false);
    }
    emitter.write_current_indent();
    emitter.write(ending);
    if let Some(anchor) = anchor {
        crate::comments::emit_trailing_comments(emitter, comments, anchor);
        if emitter.ends_with_newline() {
            emitter.write_current_indent();
        }
    }
}

/// Emits a designator without changing its field or index evaluation order.
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
            DesignatorPart::Index(index_expr, span) => {
                lists::emit_index(emitter, index_expr, *span, comments)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::format_expr;
    use fpas_parser::{Stmt, parse};

    fn expr_from_body(source: &str) -> String {
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "{errors:?}");
        let Stmt::Const(binding) = &program.body[0] else {
            panic!("expected const stmt");
        };
        format_expr(&binding.value)
    }

    #[test]
    fn literals_and_designators() {
        assert_eq!(
            expr_from_body("program T; begin const X: integer := 42; end."),
            "42"
        );
        assert_eq!(
            expr_from_body("program T; begin const X: real := 3.14; end."),
            "3.14"
        );
        assert_eq!(
            expr_from_body("program T; begin const X: string := 'hi'; end."),
            "'hi'"
        );
        assert_eq!(
            expr_from_body("program T; begin const X: boolean := true; end."),
            "true"
        );
        assert_eq!(
            expr_from_body(
                "program T; begin const X: procedure(Msg: string) := Std.Console.WriteLn; end."
            ),
            "Std.Console.WriteLn"
        );
    }

    #[test]
    fn operators_and_calls() {
        assert_eq!(
            expr_from_body("program T; begin const X: integer := 1 + 2 * 3; end."),
            "1 + 2 * 3"
        );
        assert_eq!(
            expr_from_body("program T; begin const X: string := IntToStr(42); end."),
            "IntToStr(42)"
        );
        assert_eq!(
            expr_from_body("program T; begin const X: boolean := not true; end."),
            "not true"
        );
        assert_eq!(
            expr_from_body("program T; begin const X: integer := Scene[0].resolved.rect.x; end."),
            "Scene[0].resolved.rect.x"
        );
    }

    #[test]
    fn named_call_arguments_keep_names_and_written_order() {
        assert_eq!(
            expr_from_body(
                "program T; begin const X: integer := Sub(Right:=1,Left:=Max(A := 2, B := 3)); end."
            ),
            "Sub(Right := 1, Left := Max(A := 2, B := 3))"
        );
    }

    #[test]
    fn var_arguments_keep_their_marker() {
        assert_eq!(
            expr_from_body("program T; begin const X: integer := Next(var  Items[ 0 ]); end."),
            "Next(var Items[0])"
        );
    }

    #[test]
    fn aggregates_and_wrappers() {
        assert_eq!(
            expr_from_body("program T; begin const X: array of integer := [1, 2, 3]; end."),
            "[1, 2, 3]"
        );
        assert_eq!(
            expr_from_body("program T; begin const X: dict of string to integer := ['a': 1]; end."),
            "['a': 1]"
        );
        assert_eq!(
            expr_from_body(
                "program T; type Point = record X: integer; Y: integer; end record; begin const X: Point := Point( X := 1, Y := 2 ); end."
            ),
            "Point(X := 1, Y := 2)"
        );
        assert_eq!(
            expr_from_body("program T; begin const X: result of (integer, string) := Ok(42); end."),
            "Ok(42)"
        );
        assert_eq!(
            expr_from_body("program T; begin const X: option of integer := None; end."),
            "None"
        );
    }

    #[test]
    fn record_construction_formats_named_fields() {
        let formatted = expr_from_body(
            "program T; type Point = record X: integer; end record; begin const Value: Point := Point( X := 1 ); end.",
        );
        assert_eq!(formatted, "Point(X := 1)");
    }

    #[test]
    fn empty_record_construction_has_no_inner_spaces() {
        let formatted = expr_from_body(
            "program T; type Empty = record end record; begin const Value: Empty := Empty(  ); end.",
        );
        assert_eq!(formatted, "Empty()");
    }

    #[test]
    fn nonempty_record_update_formats_field_assignment() {
        let formatted = expr_from_body(
            "program T; type Point = record X: integer; end record; begin const Value: Point := Base with X := 1; end with; end.",
        );
        assert_eq!(formatted, "Base with X := 1; end with");
    }

    #[test]
    fn nested_record_construction_formats_named_fields() {
        let formatted = expr_from_body(
            "program T; type Inner = record X: integer; end record; type Outer = record Item: Inner; end record; begin const Value: Outer := Outer( Item := Inner( X := 1 ) ); end.",
        );
        assert_eq!(formatted, "Outer(Item := Inner(X := 1))");
    }

    #[test]
    fn record_construction_inside_array_formats_named_fields() {
        let formatted = expr_from_body(
            "program T; type Item = record Value: integer; end record; type Box = record Items: array of Item; end record; begin const Value: Box := Box( Items := [Item( Value := 10 )] ); end.",
        );
        assert_eq!(formatted, "Box(Items := [Item(Value := 10)])");
    }

    #[test]
    fn ast_only_long_binary_chain_wraps_structurally() {
        let formatted = expr_from_body(
            "program T; begin const X: boolean := VeryLongIdentifierAlpha + VeryLongIdentifierBeta + VeryLongIdentifierGamma + VeryLongIdentifierDelta + VeryLongIdentifierEpsilon; end.",
        );
        assert!(formatted.contains(" +\n  "), "formatted: {formatted}");
    }
}
