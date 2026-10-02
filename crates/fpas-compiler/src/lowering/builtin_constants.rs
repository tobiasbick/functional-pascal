//! Runtime values for standard-library constants recognized during lowering.

use fpas_bytecode::Value;
use fpas_std::std_symbols as symbols;

/// Returns a canonical intrinsic constant without introducing unqualified names.
pub(super) fn value(name: &str) -> Option<Value> {
    match name {
        value if value.eq_ignore_ascii_case(symbols::STD_MATH_PI) => {
            Some(Value::Real(std::f64::consts::PI))
        }
        value if value.eq_ignore_ascii_case(symbols::STD_CONSOLE_BLACK) => Some(Value::Integer(0)),
        value if value.eq_ignore_ascii_case(symbols::STD_CONSOLE_BLUE) => Some(Value::Integer(1)),
        value if value.eq_ignore_ascii_case(symbols::STD_CONSOLE_GREEN) => Some(Value::Integer(2)),
        value if value.eq_ignore_ascii_case(symbols::STD_CONSOLE_CYAN) => Some(Value::Integer(3)),
        value if value.eq_ignore_ascii_case(symbols::STD_CONSOLE_RED) => Some(Value::Integer(4)),
        value if value.eq_ignore_ascii_case(symbols::STD_CONSOLE_MAGENTA) => {
            Some(Value::Integer(5))
        }
        value if value.eq_ignore_ascii_case(symbols::STD_CONSOLE_BROWN) => Some(Value::Integer(6)),
        value if value.eq_ignore_ascii_case(symbols::STD_CONSOLE_LIGHT_GRAY) => {
            Some(Value::Integer(7))
        }
        value if value.eq_ignore_ascii_case(symbols::STD_CONSOLE_DARK_GRAY) => {
            Some(Value::Integer(8))
        }
        value if value.eq_ignore_ascii_case(symbols::STD_CONSOLE_LIGHT_BLUE) => {
            Some(Value::Integer(9))
        }
        value if value.eq_ignore_ascii_case(symbols::STD_CONSOLE_LIGHT_GREEN) => {
            Some(Value::Integer(10))
        }
        value if value.eq_ignore_ascii_case(symbols::STD_CONSOLE_LIGHT_CYAN) => {
            Some(Value::Integer(11))
        }
        value if value.eq_ignore_ascii_case(symbols::STD_CONSOLE_LIGHT_RED) => {
            Some(Value::Integer(12))
        }
        value if value.eq_ignore_ascii_case(symbols::STD_CONSOLE_LIGHT_MAGENTA) => {
            Some(Value::Integer(13))
        }
        value if value.eq_ignore_ascii_case(symbols::STD_CONSOLE_YELLOW) => {
            Some(Value::Integer(14))
        }
        value if value.eq_ignore_ascii_case(symbols::STD_CONSOLE_WHITE) => Some(Value::Integer(15)),
        value if value.eq_ignore_ascii_case(symbols::STD_CONSOLE_BLINK) => {
            Some(Value::Integer(128))
        }
        value if value.eq_ignore_ascii_case(symbols::STD_CONSOLE_BW40) => Some(Value::Integer(0)),
        value if value.eq_ignore_ascii_case(symbols::STD_CONSOLE_C40) => Some(Value::Integer(1)),
        value if value.eq_ignore_ascii_case(symbols::STD_CONSOLE_BW80) => Some(Value::Integer(2)),
        value if value.eq_ignore_ascii_case(symbols::STD_CONSOLE_C80) => Some(Value::Integer(3)),
        value if value.eq_ignore_ascii_case(symbols::STD_CONSOLE_CO40) => Some(Value::Integer(4)),
        value if value.eq_ignore_ascii_case(symbols::STD_CONSOLE_CO80) => Some(Value::Integer(5)),
        value if value.eq_ignore_ascii_case(symbols::STD_CONSOLE_MONO) => Some(Value::Integer(7)),
        value if value.eq_ignore_ascii_case(symbols::STD_CONSOLE_FONT_8X8) => {
            Some(Value::Integer(256))
        }
        _ => None,
    }
}
