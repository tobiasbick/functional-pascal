use crate::text;

#[test]
fn source_bodies_and_signed_input_follow_the_same_grammar_and_bounds() {
    for (input, expected) in [
        ("0", 0),
        (" +1_024 ", 1024),
        ("-7", -7),
        ("$FF", 255),
        ("-$8000_0000_0000_0000", i64::MIN),
        ("9223372036854775807", i64::MAX),
        ("-9223372036854775808", i64::MIN),
    ] {
        assert_eq!(text::integer(input), Ok(expected), "{input}");
    }
    for input in [
        "",
        "_1",
        "1_",
        "1__0",
        "$",
        "$FF_",
        "$FFFFFFFFFFFFFFFF",
        "9223372036854775808",
        "-9223372036854775809",
        "++1",
        "- 1",
        "0xFF",
        "1.0",
    ] {
        assert!(text::integer(input).is_err(), "{input}");
    }
    for (input, expected) in [
        ("FF", 255),
        ("0xFF", 255),
        (" +0XFF ", 255),
        ("-$80_0000_0000_0000_00", i64::MIN),
        ("-8000000000000000", i64::MIN),
    ] {
        assert_eq!(text::hex_integer(input), Ok(expected), "{input}");
    }
    for (input, expected) in [
        ("3.14", 314.0 / 100.0),
        (" -1.5e2 ", -150.0),
        ("+0.5", 0.5),
        ("1_000.5_0", 1000.5),
        ("1_024.0e-2", 10.24),
        ("0.0e-9999", 0.0),
    ] {
        assert_eq!(text::real(input), Ok(expected), "{input}");
    }
    for input in [
        "1e3",
        ".5",
        "5.",
        "NaN",
        "inf",
        "1.0e999",
        "1.0e-9999",
        "1.0e",
        "1._0",
        "1__0.0",
        "1.0e_2",
        "1.0e+_2",
    ] {
        assert!(text::real(input).is_err(), "{input}");
        assert!(!text::is_numeric(input), "{input}");
    }
}

#[test]
fn finite_real_formatting_roundtrips_identical_bits_including_signed_zero() {
    for bits in [
        0_u64,
        1 << 63,
        1,
        (1 << 63) | 1,
        0x0010_0000_0000_0000,
        0x3ff0_0000_0000_0000,
        0xbff0_0000_0000_0000,
        0x7fef_ffff_ffff_ffff,
        0xffef_ffff_ffff_ffff,
    ] {
        let value = f64::from_bits(bits);
        let formatted = text::format_real(value).expect("finite value formats");
        let parsed = text::real(&formatted).expect("formatted real parses");
        assert_eq!(parsed.to_bits(), bits, "{formatted}");
    }
    assert_eq!(text::format_real(1.0).expect("finite"), "1.0");
    assert_eq!(text::format_real(-0.0).expect("finite"), "-0.0");
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            text::format_real(value),
            Err(crate::NumericError::NonFiniteReal)
        );
    }
}

#[test]
fn complete_syntax_precedes_range_classification() {
    use text::TextError::{InvalidSyntax, OutOfRange};
    for input in [
        "9223372036854775808",
        "999999999999999999999999999999999999",
        "-9223372036854775809",
    ] {
        assert_eq!(text::integer(input), Err(OutOfRange), "{input}");
        assert_eq!(
            text::integer(&format!("{input}x")),
            Err(InvalidSyntax),
            "{input}"
        );
    }
    for input in ["8000000000000000", "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"] {
        assert_eq!(text::hex_integer(input), Err(OutOfRange), "{input}");
        assert_eq!(
            text::hex_integer(&format!("{input}z")),
            Err(InvalidSyntax),
            "{input}"
        );
    }
    for input in ["1.0e9999", "1.0e-9999"] {
        assert_eq!(text::real(input), Err(OutOfRange), "{input}");
        assert_eq!(
            text::real(&format!("{input}x")),
            Err(InvalidSyntax),
            "{input}"
        );
    }
}
