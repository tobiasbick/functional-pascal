//! Public format semantics, precision boundaries and bounded expansion.

#![allow(
    clippy::expect_used,
    reason = "format contract tests inspect concrete results"
)]

use super::*;

fn formatted(template: &str, arguments: &[Value]) -> String {
    apply_format(template, arguments, SourceLocation::new(1, 1)).expect("valid format")
}

#[test]
fn bare_formats_keep_their_existing_behavior() {
    assert_eq!(
        formatted(
            "%d/%f/%s/%%",
            &[
                Value::Integer(42),
                Value::Integer(42),
                Value::Str("€%d".into())
            ]
        ),
        "42/42.0/€%d/%"
    );
    assert_eq!(formatted("%f", &[Value::Real(12.34)]), "12.34");
    assert_eq!(formatted("%f", &[Value::Real(-0.0)]), "-0.0");
    assert_eq!(
        formatted("%f", &[Value::Integer(9_007_199_254_740_993)]),
        "9007199254740992.0"
    );
}

#[test]
fn numeric_width_is_a_minimum_and_padding_respects_signs() {
    for (template, value, expected) in [
        ("%8d", -42, "     -42"),
        ("%08d", -42, "-0000042"),
        ("%08d", 42, "00000042"),
        ("%-08d", -42, "-42     "),
        ("%0-8d", -42, "-42     "),
        ("%2d", 12345, "12345"),
        ("%021d", i64::MIN, "-09223372036854775808"),
    ] {
        assert_eq!(
            formatted(template, &[Value::Integer(value)]),
            expected,
            "{template}"
        );
    }
    assert_eq!(formatted("%08.2f", &[Value::Real(-2.5)]), "-0002.50");
    assert_eq!(formatted("%2.2f", &[Value::Real(123.5)]), "123.50");
}

#[test]
fn fixed_precision_rounds_binary64_ties_to_even_and_keeps_negative_zero() {
    for (template, value, expected) in [
        ("%.0f", 2.5, "2"),
        ("%.0f", 3.5, "4"),
        ("%.1f", 1.25, "1.2"),
        ("%.1f", 1.75, "1.8"),
        ("%.2f", -0.004, "-0.00"),
        ("%.0f", -0.0, "-0"),
        ("%.4f", 1.5, "1.5000"),
    ] {
        assert_eq!(formatted(template, &[Value::Real(value)]), expected);
    }
}

#[test]
fn explicit_integer_precision_is_exact_over_the_full_integer_domain() {
    assert_eq!(
        formatted("%.2f", &[Value::Integer(9_007_199_254_740_993)]),
        "9007199254740993.00"
    );
    assert_eq!(
        formatted("%.0f", &[Value::Integer(i64::MIN)]),
        "-9223372036854775808"
    );
    assert_eq!(
        formatted("%.18e", &[Value::Integer(i64::MAX)]),
        "9.223372036854775807e+18"
    );
    assert_eq!(
        formatted("%.18E", &[Value::Integer(i64::MIN)]),
        "-9.223372036854775808E+18"
    );
    for (value, expected) in [(25, "2e+01"), (35, "4e+01"), (95, "1e+02"), (0, "0e+00")] {
        assert_eq!(formatted("%.0e", &[Value::Integer(value)]), expected);
    }
    assert_eq!(formatted("%.1e", &[Value::Integer(999)]), "1.0e+03");
}

#[test]
fn scientific_output_handles_extremes_exponent_signs_and_rounding_carry() {
    for (template, value, expected) in [
        ("%e", 1234.0, "1.234000e+03"),
        ("%.2E", 0.000125, "1.25E-04"),
        ("%.2e", 9.996, "1.00e+01"),
        ("%.0e", 2.5, "2e+00"),
        ("%.0e", 3.5, "4e+00"),
        ("%e", -0.0, "-0.000000e+00"),
        ("%e", f64::from_bits(1), "4.940656e-324"),
        ("%e", f64::MAX, "1.797693e+308"),
    ] {
        assert_eq!(
            formatted(template, &[Value::Real(value)]),
            expected,
            "{template}: {value}"
        );
    }
    assert_eq!(formatted("%E", &[Value::Integer(42)]), "4.200000E+01");
}

#[test]
fn malformed_or_unsupported_formats_report_the_existing_diagnostic() {
    for template in [
        "%",
        "%8",
        "%.",
        "%.f",
        "%--d",
        "%00d",
        "%0-0d",
        "%8s",
        "%.2s",
        "%.2d",
        "%8%",
        "%g",
        "%+d",
        "%#f",
        "%*d",
        "%.*f",
        "%1_000d",
        "%١f",
        "%.-1f",
        "%1000001d",
        "%.1025f",
        "%999999999999999999999999999999999999d",
    ] {
        let error = apply_format(template, &[Value::Integer(1)], SourceLocation::new(1, 1))
            .expect_err(template);
        assert_eq!(error.code, RUNTIME_FORMAT_MISMATCH, "{template}");
    }
    for (template, arguments) in [
        ("%e", vec![]),
        ("%d", vec![Value::Real(1.0)]),
        ("%e", vec![Value::Boolean(true)]),
        ("%s", vec![Value::Integer(1)]),
        ("%d", vec![Value::Integer(1), Value::Integer(2)]),
    ] {
        let error = apply_format(template, &arguments, SourceLocation::new(1, 1))
            .expect_err("bad arguments");
        assert_eq!(error.code, RUNTIME_FORMAT_MISMATCH);
    }
}

#[test]
fn nonfinite_numbers_are_not_presented_as_valid_numeric_text() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for template in ["%f", "%.2f", "%e", "%E"] {
            assert!(
                apply_format(template, &[Value::Real(value)], SourceLocation::new(1, 1)).is_err()
            );
        }
    }
}

#[test]
fn new_field_output_is_bounded_without_limiting_existing_strings() {
    let output = formatted("%1000000d", &[Value::Integer(1)]);
    assert_eq!(output.len(), 1_000_000);
    assert!(output.ends_with('1'));
    assert_eq!(formatted("%.1024f", &[Value::Integer(1)]).len(), 1026);
    let error = apply_format(
        "%500001d%500001d",
        &[Value::Integer(1), Value::Integer(2)],
        SourceLocation::new(1, 1),
    )
    .expect_err("aggregate budget");
    assert_eq!(error.code, RUNTIME_FORMAT_MISMATCH);
    let large = "€".repeat(400_000);
    let output = formatted(
        "%s%1d",
        &[Value::Str(large.clone().into()), Value::Integer(7)],
    );
    assert_eq!(output.len(), large.len() + 1);
    assert!(output.starts_with(&large));
}
