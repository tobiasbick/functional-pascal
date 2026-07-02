//! FPAS ↔ Turbo Vision command-id translation.
//!
//! Upstream Turbo Vision consumes or broadcasts several `CM_*` command ids
//! before FPAS can observe them. User widgets that reuse those ids are offset
//! into a private band when passed to Turbo Vision and translated back before
//! `OnCommand` dispatch.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

/// Base band for FPAS-owned commands that collide with Turbo Vision built-ins.
pub(in crate::vm::execute::io::tui) const FPAS_TV_COMMAND_OFFSET: u16 = 0x8000;

/// Turbo Vision 1.3.1 `core::command::CM_*` ids reserved by upstream.
///
/// Keep this list aligned with the checked upstream `turbo-vision` version in
/// `Cargo.lock`. `0` is excluded
/// because FPAS uses it as a menu separator command and never dispatches it.
const TURBO_VISION_RESERVED_COMMANDS: &[u16] = &[
    10, 11, 12, 13, 14, 24, 25, 26, 27, 28, 29, 30, 31, 50, 51, 52, 53, 62, 63, 64, 65, 66, 100,
    101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 120,
    121, 130, 131, 132, 133, 140, 141, 150, 151, 152,
];

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
        for id in [1, 4, 15, 99, 119, 122, 1000] {
            assert_eq!(fpas_command_to_turbo_vision(id), id);
            assert_eq!(turbo_vision_command_to_fpas(id), id);
        }
    }

    #[test]
    fn all_upstream_reserved_ids_are_offset() {
        for id in TURBO_VISION_RESERVED_COMMANDS {
            assert_ne!(fpas_command_to_turbo_vision(*id), *id);
            assert_eq!(
                turbo_vision_command_to_fpas(fpas_command_to_turbo_vision(*id)),
                *id
            );
        }
    }

    #[test]
    fn close_and_application_demo_ids_are_reserved() {
        for id in [25, 100, 118, 152] {
            assert!(turbo_vision_reserved_command(id));
        }
    }
}
