//! Emission for [`Expr::Postfix`] chains.
//!
//! Compact chains stay on one line. When the rendered chain exceeds the formatter width limit,
//! break before each suffix and indent continuations by two spaces from the expression base column.
//!
//! **Documentation:** `docs/pascal/tools/fmt-style.md`

use fpas_parser::{Expr, PostfixOperation};

use super::super::wrap::{exceeds_width, measure_emit, text_width};
use super::{Emitter, emit_arg_list, emit_expr, emit_expr_impl};

/// Emit `base` followed by each `.Field`, `[index]`, or `.Method(args)` suffix.
pub(super) fn emit_postfix(
    emitter: &mut Emitter,
    base: &Expr,
    operations: &[PostfixOperation],
    allow_wrap: bool,
) {
    let base_column = emitter.column();
    if allow_wrap {
        let rendered = measure_emit(|inner| emit_postfix_compact(inner, base, operations));
        if exceeds_width(base_column, text_width(&rendered)) {
            emit_postfix_wrapped(emitter, base, operations, base_column);
            return;
        }
    }
    emit_postfix_compact(emitter, base, operations);
}

fn emit_postfix_compact(emitter: &mut Emitter, base: &Expr, operations: &[PostfixOperation]) {
    emit_expr_impl(emitter, base, 0, false);
    for op in operations {
        emit_postfix_operation(emitter, op);
    }
}

fn emit_postfix_wrapped(
    emitter: &mut Emitter,
    base: &Expr,
    operations: &[PostfixOperation],
    base_column: usize,
) {
    emit_expr_impl(emitter, base, 0, false);
    let indent = " ".repeat(base_column.saturating_add(2));
    for op in operations {
        emitter.write("\n");
        emitter.write(&indent);
        emit_postfix_operation(emitter, op);
    }
}

fn emit_postfix_operation(emitter: &mut Emitter, op: &PostfixOperation) {
    match op {
        PostfixOperation::Field { name, .. } => {
            emitter.write(".");
            emitter.write(name);
        }
        PostfixOperation::Index { index, .. } => {
            emitter.write("[");
            emit_expr(emitter, index, 0);
            emitter.write("]");
        }
        PostfixOperation::MethodCall { name, args, .. } => {
            emitter.write(".");
            emitter.write(name);
            emitter.write("(");
            emit_arg_list(emitter, args);
            emitter.write(")");
        }
    }
}
