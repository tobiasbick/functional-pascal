use super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_UNKNOWN_NAME;
use fpas_parser::{CompilationUnit, parse_compilation_unit};

fn interface_for(source: &str) -> fpas_unit::interface::UnitInterface {
    let (parsed, errors) = parse_compilation_unit(source);
    assert!(errors.is_empty(), "{errors:#?}");
    let CompilationUnit::Unit(unit) = parsed else {
        panic!("fixture must be a unit");
    };
    let analysis = crate::analyze_unit(&unit, &[]).expect("unit analysis");
    assert!(
        analysis.metadata.errors.is_empty(),
        "{:#?}",
        analysis.metadata.errors
    );
    analysis.interface.expect("unit interface")
}

fn imported_call_errors(
    source: &str,
    interfaces: &[fpas_unit::interface::UnitInterface],
) -> Vec<crate::SemaError> {
    let (parsed, errors) = parse_compilation_unit(source);
    assert!(errors.is_empty(), "{errors:#?}");
    let CompilationUnit::Program(program) = parsed else {
        panic!("fixture must be a program");
    };
    crate::analyze_program_with_interfaces(&program, interfaces)
        .expect("program analysis")
        .errors
}

#[test]
fn imported_source_routines_use_qualified_ordinary_calls() {
    let interfaces = [
        interface_for(
            "unit Demo.First; public function Choose(Value: integer; Text: string): integer; begin return 1; end function;\nend unit;",
        ),
        interface_for(
            "unit Demo.Second; public function Choose(Value: string; Other: integer): integer; begin return 2; end function;\nend unit;",
        ),
    ];
    let errors = imported_call_errors(
        "program T; uses Demo.First, Demo.Second; begin const N: integer := Demo.First.Choose(1, 'x'); end.",
        &interfaces,
    );
    assert!(errors.is_empty(), "{errors:#?}");
}

#[test]
fn trailing_arguments_do_not_break_imported_receiver_tie() {
    let interfaces = [
        interface_for(
            "unit Demo.First; public function Choose(Value: integer; Text: string): integer; begin return 1; end function;\nend unit;",
        ),
        interface_for(
            "unit Demo.Second; public function Choose(Value: integer; Other: integer): integer; begin return 2; end function;\nend unit;",
        ),
    ];
    let errors = imported_call_errors(
        "program T; uses Demo.First, Demo.Second; begin const N: integer := (1).Choose('x'); end.",
        &interfaces,
    );
    assert!(
        errors.iter().any(|error| error.code == SEMA_UNKNOWN_NAME
            && error
                .help
                .as_deref()
                .is_some_and(|help| help.contains("free functions ordinarily"))),
        "{errors:#?}"
    );
}

#[test]
fn imported_intrinsics_select_by_receiver_type() {
    check_ok(
        "program T;  begin const A: array of integer := [1]; const D: dict of string to integer := ['a': 2]; const N: integer := A.Length() + D.Length() + ('ab').Length(); end.",
    );
}

#[test]
fn native_length_ignores_same_named_free_function() {
    check_ok(
        "program T;  function Length(S: string): integer; begin return 0; end function; begin const A: array of integer := [1]; const N: integer := A.Length(); end.",
    );
}

#[test]
fn native_length_ignores_same_named_local_value() {
    check_ok(
        "program T;  begin const A: array of integer := [1]; const Length: integer := 7; const N: integer := A.Length(); end.",
    );
}

#[test]
fn native_map_ignores_same_named_free_function() {
    check_ok(
        "program T;  function Map(A: array of integer; X: integer): integer; begin return X; end function; function Double(X: integer): integer; begin return X * 2; end function; begin const A: array of integer := [1]; const B: array of integer := A.Map(Double); end.",
    );
}

#[test]
fn record_field_blocks_free_call_fallback() {
    let errors = check_errors(
        "program T; type Item = record Value: integer; end record; \
         function Value(X: Item): integer; begin return 2; end function; \
         begin const I: Item := Item( Value := 1 ); \
         const N: integer := I.Value(); end.",
    );
    assert!(
        errors.iter().any(|error| error
            .message
            .contains("Record member `Value` is not callable")),
        "{errors:#?}"
    );
}

#[test]
fn constrained_generic_receiver_matches_only_valid_type() {
    check_ok(
        "program T; function Identity<T: Numeric>(X: T): T; begin return X; end function; \
         begin const N: integer := Identity(2); end.",
    );
    let errors = check_errors(
        "program T; function Identity<T: Numeric>(X: T): T; begin return X; end function; \
         begin const S: string := ('x').Identity(); end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("has no dot operation")),
        "{errors:#?}"
    );
}

#[test]
fn array_mutation_rejects_parenthesized_receiver() {
    let errors =
        check_errors("program T;  begin var A: array of integer := [1]; (A).Push(2); end.");
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_INVALID_VAR_ARGUMENT),
        "{errors:#?}"
    );
}

#[test]
fn procedure_must_end_a_statement_chain() {
    check_ok(
        "program T; procedure Consume(X: integer); begin end procedure; \
         begin Consume(2); end.",
    );
    let errors = check_errors(
        "program T; procedure Consume(X: integer); begin end procedure; \
         begin const N: integer := [2].ForEach(procedure(X: integer) begin end procedure); end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("does not return a value")),
        "{errors:#?}"
    );
}

#[test]
fn erroneous_receiver_does_not_cascade() {
    let errors = check_errors("program T;  begin const N: integer := Unknown.Length(); end.");
    assert_eq!(errors.len(), 1, "{errors:#?}");
}

#[test]
fn imported_same_named_free_functions_coexist_with_fixed_native_targets() {
    let interfaces = [interface_for(
        "unit Demo.Helpers; public function Length(Value: string): integer; begin return 99; end function; end unit;",
    )];
    let errors = imported_call_errors(
        "program T; uses Demo.Helpers; begin const Own: integer := Length('x'); const Native: integer := 'x'.Length(); end.",
        &interfaces,
    );
    assert!(errors.is_empty(), "{errors:#?}");
    let errors = imported_call_errors(
        "program T; uses Demo.Helpers; begin const N: integer := 'x'.Length('extra'); end.",
        &interfaces,
    );
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(
        errors[0].code,
        fpas_diagnostics::codes::SEMA_WRONG_ARGUMENT_COUNT
    );
}
