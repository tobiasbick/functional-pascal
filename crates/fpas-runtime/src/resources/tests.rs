//! Budgets must preserve aliasing, release accounting and execution isolation.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "test assertions"
)]

use super::*;
use std::sync::Arc;

fn budget(bytes: usize) -> Arc<Budget> {
    let budget = Arc::new(Budget::default());
    budget.configure(ExecutionLimits {
        max_heap_bytes: Some(bytes),
        ..Default::default()
    });
    budget
}

#[test]
fn reservations_release_and_cannot_exceed_a_shared_limit() {
    let budget = budget(1024);
    let _scope = Scope::enter(&budget);
    let first = Reservation::new(600);
    assert!(matches!(
        catch(|| Reservation::new(425)),
        Err(Exhausted::Heap { .. })
    ));
    assert_eq!(budget.usage().heap_bytes, 600);
    drop(first);
    assert_eq!(budget.usage().heap_bytes, 0);
    assert_eq!(budget.usage().peak_heap_bytes, 600);
}

#[test]
fn allocation_boundary_does_not_swallow_unrelated_rust_panics() {
    assert!(std::panic::catch_unwind(|| catch(|| panic!("unrelated host failure"))).is_err());
}

#[test]
fn shared_values_count_once_and_copies_count_until_the_last_release() {
    let budget = budget(4096);
    let _scope = Scope::enter(&budget);
    let original = crate::SharedArray::from(vec![crate::Value::Integer(1); 8]);
    let before = budget.usage().heap_bytes;
    let mut alias = original.clone();
    assert_eq!(budget.usage().heap_bytes, before);
    alias.push(crate::Value::Integer(9));
    assert!(budget.usage().heap_bytes > before * 2);
    drop(alias);
    assert_eq!(budget.usage().heap_bytes, before);
    drop(original);
    assert_eq!(budget.usage().heap_bytes, 0);
}

#[test]
fn each_execution_has_its_own_budget_even_when_scopes_nest() {
    let first = budget(128);
    let second = budget(256);
    let _first = Scope::enter(&first);
    let a = Reservation::new(100);
    {
        let _second = Scope::enter(&second);
        let b = Reservation::new(200);
        assert_eq!(first.usage().heap_bytes, 100);
        assert_eq!(second.usage().heap_bytes, 200);
        drop(b);
    }
    let c = Reservation::new(28);
    assert_eq!(first.usage().heap_bytes, 128);
    drop((a, c));
    assert_eq!(first.usage().heap_bytes, 0);
}

#[test]
fn copy_on_write_charges_the_execution_making_the_copy() {
    let first = budget(1024);
    let second = budget(1024);
    let _first = Scope::enter(&first);
    let original = crate::SharedArray::from(vec![crate::Value::Integer(1); 4]);
    let before = first.usage().heap_bytes;
    {
        let _second = Scope::enter(&second);
        let mut copied = original.clone();
        copied.push(crate::Value::Integer(2));
        assert_eq!(first.usage().heap_bytes, before);
        assert!(second.usage().heap_bytes > before);
        drop(copied);
        assert_eq!(second.usage().heap_bytes, 0);
    }
    drop(original);
    assert_eq!(first.usage().heap_bytes, 0);
}

fn executable(body: &str) -> crate::VerifiedExecutable {
    let source = format!("program BudgetFixture; begin {body} end.");
    let (program, diagnostics) = fpas_parser::parse(&source);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    fpas_compiler::compile(&program).unwrap()
}

#[test]
fn normal_execution_reports_heap_exhaustion_instead_of_a_panic() {
    let mut vm = crate::execution::Vm::new(executable(
        "var Text: string := 'x'; while true do Text := Text + Text; end while;",
    ))
    .with_execution_limits(ExecutionLimits {
        max_heap_bytes: Some(16384),
        ..Default::default()
    });
    let error = vm.run().unwrap_err();
    assert_eq!(
        error.code,
        fpas_diagnostics::codes::RUNTIME_RESOURCE_LIMIT_EXCEEDED
    );
    assert!(error.message.contains("heap limit"));
    assert!(vm.execution_usage().peak_heap_bytes <= 16384);
}

#[test]
fn omitted_budgets_leave_normal_vm_allocation_unlimited() {
    let mut vm = crate::execution::Vm::new(executable(
        "var Text: string := 'x'; for Index: integer := 1 to 12 do Text := Text + Text; end for;",
    ));
    vm.run().unwrap();
    assert_eq!(vm.execution_usage(), ExecutionUsage::default());
}

#[test]
fn normal_execution_enforces_instruction_and_time_budgets() {
    for limits in [
        ExecutionLimits {
            max_instructions: Some(1000),
            ..Default::default()
        },
        ExecutionLimits {
            timeout: Some(Duration::from_millis(10)),
            ..Default::default()
        },
    ] {
        let mut vm = crate::execution::Vm::new(executable("while true do null; end while;"))
            .with_execution_limits(limits);
        let error = vm.run().unwrap_err();
        assert_eq!(
            error.code,
            fpas_diagnostics::codes::RUNTIME_RESOURCE_LIMIT_EXCEEDED
        );
        assert!(vm.execution_usage().instructions <= 1000);
    }
}

#[test]
fn concurrent_reservations_share_one_atomic_account() {
    let budget = budget(1024);
    let admitted = Arc::new(std::sync::Barrier::new(5));
    let released = Arc::new(std::sync::Barrier::new(5));
    std::thread::scope(|scope| {
        for _ in 0..4 {
            let budget = budget.clone();
            let admitted = admitted.clone();
            let released = released.clone();
            scope.spawn(move || {
                let _scope = Scope::enter(&budget);
                let memory = Reservation::new(256);
                admitted.wait();
                released.wait();
                drop(memory);
            });
        }
        admitted.wait();
        assert_eq!(budget.usage().heap_bytes, 1024);
        let _scope = Scope::enter(&budget);
        assert!(catch(|| Reservation::new(1)).is_err());
        released.wait();
    });
    assert_eq!(budget.usage().heap_bytes, 0);
    assert_eq!(budget.usage().peak_heap_bytes, 1024);
}

#[test]
fn byte_copy_refusal_preserves_both_aliases_and_releases_the_charge() {
    let budget = budget(256);
    let _scope = Scope::enter(&budget);
    let original = crate::SharedBytes::zeroed(128).unwrap();
    let used = budget.usage().heap_bytes;
    let mut edited = original.clone();
    assert!(catch(|| edited.set_at(0, 1)).is_err());
    assert_eq!(original[0], 0);
    assert_eq!(edited[0], 0);
    assert_eq!(budget.usage().heap_bytes, used);
    drop((original, edited));
    assert_eq!(budget.usage().heap_bytes, 0);
}

#[test]
fn string_indexes_and_wrapper_payloads_release_their_own_storage() {
    let budget = budget(8192);
    let _scope = Scope::enter(&budget);
    let text = crate::SharedStr::from("é".repeat(100));
    let before = budget.usage().heap_bytes;
    assert_eq!(text.char_at(80), Some('é'));
    assert!(budget.usage().heap_bytes > before);
    let wrapped = crate::Value::option_some(crate::Value::Str(text));
    assert!(budget.usage().heap_bytes > before);
    drop(wrapped);
    assert_eq!(budget.usage().heap_bytes, 0);
}

fn source_executable(source: &str) -> crate::VerifiedExecutable {
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    fpas_compiler::compile(&program).unwrap()
}

#[test]
fn child_tasks_cannot_escape_the_parent_heap_budget() {
    let executable = source_executable(
        "program SharedBudget;
uses Std.Tasks as Tasks;
function Grow(): string;
begin
  var Text: string := 'x';
  while true do Text := Text + Text; end while;
  return Text;
end function;
begin
  scope
    const Child: task of string := go Grow();
    discard Tasks.Wait(Child);
  end scope;
end.",
    );
    let mut vm = crate::execution::Vm::new(executable).with_execution_limits(ExecutionLimits {
        max_heap_bytes: Some(16384),
        ..Default::default()
    });
    let error = vm.run().unwrap_err();
    assert_eq!(
        error.code,
        fpas_diagnostics::codes::RUNTIME_RESOURCE_LIMIT_EXCEEDED
    );
    assert!(error.message.contains("heap limit"));
}

#[test]
fn debugger_uses_the_same_heap_account_and_reports_a_limit_error() {
    use crate::execution::{
        DebugErrorKind, DebugExecutionLimits, DebugInspectionLimits, DebugSession,
    };
    let executable =
        executable("var Text: string := 'x'; while true do Text := Text + Text; end while;");
    let mut session = DebugSession::with_limits(
        executable,
        Vec::new(),
        DebugInspectionLimits::default(),
        DebugExecutionLimits {
            max_heap_bytes: Some(16384),
            ..Default::default()
        },
    )
    .unwrap();
    let error = session.continue_execution().unwrap_err();
    assert_eq!(error.kind, DebugErrorKind::HeapLimit);
    assert!(session.execution_usage().peak_heap_bytes <= 16384);
}

#[test]
fn output_is_checked_before_it_is_published_even_without_a_heap_limit() {
    let budget = Arc::new(Budget::default());
    budget.configure(ExecutionLimits {
        max_output_bytes: Some(3),
        ..Default::default()
    });
    let _scope = Scope::enter(&budget);
    let mut console = crate::stdlib::Console::new();
    let location = crate::SourceLocation::new(1, 1);
    console
        .write_ln(&crate::Value::Str("ab".into()), location)
        .unwrap();
    assert!(catch(|| console.write_ln(&crate::Value::Str("c".into()), location)).is_err());
    assert_eq!(console.output().lines, ["ab"]);
    assert_eq!(budget.usage().output_bytes, 3);
}

#[test]
fn cleanup_has_a_separate_bounded_instruction_allowance() {
    let budget = Budget::default();
    budget.configure(ExecutionLimits {
        max_instructions: Some(1),
        ..Default::default()
    });
    assert!(budget.instruction(false).is_ok());
    assert!(budget.instruction(false).is_err());
    for _ in 0..100_000 {
        budget.instruction(true).unwrap();
    }
    assert!(budget.instruction(true).is_err());
    assert!(budget.instruction(false).is_err());
}

#[test]
fn channels_reserve_capacity_before_allocating_their_queue() {
    let executable = source_executable(
        "program ChannelBudget; uses Std.Tasks as Tasks; begin const Messages: channel of integer := Tasks.CreateChannel(1024); discard Tasks.CloseChannel(Messages); end.",
    );
    let mut vm = crate::execution::Vm::new(executable).with_execution_limits(ExecutionLimits {
        max_heap_bytes: Some(1024),
        ..Default::default()
    });
    let error = vm.run().unwrap_err();
    assert_eq!(
        error.code,
        fpas_diagnostics::codes::RUNTIME_RESOURCE_LIMIT_EXCEEDED
    );
    assert_eq!(vm.execution_usage().heap_bytes, 0);
}
