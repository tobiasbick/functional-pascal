//! FPAS ↔ Turbo Vision command-id translation.
//!
//! Upstream Turbo Vision consumes several low command ids (`CM_QUIT = 24`,
//! `CM_TILE = 29`, …) inside `Application::handle_event` before FPAS can observe
//! them. User widgets that reuse those ids are offset into a private band when
//! passed to Turbo Vision and translated back before `OnCommand` dispatch.
//!
//! **Documentation:** `docs/future/turbo-vision-4-rust/07-post-migration-improvements.md` (Phase E)

/// Base band for FPAS-owned commands that collide with Turbo Vision built-ins.
pub(in crate::vm::execute::io::tui) const FPAS_TV_COMMAND_OFFSET: u16 = 0x8000;

/// Turbo Vision built-in command ids handled inside `Application::handle_event`
/// before unhandled commands reach FPAS.
const TURBO_VISION_RESERVED_COMMANDS: &[u16] = &[24, 29, 30, 31];

/// Returns `true` when `command_id` collides with an upstream built-in command.
pub(in crate::vm::execute::io::tui) fn turbo_vision_reserved_command(command_id: u16) -> bool {
    TURBO_VISION_RESERVED_COMMANDS.contains(&command_id)
}

/// Map a FPAS-facing command id to the value stored on a Turbo Vision widget.
pub(in crate::vm::execute::io::tui) fn fpas_command_to_turbo_vision(command_id: u16) -> u16 {
    if turbo_vision_reserved_command(command_id) {
        command_id + FPAS_TV_COMMAND_OFFSET
    } else {
        command_id
    }
}

/// Map a Turbo Vision event command back to the FPAS-facing id for `OnCommand`.
pub(in crate::vm::execute::io::tui) fn turbo_vision_command_to_fpas(command_id: u16) -> u16 {
    if command_id >= FPAS_TV_COMMAND_OFFSET {
        let candidate = command_id - FPAS_TV_COMMAND_OFFSET;
        if turbo_vision_reserved_command(candidate) {
            return candidate;
        }
    }
    command_id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserved_user_id_24_round_trips_through_offset_band() {
        let tv_id = fpas_command_to_turbo_vision(24);
        assert_ne!(tv_id, 24);
        assert_eq!(turbo_vision_command_to_fpas(tv_id), 24);
    }

    #[test]
    fn non_reserved_ids_pass_through_unchanged() {
        for id in [1, 4, 100, 1000] {
            assert_eq!(fpas_command_to_turbo_vision(id), id);
            assert_eq!(turbo_vision_command_to_fpas(id), id);
        }
    }

    #[test]
    fn all_documented_reserved_ids_are_offset() {
        for id in TURBO_VISION_RESERVED_COMMANDS {
            assert_ne!(fpas_command_to_turbo_vision(*id), *id);
            assert_eq!(
                turbo_vision_command_to_fpas(fpas_command_to_turbo_vision(*id)),
                *id
            );
        }
    }
}
