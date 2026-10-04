//! Ordinary factories and functions compose with nominal record aliases.

use super::{check_errors, check_ok};

#[test]
fn ordinary_factory_and_record_arguments_preserve_aliases_and_name_casing() {
    check_ok(
        "program T;
      type Point = record X: integer; Y: integer; end record;
      type Alias = Point;
      function PointCreate(X: integer; Y: integer): Point;
      begin return Point(X := X, Y := Y); end function;
      function PointSum(Receiver: Alias): integer;
      begin return Receiver.X + Receiver.Y; end function;
      function Identity of (T)(Value: T): T; begin return Value; end function;
      procedure Verify(Value: Alias); begin discard PointSum(Value); end procedure;
      begin
        const P: Alias := pointcreate(3, 4);
        const Factory: function(X: integer; Y: integer): Alias := PointCreate;
        const Check: procedure(Value: Point) := Verify;
        Check(Identity(P)); discard pointsum(Factory(5, 6));
      end program;",
    );
}

#[test]
fn record_factory_overloads_and_case_only_duplicates_are_rejected() {
    for signature in [
        "pointcreate(X: integer)",
        "PointCreate(X: integer; Y: integer)",
    ] {
        let errors = check_errors(&format!(
            "program T;
          type Point = record X: integer; end record;
          function PointCreate(X: integer): Point; begin return Point(X := X); end function;
          function {signature}: Point; begin return Point(X := X); end function;
          begin null; end program;"
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION),
            "{errors:#?}"
        );
    }
}

#[test]
fn record_procedure_call_cannot_initialize_a_binding() {
    let errors = check_errors(
        "program T;
      type Counter = record Value: integer; end record;
      procedure Reset(var Receiver: Counter); begin Receiver.Value := 0; end procedure;
      begin var C := Counter(Value := 1); const Value := Reset(var C); end program;",
    );
    assert!(
        errors.iter().any(
            |error| error.code == fpas_diagnostics::codes::SEMA_TYPE_MISMATCH
                && error.message.contains("does not return a value")
        ),
        "{errors:#?}"
    );
}
