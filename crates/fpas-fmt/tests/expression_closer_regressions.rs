//! AP13.6 expression ending layout and comment-preserving round trips.

#![allow(
    clippy::expect_used,
    reason = "test fixtures fail fast with direct assertions for diagnostic clarity"
)]

mod common;

fn format(source: &str) -> String {
    common::assert_round_trip("expression closers", source);
    let (unit, errors) = fpas_parser::parse_compilation_unit(source);
    assert!(errors.is_empty(), "{errors:#?}");
    fpas_fmt::format_source(source, &unit).expect("source and AST agree")
}

#[test]
fn arguments_and_enclosing_declarations_have_distinct_terminator_owners() {
    let formatted = format(
        "program T; begin const F: procedure() := procedure() begin end procedure; Apply(function(): integer begin return 1; end function, P with X := 2; end with); end.",
    );
    assert!(
        formatted.contains("procedure() begin end procedure;"),
        "{formatted}"
    );
    assert!(
        formatted.contains("end function, P with X := 2; end with);"),
        "{formatted}"
    );
    assert!(!formatted.contains("end function;,"));
    assert!(!formatted.contains("end with;("));
}

#[test]
fn nested_closures_updates_and_constructions_keep_their_matching_endings() {
    let formatted = format(
        "program T; function Make(): function(): integer; begin return function(): integer begin const Q: Holder := P with Child := P.Child with X := 1; end with; Data := Point( X := 2 ); Reader := function(): integer begin return 3; end function; end with; return Q.Reader(); end function; end function; begin end.",
    );
    assert_eq!(formatted.matches("end function").count(), 3, "{formatted}");
    assert_eq!(formatted.matches("end with").count(), 2, "{formatted}");
    assert!(formatted.contains("Data := Point(X := 2);"), "{formatted}");
}

#[test]
fn comments_before_inside_and_after_expression_endings_survive_once() {
    let formatted = format(
        "program T; begin
        Apply(procedure()
        // before closure body
        begin
          null; // body statement
        // before procedure end
        end // inside procedure end
        procedure // argument ending
        , P with
          // before field
          X := 1; // field ending
        // before update end
        end // inside update end
        with // update argument ending
        );
        const F: procedure() := procedure() begin end procedure; // declaration ending
        end.",
    );
    for comment in [
        "before closure body",
        "body statement",
        "before procedure end",
        "inside procedure end",
        "argument ending",
        "before field",
        "field ending",
        "before update end",
        "inside update end",
        "update argument ending",
        "declaration ending",
    ] {
        assert_eq!(
            formatted.matches(&format!("// {comment}\n")).count(),
            1,
            "{formatted}"
        );
    }
    assert!(
        formatted.contains("end procedure // argument ending"),
        "{formatted}"
    );
    assert!(
        formatted.contains("end with // update argument ending"),
        "{formatted}"
    );
    assert!(
        formatted.contains("end procedure; // declaration ending"),
        "{formatted}"
    );
}

#[test]
fn empty_closure_ending_comments_are_preserved() {
    let formatted = format(
        "program T; begin const F: procedure() := procedure() begin
        // empty body
        end // inside empty closer
        procedure; end.",
    );
    assert_eq!(formatted.matches("// empty body").count(), 1);
    assert_eq!(formatted.matches("// inside empty closer").count(), 1);
    assert!(formatted.contains("end procedure;"), "{formatted}");
}

#[test]
fn closure_fields_keep_ending_comments_inside_constructor_arguments() {
    let formatted = format(
        "program T; begin const P: Holder := Holder(
        // reader field
        Reader := function(): integer begin return 1;
        // field closure ending
        end function // reader terminator
        ); end.",
    );
    assert!(
        formatted.contains("end function // reader terminator\n  );"),
        "{formatted}"
    );
    assert!(!formatted.contains("end function; // reader terminator"));
    for comment in ["reader field", "field closure ending", "reader terminator"] {
        assert_eq!(
            formatted.matches(&format!("// {comment}")).count(),
            1,
            "{formatted}"
        );
    }
}

#[test]
fn if_expressions_stay_compact_or_break_before_each_branch() {
    common::assert_golden(
        "compact if expression",
        "program T; begin const Noun: string := if Count = 1 then 'item' elsif Count = 0 then 'none' else 'items' end if; end.",
        "program T;\n\nbegin\n  const Noun: string := if Count = 1 then 'item' elsif Count = 0 then 'none' else 'items' end if;\nend.\n",
    );
    let formatted = format(
        "program T; begin const Long: string := if Mode = 1 then 'a rather long first branch value' elsif Mode = 2 then 'another long branch value here' else 'and the final else branch' end if; end.",
    );
    assert!(
        formatted.contains(
            "const Long: string := if Mode = 1 then 'a rather long first branch value'\n    elsif Mode = 2 then 'another long branch value here'\n    else 'and the final else branch'\n    end if;"
        ),
        "{formatted}"
    );
}

#[test]
fn if_expression_comments_stay_with_their_branches() {
    let formatted = format(
        "program T; begin const X: integer := if A then 1 // first\n // before elsif\n elsif B then 2\n else 3 // last\n end if; // after\n end.",
    );
    assert!(
        formatted.contains(
            "const X: integer := if A then 1 // first\n    // before elsif\n    elsif B then 2\n    else 3 // last\n    end if; // after"
        ),
        "{formatted}"
    );
}

#[test]
fn case_expressions_put_each_arm_on_its_own_line() {
    common::assert_golden(
        "case expression",
        "program T; function F(X: integer): string; begin return case X of when 1, 2: 'low'; when 3..9 if X > 4: 'mid'; else 'other'; end case; end function; begin end.",
        "program T;\n\nfunction F(X: integer): string;\nbegin\n  return case X of\n    when 1, 2: 'low';\n    when 3..9 if X > 4: 'mid';\n    else 'other';\n  end case;\nend function;\n\nbegin\nend.\n",
    );
    let formatted = format(
        "program T; begin const X: integer := case 1 of\n // first arm\n when 1: 10; // ten\n else 20; // other\n end case; // after\n end.",
    );
    assert!(
        formatted.contains(
            "const X: integer := case 1 of\n    // first arm\n    when 1: 10; // ten\n    else 20; // other\n  end case; // after"
        ),
        "{formatted}"
    );
}
