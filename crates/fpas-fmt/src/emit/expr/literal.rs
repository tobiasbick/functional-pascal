//! Literals, aggregates, and formatting helpers.
//!
//! **Documentation:** `docs/pascal/tools/fmt-style.md`.

use fpas_parser::{Expr, FieldInit};

use crate::comments::CommentMap;

use super::super::Emitter;
/// Emits array items using source line groups and structural indentation.
pub(super) fn emit_array_literal(
    emitter: &mut Emitter,
    elements: &[Expr],
    span: fpas_lexer::Span,
    comments: &CommentMap,
) {
    super::lists::emit_list(
        emitter,
        elements,
        super::lists::ListStyle {
            span,
            open: "[",
            close: "]",
            compact_collection: false,
        },
        Expr::span,
        elements
            .iter()
            .any(|element| super::lists::expression_multiline(element, comments)),
        comments,
        |inner, element| super::emit_expr(inner, element, 0, comments),
    );
}
pub(super) fn emit_record_field_inits(
    emitter: &mut Emitter,
    fields: &[FieldInit],
    comments: &CommentMap,
) {
    for (index, field) in fields.iter().enumerate() {
        if index > 0 {
            emitter.write("; ");
        }
        emitter.write(&field.name);
        emitter.write(" := ");
        super::emit_expr(emitter, &field.value, 0, comments);
    }
}

pub(super) fn format_string(value: &str) -> String {
    let mut out = String::new();
    let mut quoted = false;
    for ch in value.chars() {
        if ch.is_control() {
            if quoted {
                out.push('\'');
                quoted = false;
            }
            out.push('#');
            out.push_str(&(ch as u32).to_string());
        } else if ch == '\'' {
            if !quoted {
                out.push('\'');
                quoted = true;
            }
            out.push_str("''");
        } else {
            if !quoted {
                out.push('\'');
                quoted = true;
            }
            out.push(ch);
        }
    }
    if quoted {
        out.push('\'');
    } else if out.is_empty() {
        out.push_str("''");
    }
    out
}

pub(super) fn format_real(value: f64) -> String {
    if value.is_finite() && value.fract() == 0.0 && value.abs() < 1e15 {
        return format!("{value:.1}");
    }
    let text = format!("{value}");
    if text.contains('.') || text.contains('e') || text.contains('E') {
        return text;
    }
    format!("{text}.0")
}

/// Omits spacing before literals and the explicit or required parentheses of an operand.
pub(super) fn needs_space_after_negate(operand: &Expr) -> bool {
    !matches!(
        operand,
        Expr::Integer(..)
            | Expr::Real(..)
            | Expr::Paren(..)
            | Expr::BinaryOp { .. }
            | Expr::UnaryOp {
                op: fpas_parser::UnaryOp::Not,
                ..
            }
    )
}
