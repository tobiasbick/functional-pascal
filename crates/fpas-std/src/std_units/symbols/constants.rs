//! Standard-library constant values shared by semantic checking and lowering.
//!
//! **Documentation:** `docs/pascal/std/numeric/math.md` and
//! `docs/pascal/std/console/colors.md`.

use super::std_symbols as symbols;
use fpas_bytecode::Value;

/// Returns an intrinsic constant by its case-insensitive fully qualified name.
///
/// Import aliases must be resolved before lookup. Unqualified names and ordinary
/// routines have no constant value.
pub fn intrinsic_std_constant_value(name: &str) -> Option<Value> {
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

#[cfg(test)]
mod tests {
    use super::intrinsic_std_constant_value;
    use fpas_bytecode::Value;

    #[test]
    fn builtin_constant_names_are_qualified_and_case_insensitive() {
        let Value::Real(pi) = intrinsic_std_constant_value("sTd.mAtH.pI").unwrap() else {
            panic!("Pi must be real");
        };
        assert_eq!(pi.to_bits(), std::f64::consts::PI.to_bits());
        assert_eq!(
            intrinsic_std_constant_value("Std.Console.Font8x8"),
            Some(Value::Integer(256))
        );
        for name in ["Pi", "Math.Pi", "Std.Math.Sqrt", "Std.Unknown.Pi"] {
            assert_eq!(intrinsic_std_constant_value(name), None, "{name}");
        }
    }
}
