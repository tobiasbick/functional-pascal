use super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::{SEMA_DISCARD_REQUIRES_VALUE, SEMA_UNSAFE_DISCARD};

#[test]
fn discard_accepts_ordinary_values_and_explicitly_ignored_calls() {
    check_ok(
        "program T;
      function Value(): result of integer, string; begin return Ok(1); end function;
      begin discard 42; discard 'hello'; discard Value(); discard Some(1);
      discard [1, 2]; discard ['key': 1]; discard Value(); end.",
    );
}

#[test]
fn discard_rejects_procedure_calls() {
    let errors = check_errors(
        "program T; procedure Work(); begin end procedure; begin discard Work(); end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_DISCARD_REQUIRES_VALUE)
    );
}

#[test]
fn task_aggregate_types_are_checked_even_when_empty_or_inactive() {
    for (declarations, ty, initializer) in [
        ("", "array of task of integer", "[]"),
        ("", "Option of task of integer", "None"),
        ("", "result of integer, task of integer", "Ok(1)"),
        ("", "dict of string to array of task of integer", "[:]"),
        (
            "type Box = record Job: Option of task of integer; end record;",
            "Box",
            "Box( Job := None )",
        ),
        (
            "type Choice = enum Empty; Active(Job: task of integer); end enum;",
            "Choice",
            "Choice.Empty",
        ),
        (
            "type Alias = array of Option of task of integer;",
            "Alias",
            "[]",
        ),
    ] {
        let source = format!(
            "program T; {declarations} begin const Value: {ty} := {initializer}; discard Value; end."
        );
        let errors = check_errors(&source);
        assert!(
            errors.iter().any(|error| error.code == SEMA_UNSAFE_DISCARD),
            "{source}: {errors:?}"
        );
    }
}

#[test]
fn recursive_record_payloads_use_the_completed_type() {
    check_ok(
        "program T; type Node = record Next: Option of Node; end record;
      begin const Root: Node := Node( Next := None ); discard Root; discard Root.Next; end.",
    );
    let errors = check_errors(
        "program T;
      type Node = record Next: Option of Node; Job: Option of task of integer; end record;
      begin const Root: Node := Node( Next := None, Job := None );
      discard Root.Next; end.",
    );
    assert!(errors.iter().any(|error| error.code == SEMA_UNSAFE_DISCARD));
    check_ok(
        "program T; function Ignore<T: Numeric>(Values: array of T): integer;
      begin discard Values; return 0; end function;
      begin discard Ignore([42]); end.",
    );
    let errors = check_errors(
        "program T;
      type Node = record Next: Option of Node; Job: Option of task of integer; end record;
      begin const Root: Node := Node( Next := None, Job := None );
      const Node: integer := 1; discard Root.Next; end.",
    );
    assert!(errors.iter().any(|error| error.code == SEMA_UNSAFE_DISCARD));
}

#[test]
fn only_direct_go_discard_suggests_fire_and_forget() {
    for (body, suggests_go) in [
        ("discard go Work();", true),
        ("discard (go Work());", true),
        ("const Job: task := go Work(); discard Job;", false),
        ("discard Spawn();", false),
    ] {
        let errors = check_errors(&format!(
            "program T;
          function Work(): integer; begin return 1; end function;
          function Spawn(): task of integer; begin return go Work(); end function;
          begin {body} end."
        ));
        let error = errors
            .iter()
            .find(|error| error.code == SEMA_UNSAFE_DISCARD)
            .unwrap();
        assert_eq!(
            error
                .help
                .as_deref()
                .is_some_and(|help| help.contains("go Worker();")),
            suggests_go
        );
    }
}

#[test]
fn generic_discard_uses_declared_constraints() {
    for constraint in ["Numeric", "Comparable"] {
        check_ok(&format!(
            "program T;
          function Ignore<T: {constraint}>(Value: T): integer;
          begin discard Value; discard Some(Value); return 0; end function;
          begin discard Ignore(42); end."
        ));
    }
    for constraint in ["", ": Printable"] {
        let errors = check_errors(&format!(
            "program T;
          function Ignore<T{constraint}>(Value: T): integer;
          begin discard Value; return 0; end function;
          begin discard Ignore(42); end."
        ));
        assert!(errors.iter().any(|error| error.code == SEMA_UNSAFE_DISCARD));
    }
    check_ok(
        "program T; function Identity<T>(Value: T): T; begin return Value; end function;
      begin discard Identity(42); end.",
    );
}

#[test]
fn channels_recurse_into_element_types() {
    check_ok(
        "program T; uses Std.Tasks;
      begin const Values: channel of integer := CreateChannel(1); discard Values; end.",
    );
    for ty in [
        "channel of task of integer",
        "channel of Option of array of task of integer",
        "channel of function(): integer",
    ] {
        let errors = check_errors(&format!(
            "program T; uses Std.Tasks;
          begin const Values: {ty} := CreateChannel(1); const Copy: {ty} := Values; discard Copy; end."
        ));
        assert!(
            errors.iter().any(|error| error.code == SEMA_UNSAFE_DISCARD),
            "{errors:?}"
        );
    }
}

#[test]
fn captured_channels_and_bound_receivers_are_checked_recursively() {
    check_ok("program T; uses Std.Tasks;
      begin const Values: channel of integer := CreateChannel(1);
      discard function(): integer begin const Copy: channel of integer := Values; return 1; end function;
      end.");
    let errors = check_errors("program T; uses Std.Tasks;
      type Box = record Job: Option of task of integer;
        function Ready(Self: Box): boolean; begin return true; end function;
      end record;
      begin const Values: channel of task of integer := CreateChannel(1);
      discard function(): integer begin const Copy: channel of task of integer := Values; return 1; end function;
      const Value: Box := Box( Job := None );
      discard Value.Ready;
      end.");
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.code == SEMA_UNSAFE_DISCARD)
            .count(),
        2
    );
}

#[test]
fn safe_closures_keep_proofs_through_bindings_and_returns() {
    check_ok(
        "program T;
      function Make(Value: integer): function(): integer;
      begin return function(): integer begin return Value; end function; end function;
      begin const Value: integer := 1;
      const GetValue: function(): integer := function(): integer begin return Value; end function;
      const Copy: function(): integer := GetValue;
      discard Copy; discard [Copy]; discard Some(Copy); discard Make(3);
      var Count: integer := 0;
      discard procedure() begin Count := Count + 1; end procedure;
      end.",
    );
}

#[test]
fn direct_and_transitive_task_captures_are_rejected() {
    let errors = check_errors(
        "program T; uses Std.Tasks;
      function Work(): integer; begin return 1; end function;
      begin const Job: task := go Work();
      const GetValue: function(): integer := function(): integer begin return Wait(Job); end function;
      const Outer: function(): integer := function(): integer begin return GetValue(); end function;
      discard GetValue; discard Outer; discard [Outer]; end.",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.code == SEMA_UNSAFE_DISCARD)
            .count(),
        3
    );
}

#[test]
fn callable_parameters_have_unknown_captures() {
    let errors = check_errors(
        "program T;
      procedure Ignore(Value: function(): integer); begin discard Value; end procedure;
      begin end.",
    );
    assert!(errors.iter().any(|error| error.code == SEMA_UNSAFE_DISCARD));
}

#[test]
fn omitted_record_fields_keep_default_capture_proofs() {
    check_ok("program T;
      type Box = record Value: function(): integer := function(): integer begin return 1; end function; end record;
      begin const Value: Box := Box( ); discard Value; end.");
    let errors = check_errors(
        "program T; uses Std.Tasks;
      function Work(): integer; begin return 1; end function;
      function Make(Job: task of integer): function(): integer;
      begin return function(): integer begin return Wait(Job); end function; end function;
      const Callback: function(): integer := Make(go Work());
      type Box = record Value: function(): integer := Callback; end record;
      begin const Value: Box := Box( ); discard Value; end.",
    );
    assert!(errors.iter().any(|error| error.code == SEMA_UNSAFE_DISCARD));
}

#[test]
fn scalar_loop_bindings_are_safe_closure_captures() {
    check_ok(
        "program T; begin
      for Value: integer := 1 to 2 do
        discard function(): integer begin return Value; end function;
      end for;
      end.",
    );
}

#[test]
fn mutable_callable_assignments_do_not_claim_unknown_captures_are_safe() {
    let errors = check_errors(
        "program T;
      procedure Ignore(Other: function(): integer);
      begin var Value: function(): integer := function(): integer begin return 1; end function;
      Value := Other; discard Value;
      discard function(): integer begin return Value(); end function;
      end procedure; begin end.",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.code == SEMA_UNSAFE_DISCARD)
            .count(),
        2
    );
}

#[test]
fn callable_signatures_and_record_methods_do_not_own_result_handles() {
    check_ok(
        "program T;
      function Work(): integer; begin return 1; end function;
      type Box = record Value: integer;
        function Start(Self: Box): task of integer;
        begin return go Work(); end function;
      end record;
      begin const Value: Box := Box( Value := 1 );
      discard Value; discard Value.Start;
      discard function(Job: task of integer): task of integer begin return Job; end function;
      end.",
    );
}
