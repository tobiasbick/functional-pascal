//! Comment-preservation regressions for callable bodies, closures, and member lines.

#![allow(clippy::expect_used)]

mod common;

use fpas_fmt::format_source;
use fpas_parser::parse_compilation_unit;

#[test]
fn empty_unit_closer_comment_stays_on_the_closer() {
    let source = "unit Empty; end unit; // tail";
    common::assert_golden(
        "empty unit closer comment",
        source,
        "unit Empty;\n\nend unit; // tail\n",
    );
    common::assert_round_trip("empty unit closer comment", source);
}

#[test]
fn empty_unit_header_and_closer_comments_keep_distinct_anchors() {
    for source in [
        "unit Empty; // header\nend unit; // closer",
        "unit Empty; // header\r\nend unit; // closer",
        "unit Empty; // header\nuses Std.Str as Text; // import\nend unit; // closer",
    ] {
        let formatted = format_idempotently(source);
        assert!(formatted.contains("unit Empty; // header\n"), "{formatted}");
        assert!(formatted.contains("end unit; // closer\n"), "{formatted}");
        if source.contains("uses") {
            assert!(
                formatted.contains("uses Std.Str as Text; // import\n"),
                "{formatted}"
            );
        }
    }
}

fn format_idempotently(source: &str) -> String {
    let (unit, diagnostics) = parse_compilation_unit(source);
    assert!(diagnostics.is_empty(), "source must parse: {diagnostics:?}");
    let formatted = format_source(source, &unit).expect("matching source and AST");
    common::assert_round_trip("comment regression", &formatted);
    formatted
}

#[test]
fn callable_body_comments_stay_with_each_begin() {
    let source = r#"program T;
procedure First();
// first body
begin null;
end procedure;
procedure Second();
// second body
begin null;
end procedure;
// main body
begin null; end program;"#;
    let formatted = format_idempotently(source);

    assert!(formatted.contains("procedure First();\n// first body\nbegin"));
    assert!(formatted.contains("procedure Second();\n// second body\nbegin"));
    assert!(formatted.contains("end procedure;\n\n// main body\nbegin"));
}

#[test]
fn nested_routine_body_comments_use_structural_owners() {
    let source = r#"unit Demo;
procedure Outer();
procedure Inner();
// inner body
begin null;
end procedure;
// outer body
begin null;
end procedure;
end unit;
"#;
    let formatted = format_idempotently(source);

    assert!(formatted.contains("procedure Inner();\n// inner body\nbegin"));
    assert!(formatted.contains("end procedure;\n// outer body\nbegin"));
}

#[test]
fn closure_comments_survive_expression_emission() {
    let source = r#"program T;
begin
  const Handler: procedure() := procedure()
  // closure body
  begin
    // setup
    WriteLn('ok'); // closure trail
  end procedure;
  Handler();
end program;"#;
    let formatted = format_idempotently(source);

    for comment in ["// closure body", "// setup", "// closure trail"] {
        assert!(
            formatted.contains(comment),
            "missing {comment}:\n{formatted}"
        );
    }
}

#[test]
fn record_enum_and_routine_eol_comments_remain_on_member_lines() {
    let source = r#"program T;
 type Shape = enum
  // leading member
  Plain; // plain
  Valued = 2; // valued
  Circle(Radius: real); // payload
end enum;
type Counter = record
  Value: integer; // field
  Current: function(): integer; // callable field
  Changed: Option of (procedure()) := Option.None; // optional handler
end record;
function CounterReadValue(Receiver: Counter): integer;
begin return Receiver.Value;
end function; // accessor
function Top(): integer;
begin
  return 1;
end function; // top routine
begin null; end program;"#;
    let formatted = format_idempotently(source);

    for line in [
        "Plain; // plain",
        "Valued = 2; // valued",
        "Circle(Radius: real); // payload",
        "Value: integer; // field",
        "end function; // accessor",
        "Current: function(): integer; // callable field",
        "Changed: Option of (procedure()) := Option.None; // optional handler",
        "end function; // top routine",
    ] {
        assert!(formatted.contains(line), "missing `{line}`:\n{formatted}");
    }
    assert!(formatted.contains("// leading member\n  Plain"));
}

#[test]
fn explicit_block_eol_comment_precedes_the_statement_separator() {
    let source = r#"program T; begin if true then begin WriteLn('yes'); end; end if; // block tail
WriteLn('done'); end program;"#;
    let formatted = format_idempotently(source);

    assert!(formatted.contains("end if; // block tail\n"), "{formatted}");
    assert!(!formatted.contains("// block tail;"), "{formatted}");
}

#[test]
fn cr_only_input_preserves_comment_line_ownership() {
    let source = "program T;\rbegin\r  // setup\r  WriteLn('ok'); // trail\rend program; // tail\r";
    let formatted = format_idempotently(source);

    assert!(!formatted.contains('\r'));
    assert!(formatted.contains("// setup\n  WriteLn('ok'); // trail\n"));
    assert!(formatted.contains("end program; // tail\n"));
}

#[test]
fn branch_comments_survive_single_and_explicit_block_bodies() {
    let source = r#"program T; begin if true then // single branch
WriteLn('single'); end if; if false then
// explicit block
begin WriteLn('block'); end; end if; end program;"#;
    let formatted = format_idempotently(source);

    assert!(formatted.contains("// single branch\n    WriteLn('single')"));
    assert!(formatted.contains("then\n    // explicit block\n    begin"));
}

#[test]
fn compilation_and_routine_header_comments_stay_on_header_lines() {
    let program_source = r#"program T; // program header
procedure Work(); // routine header
begin null;
end procedure; // routine end
begin null; end program;"#;
    let program = format_idempotently(program_source);
    assert!(program.contains("program T; // program header\n"));
    assert!(program.contains("procedure Work(); // routine header\n"));
    assert!(program.contains("end procedure; // routine end\n"));

    let unit_source = r#"unit Demo; // unit header
procedure Work(); // unit routine header
begin null;
end procedure;
end unit;
"#;
    let unit = format_idempotently(unit_source);
    assert!(unit.contains(
        r#"unit Demo; // unit header
"#
    ));
    assert!(unit.contains("procedure Work(); // unit routine header\n"));
}

#[test]
fn eol_comment_stays_on_its_code_line() {
    let source = r#"program T; begin const A: integer := 1; // value
WriteLn(A); end program;"#;
    let formatted = format_idempotently(source);

    assert!(formatted.contains("const A: integer := 1; // value\n"));
}

#[test]
fn uses_item_comments_survive_formatting() {
    let source = r#"program T;
 uses Std.Console as Console; // io
  uses Std.Conv as Conv;
begin null;
end program;"#;
    let formatted = format_idempotently(source);

    assert!(
        formatted.contains("uses Std.Console as Console; // io\n"),
        "{formatted}"
    );
}

#[test]
fn standalone_comment_between_uses_items_survives_formatting() {
    let source = r#"program T;
 uses Std.Console as Console;
  // conversions
  uses Std.Conv as Conv;
begin null;
end program;"#;
    let formatted = format_idempotently(source);

    assert!(
        formatted.contains("uses Std.Console as Console;\n// conversions\nuses Std.Conv as Conv;"),
        "{formatted}"
    );
}
