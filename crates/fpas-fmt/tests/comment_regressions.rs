//! Comment-preservation regressions for callable bodies, closures, and member lines.

#![allow(clippy::expect_used)]

mod common;

use fpas_fmt::format_source;
use fpas_parser::parse_compilation_unit;

fn format_idempotently(source: &str) -> String {
    let (unit, diagnostics) = parse_compilation_unit(source);
    assert!(diagnostics.is_empty(), "source must parse: {diagnostics:?}");
    let formatted = format_source(source, &unit).expect("matching source and AST");
    common::assert_round_trip("comment regression", &formatted);
    formatted
}

#[test]
fn callable_body_comments_stay_with_each_begin() {
    let source = "program T;\nprocedure First();\n// first body\nbegin\nend procedure;\nprocedure Second();\n// second body\nbegin\nend procedure;\n// main body\nbegin\nend.";
    let formatted = format_idempotently(source);

    assert!(formatted.contains("procedure First();\n// first body\nbegin"));
    assert!(formatted.contains("procedure Second();\n// second body\nbegin"));
    assert!(formatted.contains("end procedure;\n\n// main body\nbegin"));
}

#[test]
fn nested_routine_body_comments_use_structural_owners() {
    let source = "unit Demo;\nprocedure Outer();\nprocedure Inner();\n// inner body\nbegin\nend procedure;\n// outer body\nbegin\nend procedure;\nend unit;";
    let formatted = format_idempotently(source);

    assert!(formatted.contains("procedure Inner();\n// inner body\nbegin"));
    assert!(formatted.contains("end procedure;\n// outer body\nbegin"));
}

#[test]
fn closure_comments_survive_expression_emission() {
    let source = "program T;\nbegin\n  var Handler: procedure() := procedure()\n  // closure body\n  begin\n    // setup\n    WriteLn('ok'); // closure trail\n  end procedure;\n  Handler();\nend.";
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
    let source = "program T;\ntype Shape = enum\n  // leading member\n  Plain; // plain\n  Valued = 2; // valued\n  Circle(Radius: real); // payload\nend enum;\nCounter = record\n  Value: integer; // field\n  function ReadValue(Self: Counter): integer;\n  begin\n    return Self.Value;\n  end function; // method\n  property Current: integer read ReadValue; // property\n  event Changed: procedure() read ReadChanged write WriteChanged; // event\nend record;\nfunction Top(): integer;\nbegin\n  return 1;\nend function; // top routine\nbegin\nend.";
    let formatted = format_idempotently(source);

    for line in [
        "Plain; // plain",
        "Valued = 2; // valued",
        "Circle(Radius: real); // payload",
        "Value: integer; // field",
        "end function; // method",
        "property Current: integer read ReadValue; // property",
        "event Changed: procedure() read ReadChanged write WriteChanged; // event",
        "end function; // top routine",
    ] {
        assert!(formatted.contains(line), "missing `{line}`:\n{formatted}");
    }
    assert!(formatted.contains("// leading member\n    Plain"));
}

#[test]
fn explicit_block_eol_comment_precedes_the_statement_separator() {
    let source = "program T; begin if true then begin WriteLn('yes'); end; // block tail\nend if; WriteLn('done'); end.";
    let formatted = format_idempotently(source);

    assert!(formatted.contains("end; // block tail\n"), "{formatted}");
    assert!(!formatted.contains("// block tail;"), "{formatted}");
}

#[test]
fn cr_only_input_preserves_comment_line_ownership() {
    let source = "program T;\rbegin\r  // setup\r  WriteLn('ok'); // trail\rend. // tail\r";
    let formatted = format_idempotently(source);

    assert!(!formatted.contains('\r'));
    assert!(formatted.contains("// setup\n  WriteLn('ok'); // trail\n"));
    assert!(formatted.contains("end. // tail\n"));
}

#[test]
fn branch_comments_survive_single_and_explicit_block_bodies() {
    let source = "program T; begin if true then // single branch\nWriteLn('single'); end if; if false then\n// explicit block\nbegin WriteLn('block'); end; end if; end.";
    let formatted = format_idempotently(source);

    assert!(formatted.contains("// single branch\n    WriteLn('single')"));
    assert!(formatted.contains("then\n    // explicit block\n    begin"));
}

#[test]
fn compilation_and_routine_header_comments_stay_on_header_lines() {
    let program_source = "program T; // program header\nprocedure Work(); // routine header\nbegin\nend procedure; // routine end\nbegin\nend.";
    let program = format_idempotently(program_source);
    assert!(program.contains("program T; // program header\n"));
    assert!(program.contains("procedure Work(); // routine header\n"));
    assert!(program.contains("end procedure; // routine end\n"));

    let unit_source = "unit Demo; // unit header\nprocedure Work(); // unit routine header\nbegin\nend procedure;\nend unit;";
    let unit = format_idempotently(unit_source);
    assert!(unit.contains("unit Demo; // unit header\n"));
    assert!(unit.contains("procedure Work(); // unit routine header\n"));
}

#[test]
fn eol_comment_stays_on_its_code_line() {
    let source = "program T; begin var A: integer := 1; // value\nWriteLn(A); end.";
    let formatted = format_idempotently(source);

    assert!(formatted.contains("var A: integer := 1; // value\n"));
}

#[test]
fn uses_item_comments_survive_formatting() {
    let source = "program T;\nuses Std.Console, // io\n  Std.Conv;\nbegin\nend.";
    let formatted = format_idempotently(source);

    assert!(formatted.contains("Std.Console, // io\n"), "{formatted}");
}

#[test]
fn standalone_comment_between_uses_items_survives_formatting() {
    let source = "program T;\nuses Std.Console,\n  // conversions\n  Std.Conv;\nbegin\nend.";
    let formatted = format_idempotently(source);

    assert!(
        formatted.contains("Std.Console,\n  // conversions\n  Std.Conv;"),
        "{formatted}"
    );
}
