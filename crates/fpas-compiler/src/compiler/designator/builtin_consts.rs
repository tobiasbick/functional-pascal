use fpas_bytecode::Value;
use fpas_std::std_symbols as s;
use fpas_std::{
    CM_ABOUT, CM_OPEN, CM_USER, COMMAND_CANCEL, COMMAND_CLOSE, COMMAND_OK, COMMAND_QUIT,
    MESSAGE_BOX_OPTION_ABOUT, MESSAGE_BOX_OPTION_CANCEL_BUTTON, MESSAGE_BOX_OPTION_CONFIRMATION,
    MESSAGE_BOX_OPTION_ERROR, MESSAGE_BOX_OPTION_INFORMATION, MESSAGE_BOX_OPTION_NO_BUTTON,
    MESSAGE_BOX_OPTION_OK_BUTTON, MESSAGE_BOX_OPTION_OK_CANCEL, MESSAGE_BOX_OPTION_WARNING,
    MESSAGE_BOX_OPTION_YES_BUTTON, MESSAGE_BOX_OPTION_YES_NO_CANCEL,
};

use super::Compiler;

impl Compiler {
    pub(super) fn builtin_const_value(name: &str) -> Option<Value> {
        match name {
            s::STD_MATH_PI => Some(Value::Real(std::f64::consts::PI)),
            s::STD_CONSOLE_BLACK => Some(Value::Integer(0)),
            s::STD_CONSOLE_BLUE => Some(Value::Integer(1)),
            s::STD_CONSOLE_GREEN => Some(Value::Integer(2)),
            s::STD_CONSOLE_CYAN => Some(Value::Integer(3)),
            s::STD_CONSOLE_RED => Some(Value::Integer(4)),
            s::STD_CONSOLE_MAGENTA => Some(Value::Integer(5)),
            s::STD_CONSOLE_BROWN => Some(Value::Integer(6)),
            s::STD_CONSOLE_LIGHT_GRAY => Some(Value::Integer(7)),
            s::STD_CONSOLE_DARK_GRAY => Some(Value::Integer(8)),
            s::STD_CONSOLE_LIGHT_BLUE => Some(Value::Integer(9)),
            s::STD_CONSOLE_LIGHT_GREEN => Some(Value::Integer(10)),
            s::STD_CONSOLE_LIGHT_CYAN => Some(Value::Integer(11)),
            s::STD_CONSOLE_LIGHT_RED => Some(Value::Integer(12)),
            s::STD_CONSOLE_LIGHT_MAGENTA => Some(Value::Integer(13)),
            s::STD_CONSOLE_YELLOW => Some(Value::Integer(14)),
            s::STD_CONSOLE_WHITE => Some(Value::Integer(15)),
            s::STD_CONSOLE_BLINK => Some(Value::Integer(128)),
            s::STD_CONSOLE_BW40 => Some(Value::Integer(0)),
            s::STD_CONSOLE_C40 => Some(Value::Integer(1)),
            s::STD_CONSOLE_BW80 => Some(Value::Integer(2)),
            s::STD_CONSOLE_C80 => Some(Value::Integer(3)),
            s::STD_CONSOLE_CO40 => Some(Value::Integer(4)),
            s::STD_CONSOLE_CO80 => Some(Value::Integer(5)),
            s::STD_CONSOLE_MONO => Some(Value::Integer(7)),
            s::STD_CONSOLE_FONT_8X8 => Some(Value::Integer(256)),
            s::STD_TUI_CM_OK => Some(Value::Integer(COMMAND_OK)),
            s::STD_TUI_CM_CANCEL => Some(Value::Integer(COMMAND_CANCEL)),
            s::STD_TUI_CM_CLOSE => Some(Value::Integer(COMMAND_CLOSE)),
            s::STD_TUI_CM_QUIT => Some(Value::Integer(COMMAND_QUIT)),
            s::STD_TUI_CM_ABOUT => Some(Value::Integer(CM_ABOUT)),
            s::STD_TUI_CM_OPEN => Some(Value::Integer(CM_OPEN)),
            s::STD_TUI_CM_USER => Some(Value::Integer(CM_USER)),
            s::STD_TUI_MESSAGE_BOX_OPTION_WARNING => {
                Some(Value::Integer(MESSAGE_BOX_OPTION_WARNING))
            }
            s::STD_TUI_MESSAGE_BOX_OPTION_ERROR => Some(Value::Integer(MESSAGE_BOX_OPTION_ERROR)),
            s::STD_TUI_MESSAGE_BOX_OPTION_INFORMATION => {
                Some(Value::Integer(MESSAGE_BOX_OPTION_INFORMATION))
            }
            s::STD_TUI_MESSAGE_BOX_OPTION_CONFIRMATION => {
                Some(Value::Integer(MESSAGE_BOX_OPTION_CONFIRMATION))
            }
            s::STD_TUI_MESSAGE_BOX_OPTION_ABOUT => Some(Value::Integer(MESSAGE_BOX_OPTION_ABOUT)),
            s::STD_TUI_MESSAGE_BOX_OPTION_YES_BUTTON => {
                Some(Value::Integer(MESSAGE_BOX_OPTION_YES_BUTTON))
            }
            s::STD_TUI_MESSAGE_BOX_OPTION_NO_BUTTON => {
                Some(Value::Integer(MESSAGE_BOX_OPTION_NO_BUTTON))
            }
            s::STD_TUI_MESSAGE_BOX_OPTION_OK_BUTTON => {
                Some(Value::Integer(MESSAGE_BOX_OPTION_OK_BUTTON))
            }
            s::STD_TUI_MESSAGE_BOX_OPTION_CANCEL_BUTTON => {
                Some(Value::Integer(MESSAGE_BOX_OPTION_CANCEL_BUTTON))
            }
            s::STD_TUI_MESSAGE_BOX_OPTION_YES_NO_CANCEL => {
                Some(Value::Integer(MESSAGE_BOX_OPTION_YES_NO_CANCEL))
            }
            s::STD_TUI_MESSAGE_BOX_OPTION_OK_CANCEL => {
                Some(Value::Integer(MESSAGE_BOX_OPTION_OK_CANCEL))
            }
            _ => None,
        }
    }
}
