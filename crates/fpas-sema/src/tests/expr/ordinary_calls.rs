//! Ordinary routines receive every argument explicitly; dot calls select fields.

use super::{check_errors, check_ok};
use fpas_diagnostics::codes::{SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME};

#[test]
fn ordinary_routines_are_not_selected_through_a_value() {
    for call in [
        "Number.Add(2)",
        "(Number).Add(2)",
        "Make().Add(2)",
        "Values[0].Add(2)",
        "RecordValue.Add(2)",
        "(RecordValue).Add(2)",
        "Values.Length()",
        "(Values).Length()",
        "Number.Identity()",
        "Number.Callback(2)",
    ] {
        let errors = check_errors(&format!(
            "program T; uses Std.Arrays as Arrays;
            type Box = record Value: integer; end record;
            function Add(Value: integer; Other: integer): integer;
            begin return Value + Other; end function;
            function Make(): integer; begin return 1; end function;
            function Length(Value: array of (integer)): integer;
            begin return Arrays.Length(Value); end function;
            function Identity of (T: Numeric)(Value: T): T;
            begin return Value; end function;
            begin
              const Number := 1; const Values := [1]; const RecordValue := Box(Value := 1);
              const Callback := Add;
              discard {call};
            end program;"
        ));
        assert!(
            errors
                .iter()
                .any(|error| matches!(error.code, SEMA_TYPE_MISMATCH | SEMA_UNKNOWN_NAME)),
            "{call}: {errors:#?}"
        );
    }
}

#[test]
fn procedures_and_tasks_do_not_insert_receiver_arguments() {
    for statement in [
        "Value.Consume();",
        "(Value).Consume();",
        "go Value.Consume();",
        "go (Value).Consume();",
    ] {
        let errors = check_errors(&format!(
            "program T; procedure Consume(Value: integer); begin null; end procedure;
            begin const Value := 1; {statement} end program;"
        ));
        assert!(
            errors
                .iter()
                .any(|error| matches!(error.code, SEMA_TYPE_MISMATCH | SEMA_UNKNOWN_NAME)),
            "{statement}: {errors:#?}"
        );
    }
}

#[test]
fn callable_fields_use_only_their_written_arguments() {
    check_ok(
        "program T;
        type Box = record Apply: function(Value: integer): integer; end record;
        function Apply(Receiver: Box; Value: integer): integer; begin return Value; end function;
        function Increment(Value: integer): integer; begin return Value + 1; end function;
        function Make(): Box; begin return Box(Apply := Increment); end function;
        begin const Value := Make(); discard Value.Apply(1); discard (Value).Apply(2); discard Make().Apply(3); end program;"
    );
    let errors = check_errors(
        "program T; type Box = record Apply: integer; end record;
        function Apply(Receiver: Box): integer; begin return 2; end function;
        begin const Value := Box(Apply := 1); discard Value.Apply(); end program;",
    );
    assert!(errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH && error.message.contains("not callable")), "{errors:#?}");
}

#[test]
fn explicit_calls_keep_generic_and_imported_signature_checks() {
    check_ok(
        "program T; uses Std.Arrays as Arrays; uses Std.Dictionaries as Dictionaries; uses Std.Str as Str;
        function Identity of (T: Numeric)(Value: T): T; begin return Value; end function;
        begin const Values := [1]; const Map := ['a': 2];
          discard Arrays.Length(Values) + Dictionaries.Length(Map) + Str.Length('ab');
          discard Identity(2);
        end program;"
    );
    let errors = check_errors(
        "program T; function Identity of (T: Numeric)(Value: T): T; begin return Value; end function; begin discard Identity('x'); end program;",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_CONSTRAINT_VIOLATION),
        "{errors:#?}"
    );
}

#[test]
fn unknown_field_call_root_does_not_cascade() {
    let errors = check_errors("program T; begin discard Unknown.Length(); end program;");
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(errors[0].code, SEMA_UNKNOWN_NAME);
}
