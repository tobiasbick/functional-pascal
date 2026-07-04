//! FPAS ↔ Turbo Vision command-id translation.
//!
//! Upstream Turbo Vision consumes or broadcasts several `CM_*` command ids
//! before FPAS can observe them. Application-defined widgets that reuse those
//! ids are offset into a private band when passed to Turbo Vision and translated
//! back before `OnCommand` dispatch. The four `Command.*` standard ids pass
//! through unchanged because they match Borland `CM_*` values in `turbo-vision`
//! 2.0.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use fpas_std::{COMMAND_CANCEL, COMMAND_CLOSE, COMMAND_OK, COMMAND_QUIT};

/// Base band for FPAS-owned commands that collide with Turbo Vision built-ins.
pub(in crate::vm::execute::io::tui) const FPAS_TV_COMMAND_OFFSET: u16 = 0x8000;

/// `turbo-vision` 2.0 `core::command::CM_*` ids reserved by upstream.
///
/// Keep this list aligned with the checked upstream `turbo-vision` version in
/// `Cargo.lock`. `0` is excluded because FPAS uses it as a menu separator
/// command and never dispatches it.
const TURBO_VISION_RESERVED_COMMANDS: &[u16] = &[
    1, 4, 5, 6, 7, 8, 10, 11, 12, 13, 14, 20, 21, 22, 23, 24, 25, 26, 31, 50, 51, 52, 55, 57, 60,
    61, 62, 63, 66, 67, 69, 70, 100, 101, 102, 103, 108, 109, 111, 115, 116, 117, 118, 120, 121,
    130, 131, 132, 133, 140, 141, 150, 151, 152, 300, 301, 302, 303, 304, 305,
];

/// Returns `true` when `command_id` collides with an upstream built-in command.
pub(in crate::vm::execute::io::tui) fn turbo_vision_reserved_command(command_id: u16) -> bool {
    TURBO_VISION_RESERVED_COMMANDS.contains(&command_id)
}

fn fpas_standard_command(command_id: u16) -> bool {
    matches!(
        i64::from(command_id),
        COMMAND_QUIT | COMMAND_CLOSE | COMMAND_OK | COMMAND_CANCEL
    )
}

/// Map a FPAS-facing command id to the value stored on a Turbo Vision widget.
pub(in crate::vm::execute::io::tui) fn fpas_command_to_turbo_vision(command_id: u16) -> u16 {
    if fpas_standard_command(command_id) {
        return command_id;
    }
    if turbo_vision_reserved_command(command_id) {
        command_id + FPAS_TV_COMMAND_OFFSET
    } else {
        command_id
    }
}

/// Map a Turbo Vision event command back to the FPAS-facing id for `OnCommand`.
pub(in crate::vm::execute::io::tui) fn turbo_vision_command_to_fpas(command_id: u16) -> u16 {
    if fpas_standard_command(command_id) {
        return command_id;
    }
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
    fn standard_commands_pass_through_unchanged() {
        for id in [1, 4, 10, 11] {
            assert_eq!(fpas_command_to_turbo_vision(id), id);
            assert_eq!(turbo_vision_command_to_fpas(id), id);
        }
    }

    #[test]
    fn non_reserved_ids_pass_through_unchanged() {
        for id in [15, 99, 119, 122, 1000] {
            assert_eq!(fpas_command_to_turbo_vision(id), id);
            assert_eq!(turbo_vision_command_to_fpas(id), id);
        }
    }

    #[test]
    fn all_upstream_reserved_ids_are_offset_except_standard_commands() {
        for id in TURBO_VISION_RESERVED_COMMANDS {
            if fpas_standard_command(*id) {
                continue;
            }
            assert_ne!(fpas_command_to_turbo_vision(*id), *id);
            assert_eq!(
                turbo_vision_command_to_fpas(fpas_command_to_turbo_vision(*id)),
                *id
            );
        }
    }

    #[test]
    fn about_and_file_focus_ids_are_reserved() {
        for id in [100, 102, 118, 152, 301] {
            assert!(turbo_vision_reserved_command(id));
        }
    }
}
