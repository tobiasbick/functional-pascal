//! Explicit closures bind record arguments using ordinary capture rules.
//!
//! **Documentation:** `docs/pascal/language/functions/closures.md`

use super::super::{check_errors, check_ok};
use crate::analyze_with_types;

const DECLARATIONS: &str = "
type Counter = record Base: integer; end record;
function CounterAdd(Receiver: Counter; Value: integer): integer;
begin return Receiver.Base + Value; end function;
procedure CounterTouch(Receiver: Counter); begin null; end procedure;
procedure CounterIncrease(var Receiver: Counter); begin Receiver.Base := Receiver.Base + 1; end procedure;
";

#[test]
fn explicit_record_closures_have_function_and_procedure_types() {
    check_ok(&format!("program T; {DECLARATIONS}
        begin const C := Counter(Base := 10);
          const AddTen: function(Value: integer): integer := function(Value: integer): integer begin return CounterAdd(C, Value); end function;
          const Touch: procedure() := procedure() begin CounterTouch(C); end procedure;
          discard AddTen(2); Touch();
        end program;"));
}

#[test]
fn ordinary_routine_values_do_not_bind_record_arguments_implicitly() {
    for value in ["C.CounterAdd", "CounterAdd"] {
        let errors = check_errors(&format!(
            "program T; {DECLARATIONS}
          begin const C := Counter(Base := 10);
            const AddTen: function(Value: integer): integer := {value};
          end program;"
        ));
        assert!(!errors.is_empty(), "{value}");
    }
}

#[test]
fn explicit_record_closure_records_immutable_snapshot_capture() {
    let (program, errors) = fpas_parser::parse(&format!("program T; {DECLARATIONS}
        begin var C := Counter(Base := 10); const Snapshot := C;
          const AddTen := function(Value: integer): integer begin return CounterAdd(Snapshot, Value); end function;
        end program;"));
    assert!(errors.is_empty(), "{errors:#?}");
    let metadata = analyze_with_types(&program);
    assert!(metadata.errors.is_empty(), "{:#?}", metadata.errors);
    let captures = metadata.closure_infos.values().collect::<Vec<_>>();
    let [closure] = captures.as_slice() else {
        panic!("one explicit closure expected");
    };
    assert!(!closure.task_bound);
    assert_eq!(closure.captures.len(), 1);
    assert_eq!(closure.captures[0].name, "Snapshot");
    assert!(!closure.captures[0].mutable);
}

#[test]
fn mutable_record_capture_is_task_bound_and_snapshot_capture_can_spawn() {
    let source = format!(
        "program T; {DECLARATIONS}
        begin var C := Counter(Base := 10);
          const Action := procedure() begin CounterIncrease(var C); end procedure;
          go Action();
        end program;"
    );
    let errors = check_errors(&source);
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_TASK_BOUND_CALLABLE),
        "{errors:#?}"
    );
    check_ok(&format!(
        "program T; {DECLARATIONS}
        begin var C := Counter(Base := 10); const Snapshot := C;
          const Action := procedure() begin CounterTouch(Snapshot); end procedure;
          go Action();
        end program;"
    ));
}
