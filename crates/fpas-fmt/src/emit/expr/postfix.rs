//! Emission for [`Expr::Postfix`] chains.
//!
//! Suffixes stay attached to their base; argument lists retain their source layout.
//!
//! **Documentation:** `docs/pascal/tools/fmt-style.md`

use fpas_parser::{Expr, PostfixOperation};

use crate::comments::CommentMap;

use super::super::width;
use super::{Emitter, emit_arguments, emit_expr};

/// Emit `base` followed by each `.Field`, `[index]`, or `.Method(args)` suffix.
pub(super) fn emit_postfix(
    emitter: &mut Emitter,
    base: &Expr,
    operations: &[PostfixOperation],
    comments: &CommentMap,
) {
    let widths: Vec<_> = if emitter.measuring {
        Vec::new()
    } else {
        operations
            .iter()
            .map(|op| {
                width::measure(|flat| emit_postfix_operation(flat, op, &CommentMap::default()))
                    .unwrap_or(0)
            })
            .collect()
    };
    let enclosing_suffix = emitter.suffix_width;
    let mut remaining = widths.iter().sum::<usize>();
    emitter.with_suffix_width(enclosing_suffix + remaining, |inner| {
        emit_expr(inner, base, 0, comments)
    });
    for (index, op) in operations.iter().enumerate() {
        remaining = remaining.saturating_sub(widths.get(index).copied().unwrap_or(0));
        emitter.with_suffix_width(enclosing_suffix + remaining, |inner| {
            emit_postfix_operation(inner, op, comments)
        });
    }
}

fn emit_postfix_operation(emitter: &mut Emitter, op: &PostfixOperation, comments: &CommentMap) {
    match op {
        PostfixOperation::Field { name, .. } => {
            emitter.write(".");
            emitter.write(name);
        }
        PostfixOperation::Index { index, span } => {
            super::lists::emit_index(emitter, index, *span, comments)
        }
        PostfixOperation::MethodCall { name, args, span } => {
            emitter.write(".");
            emitter.write(name);
            emit_arguments(emitter, args, *span, comments);
        }
    }
}
