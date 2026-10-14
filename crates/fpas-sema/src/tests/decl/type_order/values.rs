use super::{check_errors, check_ok};
use fpas_diagnostics::codes::{SEMA_UNINITIALIZED_DEPENDENCY, SEMA_UNQUALIFIED_NAME};

#[test]
fn defaults_and_method_bodies_keep_preceding_values_visible() {
    check_ok(
        "program T; const Limit: integer := 10; var Count: integer := 0;
      type Box = record Value: integer := Limit;
        function GetValue(Self: Box): integer; begin return Self.Value + Count; end function;
      end record;
      begin const Value: Box := Box( ); discard Value.GetValue(); end.",
    );
}

#[test]
fn later_values_are_available_in_deferred_bodies_but_not_earlier_initializers() {
    for declarations in [
        "type Box = record Value: integer := Later; end record; const Later: integer := 10;",
        "type Box = record function GetValue(Self: Box): integer; begin return Later; end function; end record; const Later: integer := 10;",
        "function GetValue(): integer; begin return Later; end function; const Later: integer := 10;",
    ] {
        check_ok(&format!("program T; {declarations} begin end."));
    }
    for declarations in [
        "const Earlier: integer := Later; const Later: integer := 10;",
        "const Earlier: integer := GetValue(); function GetValue(): integer; begin return Later; end function; const Later: integer := 10;",
    ] {
        let errors = check_errors(&format!("program T; {declarations} begin end."));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_UNINITIALIZED_DEPENDENCY),
            "{errors:?}"
        );
    }
}

#[test]
fn later_enum_variants_require_qualification_in_earlier_initializers() {
    let errors = check_errors(
        "program T; const Initial: A := Ready;
      type A = enum Ready; end enum; type B = enum Ready; end enum;
      begin end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_UNQUALIFIED_NAME)
    );
    check_ok(
        "program T; const Initial: A := A.Ready;
      type A = enum Ready; end enum; type B = enum Ready; end enum;
      begin end.",
    );
}

#[test]
fn ordinary_values_keep_their_names_beside_qualified_enum_variants() {
    check_ok(
        "program T; const Ready: integer := 10;\n      type State = enum Ready; end enum;\n      begin const Value: State := State.Ready; const Number: integer := Ready; discard Value; discard Number; end.",
    );
}

#[test]
fn field_defaults_can_call_later_named_routines() {
    check_ok(
        "program T;
      type Box = record Value: integer := Later(); end record;
      function Later(): integer; begin return 10; end function;
      begin end.",
    );
}

#[test]
fn generic_shadowing_keeps_record_identity_for_ordinary_discard() {
    check_ok(
        "program T;
      type A = record Next: Option of B;
        function Identity<B>(Self: A; Value: B): B;
        begin discard Self; return Value; end function;
      end record;
      type B = record Job: task of integer; end record;
      begin end.",
    );
}
