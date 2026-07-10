//! Upstream Turbo Vision reserved `CM_*` command ids.
//!
//! Used to document collisions when assigning widget `commandId` values on the try-2 path.
//! Try-2 passes upstream command ids to `OnCommand` unchanged (no offset band).
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

/// `turbo-vision` 2.0 `core::command::CM_*` ids reserved by upstream.
///
/// Keep this list aligned with the checked upstream `turbo-vision` version in
/// `Cargo.lock`. `0` is excluded because FPAS uses it as a menu separator
/// command and never dispatches it.
#[cfg(test)]
const TURBO_VISION_RESERVED_COMMANDS: &[u16] = &[
    1, 4, 5, 6, 7, 8, 10, 11, 12, 13, 14, 20, 21, 22, 23, 24, 25, 26, 31, 50, 51, 52, 55, 57, 60,
    61, 62, 63, 66, 67, 69, 70, 100, 101, 102, 103, 108, 109, 111, 115, 116, 117, 118, 120, 121,
    130, 131, 132, 133, 140, 141, 150, 151, 152, 300, 301, 302, 303, 304, 305,
];

#[cfg(test)]
/// Returns `true` when `command_id` collides with an upstream built-in command.
pub(in crate::vm::execute::io::tui) fn turbo_vision_reserved_command(command_id: u16) -> bool {
    TURBO_VISION_RESERVED_COMMANDS.contains(&command_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn about_and_file_focus_ids_are_reserved() {
        for id in [100, 102, 118, 152, 301] {
            assert!(turbo_vision_reserved_command(id));
        }
    }

    /// Fails when `turbo-vision` adds or renumbers `CM_*` ids — update
    /// `TURBO_VISION_RESERVED_COMMANDS` after a dependency bump.
    #[test]
    fn reserved_list_matches_upstream_cm_constants() {
        use turbo_vision::core::command::{
            CM_ABOUT, CM_BIRTHDATE, CM_CANCEL, CM_CASCADE, CM_CLEAR, CM_CLOSE, CM_CLOSE_FILE,
            CM_COMMAND_SET_CHANGED, CM_CONTROLS_DEMO, CM_COPY, CM_CUT, CM_DEFAULT,
            CM_FILE_DOUBLE_CLICKED, CM_FILE_FOCUSED, CM_FIND, CM_FIND_IN_FILES, CM_FOCUS_LINK,
            CM_GOTO_LINE, CM_GRAB_DEFAULT, CM_HELP_INDEX, CM_HISTORY_SELECTED, CM_KEYBOARD_REF,
            CM_LISTBOX_DEMO, CM_LISTBOX_SELECT, CM_MEMO_DEMO, CM_NEW, CM_NEXT, CM_NO, CM_OK,
            CM_OPEN, CM_PASTE, CM_PREV, CM_QUIT, CM_RADIO_SELECTED, CM_RECEIVED_FOCUS,
            CM_RECORD_HISTORY, CM_REDO, CM_REDRAW, CM_RELEASE_DEFAULT, CM_RELEASED_FOCUS,
            CM_REPLACE, CM_RESIZE, CM_SAVE, CM_SAVE_ALL, CM_SAVE_AS, CM_SCREENSHOT,
            CM_SCROLLBAR_CHANGED, CM_SEARCH_AGAIN, CM_SELECT_ALL, CM_SELECT_WINDOW_NUM,
            CM_SHOW_HISTORY, CM_TEXT_VIEWER, CM_TILE, CM_TOGGLE_SIDEBAR, CM_TOGGLE_STATUSBAR,
            CM_UNDO, CM_YES, CM_ZOOM, CM_ZOOM_IN, CM_ZOOM_OUT,
        };

        let mut upstream = vec![
            CM_QUIT,
            CM_CLOSE,
            CM_ZOOM,
            CM_RESIZE,
            CM_NEXT,
            CM_PREV,
            CM_OK,
            CM_CANCEL,
            CM_YES,
            CM_NO,
            CM_DEFAULT,
            CM_CUT,
            CM_COPY,
            CM_PASTE,
            CM_UNDO,
            CM_CLEAR,
            CM_TILE,
            CM_CASCADE,
            CM_SCREENSHOT,
            CM_RECEIVED_FOCUS,
            CM_RELEASED_FOCUS,
            CM_COMMAND_SET_CHANGED,
            CM_SELECT_WINDOW_NUM,
            CM_SCROLLBAR_CHANGED,
            CM_RECORD_HISTORY,
            CM_GRAB_DEFAULT,
            CM_RELEASE_DEFAULT,
            CM_REDRAW,
            CM_FOCUS_LINK,
            CM_RADIO_SELECTED,
            CM_SHOW_HISTORY,
            CM_HISTORY_SELECTED,
            CM_ABOUT,
            CM_BIRTHDATE,
            CM_FILE_FOCUSED,
            CM_FILE_DOUBLE_CLICKED,
            CM_TEXT_VIEWER,
            CM_CONTROLS_DEMO,
            CM_REDO,
            CM_SELECT_ALL,
            CM_FIND,
            CM_REPLACE,
            CM_SEARCH_AGAIN,
            CM_FIND_IN_FILES,
            CM_GOTO_LINE,
            CM_ZOOM_IN,
            CM_ZOOM_OUT,
            CM_TOGGLE_SIDEBAR,
            CM_TOGGLE_STATUSBAR,
            CM_HELP_INDEX,
            CM_KEYBOARD_REF,
            CM_LISTBOX_DEMO,
            CM_LISTBOX_SELECT,
            CM_MEMO_DEMO,
            CM_NEW,
            CM_OPEN,
            CM_SAVE,
            CM_SAVE_AS,
            CM_SAVE_ALL,
            CM_CLOSE_FILE,
        ];
        upstream.sort_unstable();
        let mut fpas = TURBO_VISION_RESERVED_COMMANDS.to_vec();
        fpas.sort_unstable();
        assert_eq!(
            fpas, upstream,
            "update TURBO_VISION_RESERVED_COMMANDS after turbo-vision bump (see docs/pascal/std/tui/app/vm-bridge.md)"
        );
    }
}
