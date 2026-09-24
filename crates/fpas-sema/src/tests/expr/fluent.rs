use super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_AMBIGUOUS_IMPORTED_NAME;
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
fn imported_source_routines_filter_only_by_receiver() {
    let interfaces = [
        interface_for(
            "unit Demo.First; public function Choose(Value: integer; Text: string): integer; begin return 1 end;",
        ),
        interface_for(
            "unit Demo.Second; public function Choose(Value: string; Other: integer): integer; begin return 2 end;",
        ),
    ];
    let errors = imported_call_errors(
        "program T; uses Demo.First, Demo.Second; begin var N: integer := (1).Choose('x') end.",
        &interfaces,
    );
    assert!(errors.is_empty(), "{errors:#?}");
}

#[test]
fn trailing_arguments_do_not_break_imported_receiver_tie() {
    let interfaces = [
        interface_for(
            "unit Demo.First; public function Choose(Value: integer; Text: string): integer; begin return 1 end;",
        ),
        interface_for(
            "unit Demo.Second; public function Choose(Value: integer; Other: integer): integer; begin return 2 end;",
        ),
    ];
    let errors = imported_call_errors(
        "program T; uses Demo.First, Demo.Second; begin var N: integer := (1).Choose('x') end.",
        &interfaces,
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_AMBIGUOUS_IMPORTED_NAME
                && error
                    .help
                    .as_deref()
                    .is_some_and(|help| help.contains("Demo.First.Choose")
                        && help.contains("Demo.Second.Choose"))),
        "{errors:#?}"
    );
}

#[test]
fn imported_intrinsics_select_by_receiver_type() {
    check_ok(
        "program T; uses Std.Arrays, Std.Dictionaries, Std.Str; \
         begin var A: array of integer := [1]; \
         var D: dict of string to integer := ['a': 2]; \
         var N: integer := A.Length() + D.Length() + ('ab').Length() end.",
    );
}

#[test]
fn lexical_function_shadows_imports_even_when_incompatible() {
    let errors = check_errors(
        "program T; uses Std.Arrays; \
         function Length(S: string): integer; begin return 0 end; \
         begin var A: array of integer := [1]; var N: integer := A.Length() end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("cannot be called with receiver")),
        "{errors:#?}"
    );
}

#[test]
fn noncallable_local_shadows_matching_import() {
    let errors = check_errors(
        "program T; uses Std.Arrays; \
         begin var A: array of integer := [1]; \
         var Length: integer := 7; var N: integer := A.Length() end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("cannot be called with receiver")),
        "{errors:#?}"
    );
}

#[test]
fn trailing_arguments_do_not_reselect_a_shadowed_callable() {
    let errors = check_errors(
        "program T; uses Std.Arrays; \
         function Map(A: array of integer; X: integer): integer; begin return X end; \
         function Double(X: integer): integer; begin return X * 2 end; \
         begin var A: array of integer := [1]; \
         var B: array of integer := A.Map(Double) end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("argument")),
        "{errors:#?}"
    );
}

#[test]
fn record_field_blocks_free_call_fallback() {
    let errors = check_errors(
        "program T; type Item = record Value: integer; end; \
         function Value(X: Item): integer; begin return 2 end; \
         begin var I: Item := record Value := 1; end; \
         var N: integer := I.Value() end.",
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
        "program T; function Identity<T: Numeric>(X: T): T; begin return X end; \
         begin var N: integer := (2).Identity() end.",
    );
    let errors = check_errors(
        "program T; function Identity<T: Numeric>(X: T): T; begin return X end; \
         begin var S: string := ('x').Identity() end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("cannot be called with receiver")),
        "{errors:#?}"
    );
}

#[test]
fn array_mutation_rejects_parenthesized_receiver() {
    let errors = check_errors(
        "program T; uses Std.Arrays; \
         begin mutable var A: array of integer := [1]; (A).Push(2) end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("simple mutable array variable")),
        "{errors:#?}"
    );
}

#[test]
fn procedure_must_end_a_statement_chain() {
    check_ok(
        "program T; procedure Consume(X: integer); begin end; \
         begin (2).Consume() end.",
    );
    let errors = check_errors(
        "program T; procedure Consume(X: integer); begin end; \
         begin var N: integer := (2).Consume() end.",
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
    let errors = check_errors(
        "program T; uses Std.Arrays, Std.Dictionaries; \
         begin var N: integer := Unknown.Length() end.",
    );
    assert_eq!(errors.len(), 1, "{errors:#?}");
}
