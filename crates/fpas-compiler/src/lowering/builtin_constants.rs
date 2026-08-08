//! Runtime values for standard-library constants recognized during lowering.

use fpas_bytecode::Value;
use fpas_std::std_symbols as symbols;

pub(super) fn value(name: &str) -> Option<Value> {
    let short = name.rsplit('.').next().unwrap_or(name);
    match name {
        symbols::STD_MATH_PI => Some(Value::Real(std::f64::consts::PI)),
        symbols::STD_CONSOLE_BLACK => Some(Value::Integer(0)),
        symbols::STD_CONSOLE_BLUE => Some(Value::Integer(1)),
        symbols::STD_CONSOLE_GREEN => Some(Value::Integer(2)),
        symbols::STD_CONSOLE_CYAN => Some(Value::Integer(3)),
        symbols::STD_CONSOLE_RED => Some(Value::Integer(4)),
        symbols::STD_CONSOLE_MAGENTA => Some(Value::Integer(5)),
        symbols::STD_CONSOLE_BROWN => Some(Value::Integer(6)),
        symbols::STD_CONSOLE_LIGHT_GRAY => Some(Value::Integer(7)),
        symbols::STD_CONSOLE_DARK_GRAY => Some(Value::Integer(8)),
        symbols::STD_CONSOLE_LIGHT_BLUE => Some(Value::Integer(9)),
        symbols::STD_CONSOLE_LIGHT_GREEN => Some(Value::Integer(10)),
        symbols::STD_CONSOLE_LIGHT_CYAN => Some(Value::Integer(11)),
        symbols::STD_CONSOLE_LIGHT_RED => Some(Value::Integer(12)),
        symbols::STD_CONSOLE_LIGHT_MAGENTA => Some(Value::Integer(13)),
        symbols::STD_CONSOLE_YELLOW => Some(Value::Integer(14)),
        symbols::STD_CONSOLE_WHITE => Some(Value::Integer(15)),
        symbols::STD_CONSOLE_BLINK => Some(Value::Integer(128)),
        symbols::STD_CONSOLE_BW40 => Some(Value::Integer(0)),
        symbols::STD_CONSOLE_C40 => Some(Value::Integer(1)),
        symbols::STD_CONSOLE_BW80 => Some(Value::Integer(2)),
        symbols::STD_CONSOLE_C80 => Some(Value::Integer(3)),
        symbols::STD_CONSOLE_CO40 => Some(Value::Integer(4)),
        symbols::STD_CONSOLE_CO80 => Some(Value::Integer(5)),
        symbols::STD_CONSOLE_MONO => Some(Value::Integer(7)),
        symbols::STD_CONSOLE_FONT_8X8 => Some(Value::Integer(256)),
        _ => short_value(short),
    }
}

fn short_value(name: &str) -> Option<Value> {
    let value = match name {
        "Pi" => return Some(Value::Real(std::f64::consts::PI)),
        "Black" | "BW40" => 0,
        "Blue" | "C40" => 1,
        "Green" | "BW80" => 2,
        "Cyan" | "C80" => 3,
        "Red" | "CO40" => 4,
        "Magenta" | "CO80" => 5,
        "Brown" => 6,
        "LightGray" | "Mono" => 7,
        "DarkGray" => 8,
        "LightBlue" => 9,
        "LightGreen" => 10,
        "LightCyan" => 11,
        "LightRed" => 12,
        "LightMagenta" => 13,
        "Yellow" => 14,
        "White" => 15,
        "Blink" => 128,
        "Font8x8" => 256,
        _ => return None,
    };
    Some(Value::Integer(value))
}
