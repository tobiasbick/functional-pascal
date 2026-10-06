//! AP13.6 expression endings, terminator ownership, and boundary recovery.

use fpas_diagnostics::codes::{PARSE_EMPTY_RECORD_UPDATE, PARSE_EXPECTED_TOKEN};
use fpas_parser::{CompilationUnit, Expr, Stmt, parse_compilation_unit, parse_expression};

fn expression_ok(source: &str) -> Expr {
    let (expr, errors) = parse_expression(source);
    assert!(errors.is_empty(), "{source}\n{errors:#?}");
    expr
}

fn program_ok(source: &str) {
    let (_, errors) = parse_compilation_unit(source);
    assert!(errors.is_empty(), "{source}\n{errors:#?}");
}

#[test]
fn anonymous_routines_include_nested_declarations_and_empty_bodies() {
    for source in [
        "function(): integer begin return 1; end function",
        "procedure() begin end procedure",
        "function(): integer function Inner(): integer; begin return 2; end function; begin return Inner(); end function",
        "function(): function(): integer begin return function(): integer begin return 3; end function; end function",
        "PROCEDURE() BEGIN NULL; END PROCEDURE",
    ] {
        assert!(matches!(expression_ok(source), Expr::Closure(_)));
    }
}

#[test]
fn enclosing_declarations_returns_and_arguments_own_their_terminators() {
    program_ok(
        "program T;
        var F: function(): integer := function(): integer begin return 1; end function;
        function Make(): procedure(); begin return procedure() begin null; end procedure; end function;
        begin
        Apply(function(): integer begin return 2; end function, procedure() begin null; end procedure);
        F := function(): integer begin return 3; end function;
        end.",
    );
}

#[test]
fn updates_nest_with_literals_closures_and_parenthesized_chaining() {
    for source in [
        "P with X := 1; end with",
        "P with Child := Q with X := 1; end with; Reader := function(): integer begin return 2; end function; end with",
        "P with Child := record X := 1; end; Items := []; end with",
        "(P with X := 1; end with) with Y := 2; end with",
        "P WITH X := 1; END WITH",
    ] {
        assert!(matches!(expression_ok(source), Expr::RecordUpdate { .. }));
    }
    program_ok(
        "program T; begin Apply(P with X := 1; end with, 2); return P with X := 2; end with; end.",
    );
    assert!(matches!(
        expression_ok("record end"),
        Expr::RecordLiteral { .. }
    ));
}

#[test]
fn expression_spans_include_the_named_ending() {
    for source in [
        "function(): integer begin return 1; end function",
        "procedure() begin end procedure",
        "P with X := 1; end with",
    ] {
        assert_eq!(expression_ok(source).span().length, source.len());
    }
}

#[test]
fn legacy_and_mismatched_endings_name_the_expression_closer_without_a_semicolon() {
    for (source, expected) in [
        ("function(): integer begin return 1; end", "end function"),
        ("procedure() begin null; end", "end procedure"),
        ("P with X := 1; end", "end with"),
        (
            "function(): integer begin return 1; end procedure",
            "end function",
        ),
        ("procedure() begin null; end with", "end procedure"),
        ("P with X := 1; end procedure", "end with"),
    ] {
        let (_, errors) = parse_expression(source);
        let error = errors
            .iter()
            .filter_map(|error| error.as_parser_error())
            .find(|error| error.expected.as_deref() == Some(expected))
            .unwrap_or_else(|| panic!("{source}: {errors:#?}"));
        assert_eq!(error.code, PARSE_EXPECTED_TOKEN);
        assert!(
            error
                .help
                .as_deref()
                .unwrap()
                .contains(&format!("`{expected}`"))
        );
        assert!(
            !error
                .help
                .as_deref()
                .unwrap()
                .contains(&format!("`{expected};`"))
        );
    }
}

#[test]
fn extra_argument_terminators_are_rejected_before_parentheses_and_commas() {
    for arg in [
        "procedure() begin null; end procedure",
        "function(): integer begin return 1; end function",
        "P with X := 1; end with",
    ] {
        for tail in ["); Next(); end.", ", 2); Next(); end."] {
            let source = format!("program T; begin Apply({arg};{tail}");
            let (_, errors) = parse_compilation_unit(&source);
            assert!(
                errors
                    .iter()
                    .filter_map(|error| error.as_parser_error())
                    .any(|error| {
                        error.code == PARSE_EXPECTED_TOKEN
                            && error
                                .help
                                .as_deref()
                                .is_some_and(|help| help.contains("Remove `;`"))
                    }),
                "{source}: {errors:#?}"
            );
        }
    }
}

#[test]
fn missing_expression_endings_preserve_later_arguments_and_statements() {
    for arg in [
        "procedure() begin null;",
        "P with X := 1;",
        "procedure() begin null; end",
        "P with X := 1; end",
    ] {
        let source = format!("program T; begin Apply({arg}, 2); Next(); end.");
        let (unit, errors) = parse_compilation_unit(&source);
        let CompilationUnit::Program(program) = unit else {
            panic!("expected program")
        };
        assert_eq!(program.body.len(), 2, "{source}: {program:#?}");
        assert!(matches!(&program.body[0], Stmt::Call { args, .. } if args.len() == 2));
        assert_eq!(errors.len(), 1, "{source}: {errors:#?}");
    }
}

#[test]
fn missing_expression_endings_preserve_enclosing_routines_and_cases() {
    let (unit, errors) = parse_compilation_unit(
        "unit T; procedure Outer(); begin var F: function(): integer := function(): integer begin return 1; end procedure; procedure Next(); begin end procedure; end unit;",
    );
    let CompilationUnit::Unit(unit) = unit else {
        panic!("expected unit")
    };
    assert_eq!(unit.declarations.len(), 2, "{unit:#?}");
    assert!(
        errors
            .iter()
            .filter_map(|error| error.as_parser_error())
            .any(|error| error.expected.as_deref() == Some("end function"))
    );

    let (unit, errors) = parse_compilation_unit(
        "program T; begin case X of when 1: var P: Point := Q with X := 1; when 2: null; end case; Next(); end.",
    );
    let CompilationUnit::Program(program) = unit else {
        panic!("expected program")
    };
    assert_eq!(program.body.len(), 2);
    assert!(matches!(&program.body[0], Stmt::Case { arms, .. } if arms.len() == 2));
    assert!(
        errors
            .iter()
            .filter_map(|error| error.as_parser_error())
            .any(|error| error.expected.as_deref() == Some("end with"))
    );
}

#[test]
fn body_and_field_terminators_remain_required() {
    for source in [
        "function(): integer begin return 1 end function",
        "procedure() begin null end procedure",
        "P with X := 1 end with",
    ] {
        let (_, errors) = parse_expression(source);
        assert!(
            errors
                .iter()
                .filter_map(|error| error.as_parser_error())
                .any(|error| error.expected.as_deref() == Some(";")),
            "{errors:#?}"
        );
    }
    let (_, errors) = parse_expression("P with end with");
    assert!(
        errors
            .iter()
            .filter_map(|error| error.as_parser_error())
            .any(|error| error.code == PARSE_EMPTY_RECORD_UPDATE)
    );
}
