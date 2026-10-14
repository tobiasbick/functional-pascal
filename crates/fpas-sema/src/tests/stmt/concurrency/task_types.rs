//! Source-level task result contracts and rejection of erased task results.
//!
//! Documentation: `docs/pascal/language/concurrency/task-handles.md`.

use super::super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;

const DECLARATIONS: &str = "uses Std.Tasks as Tasks;\nfunction Number(): integer; begin return 42; end function;\nfunction Text(): string; begin return 'answer'; end function;\nprocedure Work(); begin null; end procedure;\n";

#[test]
fn procedure_task_storage_never_erases_a_function_result() {
    for (declarations, body) in [
        ("", "const Job: task := go Number();"),
        (
            "function Read(Job: task): string; begin return Tasks.Wait(Job); end function;",
            "const Job: task of integer := go Number(); const Value: string := Read(Job);",
        ),
        (
            "type ProcedureJob = task;",
            "const Job: ProcedureJob := go Number();",
        ),
        (
            "type Box = record Job: task; end record;",
            "const Value: Box := Box(Job := go Number());",
        ),
        ("", "const Jobs: array of task := [go Number()];"),
        ("", "const Job: Option of task := Some(go Number());"),
        (
            "",
            "const Job: result of (task, string) := Ok(go Number());",
        ),
        (
            "function Identity<T>(Value: T): T; begin return Value; end function;",
            "const Job: task := Identity(go Number());",
        ),
    ] {
        let source =
            format!("program T; {DECLARATIONS} {declarations} begin scope {body} end scope; end.");
        let errors = check_errors(&source);
        assert!(
            errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
            "accepted a function result as a procedure task: {source}\n{errors:?}"
        );
    }
}

#[test]
fn procedure_wait_cannot_supply_a_value_inside_an_aggregate() {
    for value in [
        "Some(Tasks.Wait(Job))",
        "[Tasks.Wait(Job)]",
        "Ok(Tasks.Wait(Job))",
    ] {
        let source = format!(
            "program T; {DECLARATIONS} begin scope  const Job: task := go Work(); discard {value};  end scope; end."
        );
        let errors = check_errors(&source);
        assert!(
            errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
            "procedure Wait supplied a value: {source}\n{errors:?}"
        );
    }
}

#[test]
fn generic_task_arguments_do_not_expose_internal_unit() {
    for (declarations, body) in [
        (
            "function Fetch<T>(Job: task of T): T; begin return Tasks.Wait(Job); end function;",
            "discard Some(Fetch(go Work()));",
        ),
        (
            "function Fetch<T>(Job: task of T): T; begin return Tasks.Wait(Job); end function;",
            "const Job: task := go Work(); const FunctionJob: task := go Fetch(Job); Tasks.Wait(FunctionJob);",
        ),
        (
            "type Holder of T = record Job: task of T; end record;",
            "const Value := Holder(Job := go Work()); Tasks.Wait(Value.Job);",
        ),
        (
            "type Holder of T = enum Held(Job: task of T); end enum;",
            "const Value := Holder.Held(go Work()); case Value of when Holder.Held(const Job): Tasks.Wait(Job); end case;",
        ),
        (
            "function Ignore<T>(Job: task of T): integer; begin return 0; end function;",
            "const Accept: function(Job: task): integer := Ignore;",
        ),
    ] {
        let source =
            format!("program T; {DECLARATIONS} {declarations} begin scope {body} end scope; end.");
        let errors = check_errors(&source);
        assert!(
            errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
            "a procedure task inferred an internal result type: {source}\n{errors:?}"
        );
    }
}

#[test]
fn task_starts_require_resolved_results_and_value_worker_parameters() {
    for (declarations, body) in [
        (
            "function Empty<T>(): array of T; begin return []; end function;",
            "go Empty();",
        ),
        (
            "function Empty<T>(): array of T; begin return []; end function; function Identity<T>(Value: T): T; begin return Value; end function;",
            "const Job: task of array of integer := go Identity(Empty());",
        ),
        (
            "function Identity<T>(Value: T): T; begin return Value; end function;",
            "const Job: task of Option of integer := go Identity(None);",
        ),
        (
            "",
            "const Job: task of channel of integer := go Tasks.CreateChannel(1);",
        ),
        (
            "function Worker<T: Numeric>(Token: T): T; begin return Token + 1; end function;",
            "const Group: Tasks.TaskGroup := Tasks.CreateTaskGroup(); const CaseValue: Tasks.WaitCase := Tasks.TaskCase(Tasks.StartTaskInGroup(Group, Worker), procedure() begin null; end procedure);",
        ),
        (
            "function Worker(var Token: Tasks.CancellationToken): integer; begin return 0; end function;",
            "const Group: Tasks.TaskGroup := Tasks.CreateTaskGroup(); const Job: task of integer := Tasks.StartTaskInGroup(Group, Worker);",
        ),
        (
            "procedure Worker<T: Numeric>(Token: T); begin discard Token + Token; end procedure;",
            "const Group: Tasks.TaskGroup := Tasks.CreateTaskGroup(); const Job: task := Tasks.StartSupervisedTask(Group, Worker, 0, 0);",
        ),
        (
            "procedure Worker(var Token: Tasks.CancellationToken); begin null; end procedure;",
            "const Group: Tasks.TaskGroup := Tasks.CreateTaskGroup(); const Job: task := Tasks.StartSupervisedTask(Group, Worker, 0, 0);",
        ),
    ] {
        let source =
            format!("program T; {DECLARATIONS} {declarations} begin scope {body} end scope; end.");
        let errors = check_errors(&source);
        assert!(
            errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
            "an unresolved or reference-taking task target was accepted: {source}\n{errors:?}"
        );
    }
}

#[test]
fn spawn_arguments_do_not_hide_ordinary_type_errors() {
    for call in [
        "Invoke(function(): string begin return Read(); end function)",
        "Echo(Read())",
    ] {
        let source = format!(
            "program T; uses Std.Tasks as Tasks;\nfunction Invoke(Callback: function(): string): string; begin return Callback(); end function;\nfunction Echo(Value: string): string; begin return Value; end function;\nprocedure Outer<T>(Value: T);\nfunction Read<T>(): T; begin return Value; end function;\nbegin scope  const Job: task of string := go {call}; discard Tasks.Wait(Job);  end scope; end procedure;\nbegin Outer(42); end."
        );
        assert!(
            check_errors(&source)
                .iter()
                .any(|error| error.code == SEMA_TYPE_MISMATCH),
            "a spawned string task accepted an integer value: {source}"
        );
    }
}

#[test]
fn typed_and_procedure_handles_support_completion_without_erasure() {
    check_ok(&format!(
        "program T; {DECLARATIONS}\nfunction Fetch<T>(Job: task of T): T; begin return Tasks.Wait(Job); end function;\nfunction Identity<T>(Value: T): T; begin return Value; end function;\nfunction Spawn<T>(Group: Tasks.TaskGroup; Value: T): task of T;\nbegin return Tasks.StartTaskInGroup(Group, function(Token: Tasks.CancellationToken): T begin return Value; end function); end function;\nfunction GenericWorker<T>(Token: T): integer; begin return 42; end function;\nfunction Answer(): integer; begin return 42; end function;\nfunction Invoke(Callback: function(): integer): integer; begin return Callback(); end function;\nprocedure Join(Job: task); begin Tasks.Wait(Job); end procedure;\nbegin scope\n  const Empty: Option of integer := None;\n  const Optional: task of Option of integer := go Identity(Empty);\n  discard Tasks.Wait(Optional);\n  const GenericGroup := Tasks.CreateTaskGroup();\n  const Generic: task of integer := Spawn(GenericGroup, 42);\n  discard Tasks.Wait(Generic);\n  const CallbackJob: task of integer := go Invoke(function(): integer begin return Answer(); end function);\n  const DirectJob: task of integer := go Answer();\n  discard Tasks.Wait(CallbackJob); discard Tasks.Wait(DirectJob);\n  const Worker: function(Token: Tasks.CancellationToken): integer := GenericWorker;\n  const Group: Tasks.TaskGroup := Tasks.CreateTaskGroup();\n  const Grouped: task of integer := Tasks.StartTaskInGroup(Group, Worker);\n  const Supervised: task of integer := Tasks.StartSupervisedTask(Group, Worker, 0, 0);\n  discard Tasks.Wait(Grouped); discard Tasks.Wait(Supervised);\n  discard Tasks.CloseTaskGroup(Group);\n  const Numbers: array of task of integer := [go Number(), go Number()];\n  const Procedures: array of task := [go Work(), go Work()];\n  Tasks.WaitAll(Numbers); Tasks.WaitAll(Procedures);\n  discard Tasks.WaitAny(Numbers); discard Tasks.WaitAny(Procedures);\n  const Answer: integer := Tasks.Wait(Numbers[0]);\n  Join(Procedures[0]);\n  const GroupProcedure := Tasks.StartTaskInGroup(GenericGroup, procedure(Token: Tasks.CancellationToken) begin null; end procedure);\n  const Nested: task of task := go Identity(GroupProcedure);\n  const Copy: task := Tasks.Wait(Nested);\n  Join(Copy);\n  const Label: task of string := go Text();\n  const Cases: array of Tasks.WaitCase := [\n    Tasks.TaskCase(Numbers[1], procedure() begin null; end procedure),\n    Tasks.TaskCase(Label, procedure() begin null; end procedure),\n    Tasks.TaskCase(Procedures[1], procedure() begin null; end procedure)\n  ];\n  discard Tasks.Select(Cases);\n end scope; end."
    ));
}
