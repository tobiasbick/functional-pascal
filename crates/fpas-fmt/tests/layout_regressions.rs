//! Layout regressions for multiline arrays and local-variable emission.

#![allow(clippy::expect_used)]

mod common;

use fpas_fmt::format_source;
use fpas_parser::parse_compilation_unit;

fn format(source: &str) -> String {
    let (unit, diagnostics) = parse_compilation_unit(source);
    assert!(diagnostics.is_empty(), "source must parse: {diagnostics:?}");
    format_source(source, &unit).expect("matching source and AST")
}

#[test]
fn wrapped_array_has_no_blank_line_before_closing_bracket() {
    let source = r#"program T; begin const Values: array of (string) := ['aaaaaaaaaaaaaaaaaaaa', 'bbbbbbbbbbbbbbbbbbbb', 'cccccccccccccccccccc', 'dddddddddddddddddddd']; end program;"#;
    let formatted = format(source);

    assert!(formatted.contains("[\n"), "array should wrap:\n{formatted}");
    assert!(!formatted.contains("\n\n  ]"), "{formatted}");
    common::assert_round_trip("wrapped array", &formatted);
}

#[test]
fn multiline_record_and_closure_array_elements_have_single_line_breaks() {
    let record_source = r#"program T;

type Point = record
  X: integer;
end record;

begin
  const Values: array of (Point) := [Point(X := 1), Point(X := 2)];
end program;
"#;
    let record_formatted = format(record_source);
    assert!(
        !record_formatted.contains("\n\n      X :="),
        "{record_formatted}"
    );
    assert!(!record_formatted.contains("\n\n  ]"), "{record_formatted}");
    common::assert_round_trip("record array", &record_formatted);

    let closure_source = r#"program T; begin const Values: array of (procedure()) := [procedure() begin WriteLn('first'); end procedure, procedure() begin WriteLn('second'); end procedure]; end program;"#;
    let closure_formatted = format(closure_source);
    let closure_array = closure_formatted
        .split_once(":= [")
        .and_then(|(_, rest)| rest.split_once("];\n"))
        .map(|(array, _)| array)
        .expect("formatted closure array");
    assert!(!closure_array.contains("\n\n"), "{closure_formatted}");
    common::assert_round_trip("closure array", &closure_formatted);
}

#[test]
fn var_and_mutable_var_eol_comments_keep_existing_output_shape() {
    let source = r#"program T; begin const A: integer := 1; // immutable
 var B: integer := 2; // mutable
end program;"#;
    let formatted = format(source);

    assert!(formatted.contains("const A: integer := 1; // immutable\n"));
    assert!(formatted.contains("var B: integer := 2; // mutable\n"));
    common::assert_round_trip("variable comments", &formatted);
}
