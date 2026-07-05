//! Pascal-facing message box option flags for `Application.MessageBox`.
//!
//! Values match `turbo_vision::helpers::msgbox` at turbo-vision 2.0.0.
//!
//! **Documentation:** `docs/pascal/std/tui/app/message-box.md`

/// Warning-style message box (`MessageBoxOption.Warning`).
pub const MESSAGE_BOX_OPTION_WARNING: i64 = 0;
/// Error-style message box (`MessageBoxOption.Error`).
pub const MESSAGE_BOX_OPTION_ERROR: i64 = 1;
/// Information-style message box (`MessageBoxOption.Information`).
pub const MESSAGE_BOX_OPTION_INFORMATION: i64 = 2;
/// Confirmation-style message box (`MessageBoxOption.Confirmation`).
pub const MESSAGE_BOX_OPTION_CONFIRMATION: i64 = 3;
/// About-style message box (`MessageBoxOption.About`).
pub const MESSAGE_BOX_OPTION_ABOUT: i64 = 4;
/// Show a Yes button (`MessageBoxOption.YesButton`).
pub const MESSAGE_BOX_OPTION_YES_BUTTON: i64 = 0x0100;
/// Show a No button (`MessageBoxOption.NoButton`).
pub const MESSAGE_BOX_OPTION_NO_BUTTON: i64 = 0x0200;
/// Show an OK button (`MessageBoxOption.OkButton`).
pub const MESSAGE_BOX_OPTION_OK_BUTTON: i64 = 0x0400;
/// Show a Cancel button (`MessageBoxOption.CancelButton`).
pub const MESSAGE_BOX_OPTION_CANCEL_BUTTON: i64 = 0x0800;
/// Yes, No, and Cancel buttons (`MessageBoxOption.YesNoCancel`).
pub const MESSAGE_BOX_OPTION_YES_NO_CANCEL: i64 =
    MESSAGE_BOX_OPTION_YES_BUTTON | MESSAGE_BOX_OPTION_NO_BUTTON | MESSAGE_BOX_OPTION_CANCEL_BUTTON;
/// OK and Cancel buttons (`MessageBoxOption.OkCancel`).
pub const MESSAGE_BOX_OPTION_OK_CANCEL: i64 =
    MESSAGE_BOX_OPTION_OK_BUTTON | MESSAGE_BOX_OPTION_CANCEL_BUTTON;
