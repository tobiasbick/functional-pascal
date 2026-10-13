//! Ordinary value disposal; task ownership and transfer are separate boundaries.

use super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_DISCARD_REQUIRES_VALUE;

#[test]
fn discard_accepts_scalars_results_and_explicitly_ignored_calls() {
    check_ok(
        "program T;
        function Value(): result of (integer, string); begin return Ok(1); end function;
        begin discard 42; discard 'hello'; discard Value(); discard Some(1);
        discard [1, 2]; discard ['key': 1]; end.",
    );
}

#[test]
fn discard_rejects_procedure_calls() {
    let errors = check_errors(
        "program T; procedure Work(); begin end procedure;
        begin discard Work(); end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_DISCARD_REQUIRES_VALUE)
    );
}

#[test]
fn discard_accepts_task_containing_types_even_when_empty_or_inactive() {
    for (declarations, ty, initializer) in [
        ("", "array of task of integer", "[]"),
        ("", "Option of task of integer", "None"),
        ("", "result of (integer, task of integer)", "Ok(1)"),
        ("", "dict of string to array of task of integer", "[:]"),
        (
            "type Box = record Job: Option of task of integer; end record;",
            "Box",
            "Box(Job := None)",
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
        (
            "type Node = record Next: Option of Node; Job: Option of task of integer; end record;",
            "Node",
            "Node(Next := None, Job := None)",
        ),
    ] {
        check_ok(&format!(
            "program T; {declarations} begin const Value: {ty} := {initializer}; discard Value; end."
        ));
    }
}

#[test]
fn discard_accepts_unconstrained_generic_and_callable_parameters() {
    check_ok(
        "program T;
        procedure Ignore<T>(Value: T); begin discard Value; discard Some(Value); end procedure;
        procedure IgnoreCallable(Value: function(): integer);
        begin var Copy := Value; discard Copy; discard [Copy]; end procedure;
        begin Ignore(42); Ignore(function(): integer begin return 1; end function); end.",
    );
}

#[test]
fn discard_accepts_handles_aliases_aggregates_and_task_capturing_closures() {
    check_ok(
        "program T; uses Std.Tasks;
        function Work(): integer; begin return 1; end function;
        function Make(Job: task of integer): function(): integer;
        begin return function(): integer begin return Wait(Job); end function; end function;
        begin scope
        const Job := go Work(); const Copy := Job;
        discard Job; discard Copy; discard [Job]; discard Some(Job);
        discard go Work(); discard (go Work());
        end scope; end.",
    );
    check_ok(
        "program T; uses Std.Tasks;
        procedure Ignore(Job: task of integer);
        begin const Read := function(): integer begin return Wait(Job); end function;
        const Outer := function(): integer begin return Read(); end function;
        discard Read; discard Outer; discard [Outer];
        end procedure; begin end.",
    );
}

#[test]
fn discard_accepts_channels_bound_receivers_and_mutable_captures() {
    check_ok(
        "program T; uses Std.Tasks;
        type Box = record Job: Option of task of integer;
        function Ready(Self: Box): boolean; begin return true; end function;
        end record;
        begin const Values: channel of task of integer := CreateChannel(1);
        const Copy := Values; discard Copy;
        discard function(): integer begin discard Values; return 1; end function;
        const Value: Box := Box(Job := None); discard Value.Ready;
        var Count := 0;
        discard procedure() begin Count := Count + 1; end procedure;
        end.",
    );
}

#[test]
fn unused_results_always_offer_the_ordinary_discard_repair() {
    let errors = check_errors(
        "program T; uses Std.Tasks;
        function Work(): integer; begin return 1; end function;
        begin scope go Work(); end scope; end.",
    );
    let error = errors
        .iter()
        .find(|error| error.code == fpas_diagnostics::codes::SEMA_UNUSED_FUNCTION_RESULT)
        .unwrap();
    assert!(error.help.as_deref().unwrap().contains("discard go"));
}
