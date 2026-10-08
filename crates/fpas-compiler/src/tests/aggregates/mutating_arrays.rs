//! Caller-mutating intrinsic regression programs run through the compiler and VM.

use super::assert_succeeds;

#[test]
fn mutating_receivers_write_through_fields_elements_and_forwarding() {
    assert_succeeds(include_str!(
        "../../../../../tests/stdlib/array/mutating_receivers_test.fpas"
    ));
}

#[test]
fn mutating_receivers_fix_their_path_before_evaluating_value_arguments() {
    assert_succeeds(include_str!(
        "../../../../../tests/stdlib/array/mutating_receiver_order_test.fpas"
    ));
}

#[test]
fn mutating_receiver_writes_survive_try_exit() {
    assert_succeeds(include_str!(
        "../../../../../tests/stdlib/array/mutating_receiver_try_test.fpas"
    ));
}

#[test]
fn native_and_forwarded_receivers_use_the_same_storage_paths() {
    assert_succeeds(
        r#"program T;

type Holder = record Items: array of integer; end record;
procedure Add(var Items: array of integer);
begin
  Items.Push(3);
end procedure;
begin
  var H: Holder := Holder( Items := [1] );
  H.Items.Push(2);
  Add(var H.Items);
  if H.Items.Pop() <> 3 then panic('forwarded write'); end if;
  if H.Items.Pop() <> 2 then panic('field write'); end if;
  var Rows: array of array of integer := [[1]];
  Rows[0].Push(4);
  if Rows[0].Pop() <> 4 then panic('element write'); end if;
end."#,
    );
}

#[test]
fn writes_before_panic_and_failed_pop_remain_in_caller_storage() {
    use fpas_vm::{DebugExpression, DebugRunResult, DebugSession, DebugStopReason};

    for tail in ["panic('stopped');", "const N: integer := H.Items.Pop();"] {
        let mutation = if tail.starts_with("panic") {
            "H.Items.Push(2); const Last: integer := H.Items.Pop(); H.Items.Push(Last + 1);"
        } else {
            "const Last: integer := H.Items.Pop();"
        };
        let source = format!(
            r#"program T;

type Holder = record Items: array of integer; end record;
var Global: Holder := Holder( Items := [1] );
procedure Change(var H: Holder);
begin {mutation} {tail} end procedure;
begin Change(var Global); end."#
        );
        let executable = crate::compile(&super::super::parse_ok(&source)).expect("compile");
        let mut session = DebugSession::new(executable).expect("debug session");
        let DebugRunResult::Stopped(stop) = session.continue_execution().expect("runtime failure")
        else {
            panic!("expected runtime failure");
        };
        assert_eq!(stop.reason, DebugStopReason::RuntimeError);
        let items = DebugExpression::Field {
            base: Box::new(DebugExpression::Name("Global".to_string())),
            name: "Items".to_string(),
        };
        let value = session
            .evaluate(&items, None)
            .expect("caller array after failure");
        if tail.starts_with("panic") {
            assert_eq!(value.indexed_variables, 2);
            let last = DebugExpression::Index {
                base: Box::new(items),
                index: Box::new(DebugExpression::Integer(1)),
            };
            assert_eq!(
                session
                    .evaluate(&last, None)
                    .expect("retained last value")
                    .value,
                "3"
            );
        } else {
            assert_eq!(value.indexed_variables, 0);
        }
    }
}

#[test]
fn native_named_arguments_factories_and_empty_checks_preserve_evaluation_order() {
    assert_succeeds(include_str!(
        "../../../../../tests/stdlib/fluent/native_named_order_test.fpas"
    ));
}

#[test]
fn native_tasks_preserve_named_argument_mapping_factories_and_empty_results() {
    assert_succeeds(include_str!(
        "../../../../../tests/stdlib/fluent/native_tasks_test.fpas"
    ));
}
