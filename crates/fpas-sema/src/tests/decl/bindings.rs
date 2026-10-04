//! Binding initialization, inference, static contexts and lexical capture contracts.

use super::{check_errors, check_ok};
use fpas_diagnostics::codes::{
    SEMA_IMMUTABLE_ASSIGNMENT, SEMA_NON_CONSTANT_EXPRESSION, SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME,
};

#[test]
fn computed_consts_and_mutable_vars_use_explicit_or_local_inferred_types() {
    check_ok(
        "program T; function Compute(): integer; begin return 42; end function;
        const Global: integer := Compute(); var State: integer := Global;
        begin const Answer := Compute(); var Copy := Answer; Copy := Copy + 1;
          const Snapshot := State; State := 0; const RealValue: real := 1.0;
        end program;",
    );
}

#[test]
fn local_inference_composes_constructors_decisions_and_callable_values() {
    check_ok("program T; uses Std.Tasks as Tasks; type Pair of (T) = record Value: T; end record;
        type Choice of (T) = enum Present(Value: T); Missing; end enum;
        function Work(): integer; begin return 1; end function;
        begin const RecordValue := Pair(Value := 1); const Variant := Choice.Present('text');
          const Values := [1, 2]; const Mapping := ['one': RecordValue];
          const Number := if true then 1.0 else 2.0 end if;
          const Selected := case Variant of when Choice.Present(const Value): Value; when Choice.Missing: ''; end case;
          const Callback := function(Value: integer): integer begin return Value + 1; end function;
          const Action := procedure(Value: integer) begin discard Callback(Value); end procedure;
          const Job := go Work(); const Finished := Tasks.Wait(Job);
        end program;");
}

#[test]
fn missing_initializer_context_requests_an_annotation_without_using_later_statements() {
    for source in [
        "program T; begin const Values := []; end program;",
        "program T; begin var Values := [:]; Values['one'] := 1; end program;",
        "program T; begin const Empty := Option.None; end program;",
        "program T; type Choice of (T) = enum Present(Value: T); Missing; end enum; begin const Empty := Choice.Missing; end program;",
        "program T; begin const Value := if true then 1 else 'text' end if; end program;",
        "program T; begin const Value: integer := 'text'; end program;",
        "program T; function Identity of (T)(Value: T): T; begin return Value; end function; begin const Action := Identity; end program;",
    ] {
        let errors = check_errors(source);
        assert!(
            errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
            "{source}: {errors:#?}"
        );
    }
    let errors =
        check_errors("program T; begin const First := Later; const Later := 1; end program;");
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(errors[0].code, SEMA_UNKNOWN_NAME);
}

#[test]
fn consts_reject_writes_and_procedure_calls_but_accept_procedure_values() {
    for source in [
        "program T; begin const Value := 1; Value := 2; end program;",
        "program T; type Data = record Values: array of (integer); end record; begin const Value := Data(Values := [1]); Value.Values[0] := 2; end program;",
    ] {
        assert!(
            check_errors(source)
                .iter()
                .any(|error| error.code == SEMA_IMMUTABLE_ASSIGNMENT)
        );
    }
    check_errors(
        "program T; procedure Act(); begin null; end procedure; begin const Value := Act(); end program;",
    );
    check_ok(
        "program T; procedure Act(); begin null; end procedure; begin const Value := Act; Value(); end program;",
    );
}

#[test]
fn computed_const_bindings_remain_non_static_in_patterns() {
    for definition in [
        "var Lower: integer := 1;",
        "const Lower: integer := Next();",
        "var Seed: integer := 1; const Lower: integer := Seed;",
    ] {
        let errors = check_errors(&format!(
            "program T; function Next(): integer; begin return 1; end function; {definition} begin case 1 of when Lower: null; else null; end case; end program;"
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_NON_CONSTANT_EXPRESSION),
            "{errors:#?}"
        );
    }
}

#[test]
fn static_local_constants_follow_lexical_shadowing_and_scope_exit() {
    check_ok(
        "program T; type Choice = enum Present(Value: boolean); end enum;
        const Flag: boolean := true;
        begin const Item := Choice.Present(true);
          begin const Flag := false;
            case Item of when Choice.Present(Flag): null; when Choice.Present(true): null; end case;
          end;
          case Item of when Choice.Present(Flag): null; when Choice.Present(false): null; end case;
        end program;",
    );
}

#[test]
fn named_nested_routines_capture_inferred_locals_without_signature_inference() {
    check_ok(
        "program T; function Make(Base: integer): function(): integer;
        function GetValue(): integer; begin return Offset; end function;
        function Compute(): integer; begin return Base + 1; end function;
        begin const Offset := Compute(); return GetValue; end function;
        begin const GetValue := Make(41); discard GetValue(); end program;",
    );
}

#[test]
fn computed_const_closures_preserve_task_bound_capture_information() {
    let errors = check_errors(
        "program T; begin var Count := 0;
        const GetValue := function(): integer begin Count := Count + 1; return Count; end function;
        const Job := go GetValue(); end program;",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("task-bound")),
        "{errors:#?}"
    );
}

#[test]
fn duplicate_bindings_are_case_insensitive() {
    check_errors("program T; begin const Value := 1; var value := 2; end program;");
    check_ok("program T; begin const Value := 1; const Copy := VALUE; end program;");
}
