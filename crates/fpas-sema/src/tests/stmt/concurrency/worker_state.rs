//! Source boundaries for worker ownership and transitive foreign mutable access.

use super::super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::{SEMA_FOREIGN_TASK_STATE, SEMA_GO_OUTSIDE_OWNING_SCOPE};

#[test]
fn named_and_anonymous_helpers_never_inherit_a_callers_scope() {
    for helper in [
        "procedure Helper(); begin discard go Work(); end procedure;",
        "const Helper: procedure() := procedure() begin discard go Work(); end procedure;",
    ] {
        let errors = check_errors(&format!(
            "program T;
            procedure Work(); begin null; end procedure; {helper}
            begin scope Helper(); end scope; end."
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_GO_OUTSIDE_OWNING_SCOPE),
            "{helper}: {errors:?}"
        );
    }
}

#[test]
fn every_worker_start_rejects_direct_and_transitive_global_reads_and_writes() {
    for access in [
        "return Counter;",
        "Counter := Counter + 1; return 1;",
        "return ReadGlobal();",
        "return Invoke(ReadGlobal);",
    ] {
        for start in [
            "scope discard go Work(); end scope;",
            "const Group := Tasks.CreateTaskGroup(); discard Tasks.StartTaskInGroup(Group, GroupWork);",
            "const Group := Tasks.CreateTaskGroup(); discard Tasks.StartSupervisedTask(Group, GroupWork, 0, 0);",
        ] {
            let errors = check_errors(&format!(
                "program T; uses Std.Tasks as Tasks;\n                var Counter: integer := 0;\n                function ReadGlobal(): integer; begin return Counter; end function;\n                function Invoke(F: function(): integer): integer; begin return F(); end function;\n                function Work(): integer; begin {access} end function;\n                function GroupWork(Token: Tasks.CancellationToken): integer; begin return Work(); end function;\n                begin {start} end."
            ));
            assert!(
                errors
                    .iter()
                    .any(|error| error.code == SEMA_FOREIGN_TASK_STATE),
                "{access} / {start}: {errors:?}"
            );
        }
    }
}

#[test]
fn callback_worker_role_propagates_through_helpers_patterns_and_loops() {
    for body in [
        "Start(Group, Work);",
        "case Some(Work) of when Some(const F): Start(Group, F); when None: null; end case;",
        "for F: function(Token: Tasks.CancellationToken): integer in [Work] do Start(Group, F); end for;",
    ] {
        let source = format!(
            "program T; uses Std.Tasks as Tasks;\n            var Counter: integer := 1;\n            function Work(Token: Tasks.CancellationToken): integer; begin return Counter; end function;\n            procedure Start(Group: Tasks.TaskGroup; F: function(Token: Tasks.CancellationToken): integer);\n            begin discard Tasks.StartTaskInGroup(Group, F); end procedure;\n            begin const Group := Tasks.CreateTaskGroup(); {body} end."
        );
        let (program, parsing) = fpas_parser::parse(&source);
        assert!(parsing.is_empty(), "{parsing:?}");
        let errors = crate::analyze(&program);
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_FOREIGN_TASK_STATE),
            "{body}: {errors:?}"
        );
    }
}

#[test]
fn caller_value_reads_worker_locals_and_channels_remain_valid() {
    check_ok(
        "program T; uses Std.Tasks as Tasks;\n        var Counter: integer := 1;\n        function Work(Value: integer; Queue: channel of integer): integer;\n        begin var Local := Value; Local := Local + 1; discard Tasks.Send(Queue, Local); return Local; end function;\n        begin const Queue: channel of integer := Tasks.CreateChannel(1);\n        scope const Job := go Work(Counter, Queue); discard Job; discard [Job];\n        discard Tasks.Receive(Queue); discard Tasks.Wait(Job); end scope; end.",
    );
}

#[test]
fn bound_worker_methods_keep_foreign_global_dependencies() {
    for start in [
        "scope discard go Bound(); end scope;",
        "const Group := Tasks.CreateTaskGroup(); discard Tasks.StartTaskInGroup(Group, GroupBound);",
    ] {
        let errors = check_errors(&format!(
            "program T; uses Std.Tasks as Tasks;\n            var Counter: integer := 1;\n            type Box = record Value: integer;\n            function Work(Self: Box): integer; begin return Counter + Self.Value; end function;\n            function GroupWork(Self: Box; Token: Tasks.CancellationToken): integer; begin return Self.Work(); end function;\n            end record;\n            begin const Value := Box(Value := 2);\n            const Bound: function(): integer := Value.Work;\n            const GroupBound: function(Token: Tasks.CancellationToken): integer := Value.GroupWork;\n            {start} end."
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_FOREIGN_TASK_STATE),
            "{start}: {errors:?}"
        );
    }
}
