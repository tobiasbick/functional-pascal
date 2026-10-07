//! AP13.5 case indentation, explicit scopes, and comment ownership.

#![allow(
    clippy::expect_used,
    reason = "test fixtures fail fast with direct assertions for diagnostic clarity"
)]

mod common;

fn format(source: &str) -> String {
    common::assert_round_trip("case blocks", source);
    let (unit, errors) = fpas_parser::parse_compilation_unit(source);
    assert!(errors.is_empty(), "{errors:#?}");
    fpas_fmt::format_source(source, &unit).expect("source and AST agree")
}

#[test]
fn arm_lists_indent_without_inserting_compound_blocks() {
    let formatted = format(
        "program T; begin case X of when 1, 2: A(); B(); when 3..5 if Ready: null; else C(); D(); end case; end.",
    );
    assert!(formatted.contains("case X of\n    when 1, 2:\n      A();\n      B();\n    when 3..5 if Ready:\n      null;\n    else\n      C();\n      D();\n  end case;"), "{formatted}");
    assert_eq!(formatted.matches("begin").count(), 1);
}

#[test]
fn nested_cases_and_explicit_blocks_keep_their_endings() {
    let formatted = format(
        "program T; begin case X of when 1: begin A(); end; case Y of when Some(V): B(V); when None: null; end case; else null; end case; end.",
    );
    assert_eq!(formatted.matches("end case;").count(), 2);
    assert!(
        formatted.contains("when 1:\n      begin\n        A();\n      end;"),
        "{formatted}"
    );
    assert!(
        formatted.contains("when Some(V):\n          B(V);"),
        "{formatted}"
    );
}

#[test]
fn comments_between_arms_and_inside_named_endings_survive_once() {
    let formatted = format(
        "program T; begin
        case X of
        // first arm
        when 1: // first header
          null; // first statement
        // next arm
        when 2: null;
        // catch-all
        else
          null;
        // before case end
        end // inside case end
        case; // after case end
        end.",
    );
    for comment in [
        "first arm",
        "first header",
        "first statement",
        "next arm",
        "catch-all",
        "before case end",
        "inside case end",
        "after case end",
    ] {
        assert_eq!(
            formatted.matches(&format!("// {comment}")).count(),
            1,
            "{formatted}"
        );
    }
    assert!(
        formatted.contains("// next arm\n    when 2:"),
        "{formatted}"
    );
    assert!(formatted.contains("// catch-all\n    else"), "{formatted}");
    assert!(
        formatted.contains("end case; // after case end"),
        "{formatted}"
    );
}
