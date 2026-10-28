//! IR constants for standard-library names recognized during lowering.

use fpas_builtins::std_symbols as symbols;
use fpas_ir::Constant;

pub(super) fn value(name: &str) -> Option<Constant> {
    let short = name.rsplit('.').next().unwrap_or(name);
    match name {
        symbols::STD_MATH_PI => Some(Constant::Real(std::f64::consts::PI)),
        symbols::STD_CONSOLE_BLACK => Some(Constant::Integer(0)),
        symbols::STD_CONSOLE_BLUE => Some(Constant::Integer(1)),
        symbols::STD_CONSOLE_GREEN => Some(Constant::Integer(2)),
        symbols::STD_CONSOLE_CYAN => Some(Constant::Integer(3)),
        symbols::STD_CONSOLE_RED => Some(Constant::Integer(4)),
        symbols::STD_CONSOLE_MAGENTA => Some(Constant::Integer(5)),
        symbols::STD_CONSOLE_BROWN => Some(Constant::Integer(6)),
        symbols::STD_CONSOLE_LIGHT_GRAY => Some(Constant::Integer(7)),
        symbols::STD_CONSOLE_DARK_GRAY => Some(Constant::Integer(8)),
        symbols::STD_CONSOLE_LIGHT_BLUE => Some(Constant::Integer(9)),
        symbols::STD_CONSOLE_LIGHT_GREEN => Some(Constant::Integer(10)),
        symbols::STD_CONSOLE_LIGHT_CYAN => Some(Constant::Integer(11)),
        symbols::STD_CONSOLE_LIGHT_RED => Some(Constant::Integer(12)),
        symbols::STD_CONSOLE_LIGHT_MAGENTA => Some(Constant::Integer(13)),
        symbols::STD_CONSOLE_YELLOW => Some(Constant::Integer(14)),
        symbols::STD_CONSOLE_WHITE => Some(Constant::Integer(15)),
        symbols::STD_CONSOLE_BLINK => Some(Constant::Integer(128)),
        symbols::STD_CONSOLE_BW40 => Some(Constant::Integer(0)),
        symbols::STD_CONSOLE_C40 => Some(Constant::Integer(1)),
        symbols::STD_CONSOLE_BW80 => Some(Constant::Integer(2)),
        symbols::STD_CONSOLE_C80 => Some(Constant::Integer(3)),
        symbols::STD_CONSOLE_CO40 => Some(Constant::Integer(4)),
        symbols::STD_CONSOLE_CO80 => Some(Constant::Integer(5)),
        symbols::STD_CONSOLE_MONO => Some(Constant::Integer(7)),
        symbols::STD_CONSOLE_FONT_8X8 => Some(Constant::Integer(256)),
        _ => short_value(short),
    }
}

fn short_value(name: &str) -> Option<Constant> {
    let value = match name {
        "Pi" => return Some(Constant::Real(std::f64::consts::PI)),
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
    Some(Constant::Integer(value))
}
