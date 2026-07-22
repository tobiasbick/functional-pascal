use fpas_bytecode::Value;
use fpas_std::std_symbols as s;

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
            _ => None,
        }
    }
}
