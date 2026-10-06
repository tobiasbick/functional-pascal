//! Terminators in nested bodies, expressions, and comments.

mod common;

use fpas_parser::parse_compilation_unit;

#[test]
#[expect(
    clippy::expect_used,
    reason = "the fixture asserts that matching source and AST can be formatted"
)]
fn formatter_emits_all_terminators_and_preserves_comments() {
    let source = "program P; begin\nif C then A(); // first\nelse B(); // last\nend if;\nrepeat C(); // body\nuntil Done; // loop\nend.";
    let (unit, diagnostics) = parse_compilation_unit(source);
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    let formatted = fpas_fmt::format_source(source, &unit).expect("matching source and AST");
    assert!(formatted.contains("A(); // first"), "{formatted}");
    assert!(formatted.contains("B(); // last"), "{formatted}");
    assert!(formatted.contains("// first\n  else"), "{formatted}");
    assert!(formatted.contains("until Done; // loop"), "{formatted}");
    assert!(formatted.ends_with("end.\n"), "{formatted}");
    common::assert_round_trip("terminated comments", &formatted);
}

#[test]
fn anonymous_routine_body_is_terminated_inside_an_argument() {
    let source = "program P; begin Consume(function(): integer begin return 1; end); end.";
    common::assert_round_trip("anonymous routine body", source);
    let (unit, diagnostics) = parse_compilation_unit(source);
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    let formatted = fpas_fmt::format_compilation_unit(&unit);
    assert!(formatted.contains("return 1;"), "{formatted}");
    assert!(!formatted.contains("end;)"), "{formatted}");
}

#[test]
fn nested_control_bodies_format_without_duplicate_terminators() {
    for source in [
        "program P; begin if C then while D do A(); end while; else B(); end if; end.",
        "program P; begin for I: integer in Values do if C then A(); else B(); end if; end for; end.",
        "program P; begin case V of 1: A(); else B(); end; end.",
    ] {
        common::assert_round_trip("single body", source);
        let (unit, _) = parse_compilation_unit(source);
        let formatted = fpas_fmt::format_compilation_unit(&unit);
        assert!(!formatted.contains(";;"), "{formatted}");
    }
}
