//! Pattern emission for case labels and `is` tests.
//!
//! **Documentation:** `docs/pascal/tools/fmt-style.md`.

use fpas_parser::{DestructureVariant, Pattern};

use crate::comments::CommentMap;

use super::super::Emitter;
use super::{emit_designator, emit_expr};

/// Emits a case or `is` pattern with canonical spacing.
pub(crate) fn emit_pattern(emitter: &mut Emitter, pattern: &Pattern, comments: &CommentMap) {
    match pattern {
        Pattern::Binding { name, .. } => {
            emitter.write("const ");
            emitter.write(name);
        }
        Pattern::Wildcard(_) => emitter.write("_"),
        Pattern::Value(expr) => emit_expr(emitter, expr, 0, comments),
        Pattern::Variant {
            constructor,
            fields,
            ..
        } => {
            emit_designator(emitter, constructor, comments);
            emitter.write("(");
            for (index, field) in fields.iter().enumerate() {
                if index > 0 {
                    emitter.write(", ");
                }
                if let Some((label, _)) = &field.label {
                    emitter.write(label);
                    emitter.write(" := ");
                }
                emit_pattern(emitter, &field.pattern, comments);
            }
            emitter.write(")");
        }
        Pattern::Destructure {
            variant, payload, ..
        } => {
            emitter.write(match variant {
                DestructureVariant::Ok => "Ok",
                DestructureVariant::Error => "Error",
                DestructureVariant::Some => "Some",
                DestructureVariant::None => "None",
            });
            if let Some(payload) = payload {
                emitter.write("(");
                emit_pattern(emitter, payload, comments);
                emitter.write(")");
            }
        }
    }
}
