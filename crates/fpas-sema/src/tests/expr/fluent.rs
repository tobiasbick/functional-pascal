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
fn explicit_alias_selects_imported_source_routines() {
    let interfaces = [
        interface_for(
            r#"unit Demo.First; public function Choose(Value: integer; Text: string): integer; begin return 1; end function;
end unit;
"#,
        ),
        interface_for(
            r#"unit Demo.Second; public function Choose(Value: string; Other: integer): integer; begin return 2; end function;
end unit;
"#,
        ),
    ];
    let errors = imported_call_errors(
        r#"program T;  uses Demo.First as First; uses Demo.Second as Second; begin var N: integer := First.Choose(1, 'x'); end program;"#,
        &interfaces,
    );
    assert!(errors.is_empty(), "{errors:#?}");
}

#[test]
fn receiver_syntax_does_not_open_imported_routines() {
    let interfaces = [
        interface_for(
            r#"unit Demo.First; public function Choose(Value: integer; Text: string): integer; begin return 1; end function;
end unit;
"#,
        ),
        interface_for(
            r#"unit Demo.Second; public function Choose(Value: integer; Other: integer): integer; begin return 2; end function;
end unit;
"#,
        ),
    ];
    let errors = imported_call_errors(
        r#"program T;  uses Demo.First as First; uses Demo.Second as Second; begin var N: integer := (1).Choose('x'); end program;"#,
        &interfaces,
    );
    assert!(
        errors.iter().any(|error| error.code == SEMA_UNKNOWN_NAME
            && error.help.as_deref().is_some_and(
                |help| help.contains("First.Choose") && help.contains("Second.Choose")
            )),
        "{errors:#?}"
    );
}

#[test]
fn imported_intrinsics_select_by_receiver_type() {
    check_ok(
        r#"program T;  uses Std.Arrays as Arrays; uses Std.Dictionaries as Dictionaries; uses Std.Str as Str; begin var A: array of integer := [1]; var D: dict of string to integer := ['a': 2]; var N: integer := Arrays.Length(A) + Dictionaries.Length(D) + Str.Length(('ab')); end program;"#,
    );
}

#[test]
fn lexical_function_shadows_imports_even_when_incompatible() {
    let errors = check_errors(
        r#"program T;  uses Std.Arrays as Arrays; function Length(S: string): integer; begin return 0; end function; begin var A: array of integer := [1]; var N: integer := A.Length(); end program;"#,
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
        r#"program T;  uses Std.Arrays as Arrays; begin var A: array of integer := [1]; var Length: integer := 7; var N: integer := A.Length(); end program;"#,
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
        r#"program T;  uses Std.Arrays as Arrays; function Map(A: array of integer; X: integer): integer; begin return X; end function; function Double(X: integer): integer; begin return X * 2; end function; begin var A: array of integer := [1]; var B: array of integer := A.Map(Double); end program;"#,
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
        r#"program T;  type Item = record Value: integer; end record; function Value(X: Item): integer; begin return 2; end function; begin var I: Item := record Value := 1; end record; var N: integer := I.Value(); end program;"#,
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
        r#"program T; function Identity<T: Numeric>(X: T): T; begin return X; end function; begin var N: integer := (2).Identity(); end program;"#,
    );
    let errors = check_errors(
        r#"program T; function Identity<T: Numeric>(X: T): T; begin return X; end function; begin var S: string := ('x').Identity(); end program;"#,
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
        r#"program T;  uses Std.Arrays as Arrays; begin mutable var A: array of integer := [1]; Arrays.Push((A), 2); end program;"#,
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
        r#"program T; procedure Consume(X: integer); begin null; end procedure; begin (2).Consume(); end program;"#,
    );
    let errors = check_errors(
        r#"program T; procedure Consume(X: integer); begin null; end procedure; begin var N: integer := (2).Consume(); end program;"#,
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
        r#"program T;  uses Std.Arrays as Arrays; uses Std.Dictionaries as Dictionaries; begin var N: integer := Unknown.Length(); end program;"#,
    );
    assert_eq!(errors.len(), 1, "{errors:#?}");
}
