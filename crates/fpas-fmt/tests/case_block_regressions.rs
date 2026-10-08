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
        "program T; begin case X of when 1: begin A(); end; case Y of when Some(const V): B(V); when None: null; end case; else null; end case; end.",
    );
    assert_eq!(formatted.matches("end case;").count(), 2);
    assert!(
        formatted.contains("when 1:\n      begin\n        A();\n      end;"),
        "{formatted}"
    );
    assert!(
        formatted.contains("when Some(const V):\n          B(V);"),
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

#[test]
fn explicit_pattern_bindings_and_wildcards_round_trip() {
    let formatted = format(
        "program T; begin case S of when Shape.Rect(const W,_): A(W); when Shape.Circle(Radius:=const R): null; end case; case N of when const V if V>0: null; else null; end case; case O of when Error( _ ): null; when Ok(const Value): null; end case; end.",
    );
    for line in [
        "when Shape.Rect(const W, _):",
        "when Shape.Circle(Radius := const R):",
        "when const V if V > 0:",
        "when Error(_):",
        "when Ok(const Value):",
    ] {
        assert!(formatted.contains(line), "missing `{line}`:\n{formatted}");
    }
}

#[test]
fn nested_patterns_round_trip() {
    let formatted = format(
        "program T; begin case R of when Ok(Some(Shape.Rect(const W,0))): null; when Ok( None ): null; when Error('x'): null; else null; end case; end.",
    );
    for line in [
        "when Ok(Some(Shape.Rect(const W, 0))):",
        "when Ok(None):",
        "when Error('x'):",
    ] {
        assert!(
            formatted.contains(line),
            "missing `{line}`:
{formatted}"
        );
    }
}

#[test]
fn is_tests_round_trip_with_comparison_precedence() {
    let formatted = format(
        "program T; begin if X is Some( const V ) and V>0 then null; elsif not (X is None) then null; end if; while Q.Next() is Some(const Job) do null; end while; end.",
    );
    for line in [
        "if X is Some(const V) and V > 0 then",
        "elsif not (X is None) then",
        "while Q.Next() is Some(const Job) do",
    ] {
        assert!(
            formatted.contains(line),
            "missing `{line}`:
{formatted}"
        );
    }
}
