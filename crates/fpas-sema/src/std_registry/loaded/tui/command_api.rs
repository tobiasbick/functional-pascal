use super::super::super::define_const;
use crate::check::Checker;
use crate::types::Ty;
use fpas_std::std_symbols as s;

/// Register Pascal-facing `Std.Tui.Command.*` integer constants.
///
/// **Documentation:** `docs/pascal/std/tui/app/types.md`
pub(super) fn register_command_constants(checker: &mut Checker) {
    for name in [
        s::STD_TUI_COMMAND_ACCEPT,
        s::STD_TUI_COMMAND_CANCEL,
        s::STD_TUI_COMMAND_CLOSE,
        s::STD_TUI_COMMAND_QUIT,
    ] {
        define_const(checker, name, Ty::Integer);
    }
}
