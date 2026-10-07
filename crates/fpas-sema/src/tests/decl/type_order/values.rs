use super::{check_errors, check_ok};
use fpas_diagnostics::codes::{SEMA_AMBIGUOUS_IMPORTED_NAME, SEMA_UNKNOWN_NAME};

#[test]
fn defaults_and_method_bodies_keep_preceding_values_visible() {
    check_ok(
        "program T; const Limit: integer := 10; var Count: integer := 0;
      type Box = record Value: integer := Limit;
        function GetValue(Self: Box): integer; begin return Self.Value + Count; end function;
      end record;
      begin const Value: Box := record end; discard Value.GetValue(); end.",
    );
}

#[test]
fn collection_does_not_expose_later_values_in_defaults_or_bodies() {
    for declarations in [
        "type Box = record Value: integer := Later; end record; const Later: integer := 10;",
        "type Box = record function GetValue(Self: Box): integer; begin return Later; end function; end record; const Later: integer := 10;",
        "const Earlier: integer := Later; const Later: integer := 10;",
        "const Earlier: integer := Later; const Later: integer := 10;",
        "function GetValue(): integer; begin return Later; end function; const Later: integer := 10;",
    ] {
        let errors = check_errors(&format!("program T; {declarations} begin end."));
        assert!(
            errors.iter().any(|error| error.code == SEMA_UNKNOWN_NAME),
            "{errors:?}"
        );
    }
}

#[test]
fn later_enum_variants_still_require_qualification_when_ambiguous() {
    let errors = check_errors(
        "program T; const Initial: A := Ready;
      type A = enum Ready; end enum; type B = enum Ready; end enum;
      begin end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_AMBIGUOUS_IMPORTED_NAME)
    );
    check_ok(
        "program T; const Initial: A := A.Ready;
      type A = enum Ready; end enum; type B = enum Ready; end enum;
      begin end.",
    );
}

#[test]
fn preceding_values_keep_priority_over_optional_enum_short_names() {
    check_ok(
        "program T; const Ready: integer := 10;
      type State = enum Ready; end enum;
      begin const Value: State := State.Ready; discard Value; discard Ready; end.",
    );
}

#[test]
fn types_do_not_make_free_routines_visible_before_their_declarations() {
    let errors = check_errors(
        "program T;
      type Box = record Value: integer := Later(); end record;
      function Later(): integer; begin return 10; end function;
      begin end.",
    );
    assert!(errors.iter().any(|error| error.code == SEMA_UNKNOWN_NAME));
}

#[test]
fn generic_parameters_cannot_hide_task_fields_from_discard_analysis() {
    let errors = check_errors(
        "program T;
      type A = record Next: Option of B;
        function Identity<B>(Self: A; Value: B): B;
        begin discard Self; return Value; end function;
      end record;
      type B = record Job: task of integer; end record;
      begin end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_UNSAFE_DISCARD),
        "{errors:?}"
    );
}
