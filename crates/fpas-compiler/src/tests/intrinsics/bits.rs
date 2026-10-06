//! Integer bit-function results and runtime shift diagnostics for `Std.Bits`.

use super::super::{assert_succeeds, run_program};
use fpas_diagnostics::codes::RUNTIME_NUMERIC_DOMAIN_ERROR;

fn integer_literal(value: i64) -> String {
    if value == i64::MIN {
        "(-9223372036854775807 - 1)".to_string()
    } else {
        value.to_string()
    }
}

#[test]
fn bits_intrinsics_preserve_results_after_operator_removal() {
    let mut source = String::from("program BitsCalls; uses Std.Bits; begin\n");
    for value in [0_i64, 1, -1, -8, i64::MIN, i64::MAX] {
        let literal = integer_literal(value);
        for (function, expected) in [
            ("BitAnd", value & 7),
            ("BitOr", value | 7),
            ("BitXor", value ^ 7),
        ] {
            source.push_str(&format!(
                "if {function}({literal}, 7) <> {} then panic('{function}'); end if;\n",
                integer_literal(expected)
            ));
        }
        source.push_str(&format!(
            "if BitNot({literal}) <> {} then panic('not'); end if;\n",
            integer_literal(!value)
        ));
        for count in 0..64 {
            for (function, expected) in [
                ("ShiftLeft", value.wrapping_shl(count)),
                ("ShiftRight", value >> count),
            ] {
                source.push_str(&format!(
                    "if {function}({literal}, {count}) <> {} then panic('{function}'); end if;\n",
                    integer_literal(expected)
                ));
            }
        }
    }
    source.push_str("end.");
    assert_succeeds(&source);
}

#[test]
fn invalid_shift_counts_keep_numeric_domain_diagnostics() {
    for function in ["ShiftLeft", "ShiftRight"] {
        for count in [-1, 64, i64::MAX] {
            let error = run_program(&format!(
                "program Bad; uses Std.Bits; begin var A: integer := {function}(1, {count}); end."
            ))
            .unwrap_err();
            assert_eq!(error.code, RUNTIME_NUMERIC_DOMAIN_ERROR);
            assert!(error.message.contains(&count.to_string()), "{error:?}");
            assert!(error.message.contains("0..63"), "{error:?}");
            assert!(
                error
                    .help
                    .as_deref()
                    .expect("range hint")
                    .contains("0 and 63")
            );
        }
    }
}
