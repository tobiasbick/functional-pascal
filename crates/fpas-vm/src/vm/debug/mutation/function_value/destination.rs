//! Same-frame register destination policy for task-bound functions.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use super::super::super::inspection::{MutationRoot, MutationTarget};
use super::super::super::types::DebugSessionError;
use super::type_error;

/// Reject task-bound assignment unless the destination is a same-frame register root.
pub(super) fn require_frame_register(
    destination: Option<&MutationTarget>,
    request_frame: Option<u64>,
    generation: u32,
) -> Result<(), DebugSessionError> {
    let Some(destination) = destination else {
        return Err(type_error(
            "task-bound function assignment requires a selected destination",
            "Assign the function onto a mutable local or parameter in the same live owner frame.",
        ));
    };
    if destination.generation != generation {
        return Err(type_error(
            "task-bound function assignment destination belongs to an expired stop snapshot",
            "Request variables again for the current stop, then assign in the selected owner frame.",
        ));
    }
    if !destination.path.is_empty() {
        return Err(type_error(
            "task-bound function assignment rejects descendant destinations",
            "Assign onto the complete mutable function binding, not a field, index, or payload child.",
        ));
    }
    match destination.root {
        MutationRoot::FrameRegister(_) => {}
        MutationRoot::Global(_) => {
            return Err(type_error(
                "task-bound function assignment rejects a global destination",
                "Assign onto a mutable local or parameter in the selected owner frame. Globals can escape the owning task.",
            ));
        }
        MutationRoot::ClosureCell(_) => {
            return Err(type_error(
                "task-bound function assignment rejects a capture-cell destination",
                "Assign onto a mutable local or parameter register, not a captured mutable cell.",
            ));
        }
    }
    if destination.frame_id != request_frame {
        return Err(type_error(
            "task-bound function assignment requires the destination to share the selected owner frame",
            "Assign onto a mutable local or parameter in the function's owner frame. The debugger does not write other frames or tasks.",
        ));
    }
    Ok(())
}
