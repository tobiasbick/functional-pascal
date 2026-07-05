use super::super::super::define_const;
use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;

/// Register Pascal-facing `Std.Tui.MessageBoxOption.*` integer constants.
///
/// **Documentation:** `docs/pascal/std/tui/app/message-box.md`
pub(super) fn register_message_box_option_constants(checker: &mut Checker) {
    for name in [
        s::STD_TUI_MESSAGE_BOX_OPTION_WARNING,
        s::STD_TUI_MESSAGE_BOX_OPTION_ERROR,
        s::STD_TUI_MESSAGE_BOX_OPTION_INFORMATION,
        s::STD_TUI_MESSAGE_BOX_OPTION_CONFIRMATION,
        s::STD_TUI_MESSAGE_BOX_OPTION_ABOUT,
        s::STD_TUI_MESSAGE_BOX_OPTION_YES_BUTTON,
        s::STD_TUI_MESSAGE_BOX_OPTION_NO_BUTTON,
        s::STD_TUI_MESSAGE_BOX_OPTION_OK_BUTTON,
        s::STD_TUI_MESSAGE_BOX_OPTION_CANCEL_BUTTON,
        s::STD_TUI_MESSAGE_BOX_OPTION_YES_NO_CANCEL,
        s::STD_TUI_MESSAGE_BOX_OPTION_OK_CANCEL,
    ] {
        define_const(checker, name, Ty::Integer);
    }
}
