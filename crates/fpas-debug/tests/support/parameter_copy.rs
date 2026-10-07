//! Stop after a migrated parameter's writable local copy is initialized.

#![allow(
    clippy::panic,
    reason = "test fixtures fail fast with direct assertions for diagnostic clarity"
)]

use fpas_vm::{DebugRunResult, DebugSession};

/// Finds the current frame with a live, initialized local copy.
pub(crate) fn initialized_local_frame(session: &mut DebugSession, name: &str) -> u64 {
    for _ in 0..64 {
        let frame = session.stack(0, 1).expect("stack").items[0].id;
        for scope in session.scopes(frame).expect("scopes") {
            if scope.name == "Locals"
                && session
                    .variables(scope.variables_reference, 0, 32)
                    .expect("locals")
                    .items
                    .iter()
                    .any(|value| value.name == name && value.value != "<uninitialized>")
            {
                return frame;
            }
        }
        assert!(matches!(
            session.step_into().expect("step to local copy"),
            DebugRunResult::Stopped(_)
        ));
    }
    panic!("local copy {name} never became initialized");
}
