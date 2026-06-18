//! Shared helpers for hosted `Application.Run` loops (TUI and Graph).
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md`, `docs/pascal/std/graph/app.md`

use fpas_bytecode::Value;

/// Builds an exit-reason enum value for hosted application run loops.
pub(in crate::vm::execute::io) fn hosted_exit_reason(type_name: &str, variant: &str) -> Value {
    Value::Enum {
        type_name: type_name.into(),
        variant: variant.into(),
        fields: vec![],
    }
}
