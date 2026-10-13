//! Source-level task result contracts and rejection of erased task results.
//!
//! Documentation: `docs/pascal/language/concurrency/task-handles.md`.

use super::super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;

const DECLARATIONS: &str = "\
uses Std.Tasks;
function Number(): integer; begin return 42; end function;
function Text(): string; begin return 'answer'; end function;
procedure Work(); begin null; end procedure;
";

#[test]
fn procedure_task_storage_never_erases_a_function_result() {
    for (declarations, body) in [
        ("", "const Job: task := go Number();"),
        (
            "function Read(Job: task): string; begin return Wait(Job); end function;",
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
    for value in ["Some(Wait(Job))", "[Wait(Job)]", "Ok(Wait(Job))"] {
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
            "function Fetch<T>(Job: task of T): T; begin return Wait(Job); end function;",
            "discard Some(Fetch(go Work()));",
        ),
        (
            "function Fetch<T>(Job: task of T): T; begin return Wait(Job); end function;",
            "const Job: task := go Work(); const FunctionJob: task := go Fetch(Job); Wait(FunctionJob);",
        ),
        (
            "type Holder of T = record Job: task of T; end record;",
            "const Value := Holder(Job := go Work()); Wait(Value.Job);",
        ),
        (
            "type Holder of T = enum Held(Job: task of T); end enum;",
            "const Value := Holder.Held(go Work()); case Value of when Holder.Held(const Job): Wait(Job); end case;",
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
            "const Job: task of channel of integer := go CreateChannel(1);",
        ),
        (
            "function Worker<T: Numeric>(Token: T): T; begin return Token + 1; end function;",
            "const Group: TaskGroup := CreateTaskGroup(); const CaseValue: WaitCase := TaskCase(StartTaskInGroup(Group, Worker), procedure() begin null; end procedure);",
        ),
        (
            "function Worker(var Token: CancellationToken): integer; begin return 0; end function;",
            "const Group: TaskGroup := CreateTaskGroup(); const Job: task of integer := StartTaskInGroup(Group, Worker);",
        ),
        (
            "procedure Worker<T: Numeric>(Token: T); begin discard Token + Token; end procedure;",
            "const Group: TaskGroup := CreateTaskGroup(); const Job: task := StartSupervisedTask(Group, Worker, 0, 0);",
        ),
        (
            "procedure Worker(var Token: CancellationToken); begin null; end procedure;",
            "const Group: TaskGroup := CreateTaskGroup(); const Job: task := StartSupervisedTask(Group, Worker, 0, 0);",
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
            "program T; uses Std.Tasks;
function Invoke(Callback: function(): string): string; begin return Callback(); end function;
function Echo(Value: string): string; begin return Value; end function;
procedure Outer<T>(Value: T);
function Read<T>(): T; begin return Value; end function;
begin scope  const Job: task of string := go {call}; discard Wait(Job);  end scope; end procedure;
begin Outer(42); end."
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
        "program T; {DECLARATIONS}
function Fetch<T>(Job: task of T): T; begin return Wait(Job); end function;
function Identity<T>(Value: T): T; begin return Value; end function;
function Spawn<T>(Group: TaskGroup; Value: T): task of T;
begin return StartTaskInGroup(Group, function(Token: CancellationToken): T begin return Value; end function); end function;
function GenericWorker<T>(Token: T): integer; begin return 42; end function;
function Answer(): integer; begin return 42; end function;
function Invoke(Callback: function(): integer): integer; begin return Callback(); end function;
procedure Join(Job: task); begin Wait(Job); end procedure;
begin scope
  const Empty: Option of integer := None;
  const Optional: task of Option of integer := go Identity(Empty);
  discard Wait(Optional);
  const GenericGroup := CreateTaskGroup();
  const Generic: task of integer := Spawn(GenericGroup, 42);
  discard Wait(Generic);
  const CallbackJob: task of integer := go Invoke(function(): integer begin return Answer(); end function);
  const DirectJob: task of integer := go Answer();
  discard Wait(CallbackJob); discard Wait(DirectJob);
  const Worker: function(Token: CancellationToken): integer := GenericWorker;
  const Group: TaskGroup := CreateTaskGroup();
  const Grouped: task of integer := StartTaskInGroup(Group, Worker);
  const Supervised: task of integer := StartSupervisedTask(Group, Worker, 0, 0);
  discard Wait(Grouped); discard Wait(Supervised);
  discard CloseTaskGroup(Group);
  const Numbers: array of task of integer := [go Number(), go Number()];
  const Procedures: array of task := [go Work(), go Work()];
  WaitAll(Numbers); WaitAll(Procedures);
  discard WaitAny(Numbers); discard WaitAny(Procedures);
  const Answer: integer := Wait(Numbers[0]);
  Join(Procedures[0]);
  const GroupProcedure := StartTaskInGroup(GenericGroup, procedure(Token: CancellationToken) begin null; end procedure);
  const Nested: task of task := go Identity(GroupProcedure);
  const Copy: task := Wait(Nested);
  Join(Copy);
  const Label: task of string := go Text();
  const Cases: array of WaitCase := [
    TaskCase(Numbers[1], procedure() begin null; end procedure),
    TaskCase(Label, procedure() begin null; end procedure),
    TaskCase(Procedures[1], procedure() begin null; end procedure)
  ];
  discard Select(Cases);
 end scope; end."
    ));
}
