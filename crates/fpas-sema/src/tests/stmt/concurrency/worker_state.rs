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
            "const Group := CreateTaskGroup(); discard StartTaskInGroup(Group, GroupWork);",
            "const Group := CreateTaskGroup(); discard StartSupervisedTask(Group, GroupWork, 0, 0);",
        ] {
            let errors = check_errors(&format!("program T; uses Std.Tasks;
                var Counter: integer := 0;
                function ReadGlobal(): integer; begin return Counter; end function;
                function Invoke(F: function(): integer): integer; begin return F(); end function;
                function Work(): integer; begin {access} end function;
                function GroupWork(Token: CancellationToken): integer; begin return Work(); end function;
                begin {start} end."));
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
        "for F: function(Token: CancellationToken): integer in [Work] do Start(Group, F); end for;",
    ] {
        let source = format!(
            "program T; uses Std.Tasks;
            var Counter: integer := 1;
            function Work(Token: CancellationToken): integer; begin return Counter; end function;
            procedure Start(Group: TaskGroup; F: function(Token: CancellationToken): integer);
            begin discard StartTaskInGroup(Group, F); end procedure;
            begin const Group := CreateTaskGroup(); {body} end."
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
    check_ok("program T; uses Std.Tasks;
        var Counter: integer := 1;
        function Work(Value: integer; Queue: channel of integer): integer;
        begin var Local := Value; Local := Local + 1; discard Send(Queue, Local); return Local; end function;
        begin const Queue: channel of integer := CreateChannel(1);
        scope const Job := go Work(Counter, Queue); discard Job; discard [Job];
        discard Receive(Queue); discard Wait(Job); end scope; end.");
}

#[test]
fn bound_worker_methods_keep_foreign_global_dependencies() {
    for start in [
        "scope discard go Bound(); end scope;",
        "const Group := CreateTaskGroup(); discard StartTaskInGroup(Group, GroupBound);",
    ] {
        let errors = check_errors(&format!("program T; uses Std.Tasks;
            var Counter: integer := 1;
            type Box = record Value: integer;
            function Work(Self: Box): integer; begin return Counter + Self.Value; end function;
            function GroupWork(Self: Box; Token: CancellationToken): integer; begin return Self.Work(); end function;
            end record;
            begin const Value := Box(Value := 2);
            const Bound: function(): integer := Value.Work;
            const GroupBound: function(Token: CancellationToken): integer := Value.GroupWork;
            {start} end."));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_FOREIGN_TASK_STATE),
            "{start}: {errors:?}"
        );
    }
}
