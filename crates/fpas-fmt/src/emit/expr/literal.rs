//! Literals, aggregates, and formatting helpers.

use fpas_parser::{Expr, FieldInit};

use crate::style::INDENT_WIDTH;

use super::super::Emitter;
use super::super::wrap::{exceeds_width, measure_emit, text_width};

pub(super) fn emit_record_fields(emitter: &mut Emitter, fields: &[FieldInit]) {
    if fields.is_empty() {
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
        if matches!(&field.value, Expr::RecordLiteral { .. }) {
            emitter.with_indent(|inner| {
                super::emit_expr_impl(inner, &field.value, 0, false);
            });
        } else {
            super::emit_expr_impl(emitter, &field.value, 0, false);
        }
        emitter.write(";");
    }
    emitter.write("\n");
    write_to_column(emitter, base_column);
    emitter.write("end");
}

pub(super) fn write_to_column(emitter: &mut Emitter, column: usize) {
    let pad = column.saturating_sub(emitter.column());
    emitter.write(&" ".repeat(pad));
}

pub(super) fn emit_array_literal(emitter: &mut Emitter, elements: &[Expr]) {
    let items: Vec<String> = elements
        .iter()
        .map(|element| measure_emit(|inner| super::emit_expr_impl(inner, element, 0, false)))
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

pub(super) fn write_block_at_column(emitter: &mut Emitter, column: usize, text: &str) {
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
pub(super) fn emit_record_field_inits(emitter: &mut Emitter, fields: &[FieldInit]) {
    for (index, field) in fields.iter().enumerate() {
        if index > 0 {
            emitter.write("; ");
        }
        emitter.write(&field.name);
        emitter.write(" := ");
        super::emit_expr(emitter, &field.value, 0);
    }
}

pub(super) fn record_literal_end(fields: &[FieldInit]) -> &'static str {
    if fields.is_empty() { "end" } else { "; end" }
}

pub(super) fn format_string(value: &str) -> String {
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

pub(super) fn needs_space_after_negate(operand: &Expr) -> bool {
    !matches!(
        operand,
        Expr::Integer(..) | Expr::Real(..) | Expr::Paren(..)
    )
}
