//! AP13.3 declaration endings, diagnostics, and enclosing-boundary recovery.

use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;
use fpas_parser::{CompilationUnit, Decl, parse_compilation_unit};

fn parse_ok(source: &str) -> CompilationUnit {
    let (unit, errors) = parse_compilation_unit(source);
    assert!(errors.is_empty(), "{source}\n{errors:#?}");
    unit
}

#[test]
fn all_declaration_kinds_include_methods_and_nested_routines() {
    let source = "unit Demo; type Color = enum Red; end enum; Point = record
        X: integer;
        function ReadValue(self: Point): integer; begin return self.X; end function;
        procedure WriteValue(mutable self: Point); begin return; end procedure;
        static function Create(): Point; begin return record X := 1; end; end function;
        static procedure Reset(); begin return; end procedure;
        end record;
        function Outer(): integer;
            procedure Inner(); begin return; end procedure;
        begin Inner(); return 1; end function;
        end unit;";
    let CompilationUnit::Unit(unit) = parse_ok(source) else {
        panic!("expected unit")
    };
    assert_eq!(unit.declarations.len(), 3);
    assert_eq!(unit.span.length, source.len());
}

#[test]
fn declaration_keywords_are_case_insensitive() {
    parse_ok(
        "UNIT Demo; TYPE E = ENUM A; END ENUM; R = RECORD END RECORD;
        FUNCTION F(): integer; BEGIN RETURN 1; END FUNCTION;
        PROCEDURE P(); BEGIN RETURN; END PROCEDURE; END UNIT;",
    );
}

#[test]
fn program_closer_and_anonymous_expression_terminator_ownership_are_distinct() {
    parse_ok(
        "program Demo; type Handler = function(): integer;
        function F(): integer; begin begin return 1; end; end function;
        begin var Callback: Handler := function(): integer
            function Nested(): integer; begin return 2; end function;
            begin return Nested(); end function;
        var Value: integer := Apply(procedure() begin return; end procedure);
        end.",
    );
}

#[test]
fn legacy_declaration_endings_report_the_full_expected_closer() {
    for (source, expected) in [
        (
            "program T; function F(): integer; begin return 1; end; begin end.",
            "end function;",
        ),
        (
            "program T; procedure P(); begin return; end; begin end.",
            "end procedure;",
        ),
        ("program T; type R = record end; begin end.", "end record;"),
        ("program T; type E = enum A; end; begin end.", "end enum;"),
        ("unit Demo;", "end unit;"),
    ] {
        let (_, errors) = parse_compilation_unit(source);
        let error = errors
            .iter()
            .filter_map(|error| error.as_parser_error())
            .find(|error| error.expected.as_deref() == Some(expected))
            .unwrap_or_else(|| panic!("missing {expected}: {errors:#?}"));
        assert_eq!(error.code, PARSE_EXPECTED_TOKEN);
        assert!(error.help.as_deref().unwrap().contains(expected));
    }
}

#[test]
fn mismatched_closer_is_consumed_before_the_next_declaration() {
    let (unit, errors) = parse_compilation_unit(
        "unit Demo;
        function Wrong(): integer; begin return 1; end procedure;
        procedure Next(); begin return; end procedure;
        end unit;",
    );
    let CompilationUnit::Unit(unit) = unit else {
        panic!("expected unit")
    };
    assert_eq!(unit.declarations.len(), 2);
    let errors: Vec<_> = errors
        .iter()
        .filter_map(|error| error.as_parser_error())
        .collect();
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(errors[0].expected.as_deref(), Some("end function;"));
    assert_eq!(errors[0].found.as_deref(), Some("end procedure;"));
}

#[test]
fn missing_method_closer_preserves_the_enclosing_record_and_unit() {
    let (unit, errors) = parse_compilation_unit(
        "unit Demo; type R = record
        function F(self: R): integer; begin return 1;
        end record;
        procedure Next(); begin return; end procedure;
        end unit;",
    );
    let CompilationUnit::Unit(unit) = unit else {
        panic!("expected unit")
    };
    assert_eq!(unit.declarations.len(), 2, "{unit:#?}");
    assert!(
        matches!(&unit.declarations[1], Decl::Procedure(procedure) if procedure.name == "Next")
    );
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert!(
        errors[0]
            .as_parser_error()
            .unwrap()
            .message
            .contains("found `end record;`")
    );
}

#[test]
fn missing_routine_closer_preserves_unit_and_program_endings() {
    for source in [
        "unit Demo; function F(): integer; begin return 1; end unit;",
        "program Demo; function F(): integer; begin return 1; end.",
    ] {
        let (unit, errors) = parse_compilation_unit(source);
        assert_eq!(
            errors.len(),
            if source.starts_with("unit") { 1 } else { 2 },
            "{errors:#?}"
        );
        assert_eq!(
            errors[0].as_parser_error().unwrap().expected.as_deref(),
            Some("end function;")
        );
        let span = match unit {
            CompilationUnit::Unit(unit) => unit.span,
            CompilationUnit::Program(program) => program.span,
        };
        assert_eq!(span.length, source.len());
    }
}

#[test]
fn named_declaration_endings_require_one_terminator() {
    for source in [
        "program T; function F(): integer; begin return 1; end function begin end.",
        "program T; type R = record end record begin end.",
        "unit Demo; end unit",
    ] {
        let (_, errors) = parse_compilation_unit(source);
        assert!(
            errors
                .iter()
                .filter_map(|error| error.as_parser_error())
                .any(|error| error.expected.as_deref() == Some(";")),
            "{errors:#?}"
        );
    }
}

#[test]
fn empty_unit_header_and_final_span_are_preserved() {
    let source = "unit Demo; end unit;";
    let (unit, errors) = parse_compilation_unit(&format!("{source} trailing"));
    let CompilationUnit::Unit(unit) = unit else {
        panic!("expected unit")
    };
    assert_eq!(unit.span.length, source.len());
    assert!(unit.declarations.is_empty());
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert!(
        errors[0]
            .as_parser_error()
            .unwrap()
            .message
            .contains("after unit terminator")
    );
}

#[test]
fn missing_type_ending_preserves_the_unit_ending_without_a_cascade() {
    for declaration in ["type R = record X: integer;", "type E = enum A;"] {
        let source = format!("unit Demo; {declaration} end unit;");
        let (unit, errors) = parse_compilation_unit(&source);
        let CompilationUnit::Unit(unit) = unit else {
            panic!("expected unit")
        };
        assert_eq!(unit.span.length, source.len());
        assert_eq!(unit.declarations.len(), 1);
        assert_eq!(errors.len(), 1, "{errors:#?}");
    }
}

#[test]
fn missing_routine_ending_keeps_the_next_declaration() {
    let (unit, errors) = parse_compilation_unit(
        "unit Demo;
        function First(): integer; begin return 1;
        procedure Next(); begin return; end procedure; end unit;",
    );
    let CompilationUnit::Unit(unit) = unit else {
        panic!("expected unit")
    };
    assert_eq!(unit.declarations.len(), 2);
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(
        errors[0].as_parser_error().unwrap().expected.as_deref(),
        Some("end function;")
    );
}

#[test]
fn mismatched_unknown_ending_recovers_at_the_next_declaration() {
    let (unit, errors) = parse_compilation_unit(
        "unit Demo;
        procedure Wrong(); begin return; end mystery;
        procedure Next(); begin return; end procedure; end unit;",
    );
    let CompilationUnit::Unit(unit) = unit else {
        panic!("expected unit")
    };
    assert_eq!(unit.declarations.len(), 2);
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(
        errors[0].as_parser_error().unwrap().found.as_deref(),
        Some("end mystery;")
    );
}

#[test]
fn missing_scoped_block_ending_does_not_consume_the_routine_ending() {
    let (unit, errors) = parse_compilation_unit(
        "unit Demo;
        procedure Wrong(); begin begin return; end procedure;
        procedure Next(); begin return; end procedure; end unit;",
    );
    let CompilationUnit::Unit(unit) = unit else {
        panic!("expected unit")
    };
    assert_eq!(unit.declarations.len(), 2);
    assert!(
        errors.iter().any(|error| error
            .as_parser_error()
            .is_some_and(|error| error.message.contains("scoped")
                || error.message.contains("Expected `end;`")))
    );
    assert!(!errors.iter().any(|error| {
        error
            .as_parser_error()
            .is_some_and(|error| error.expected.as_deref() == Some("end procedure;"))
    }));
}
