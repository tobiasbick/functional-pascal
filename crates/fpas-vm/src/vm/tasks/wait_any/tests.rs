//! End-to-end wait-any progress with one scheduler worker.

#[test]
fn task_count_is_checked_before_allocating_identity_storage() {
    assert!(super::validate_task_count(0).is_err());
    assert!(super::validate_task_count(1).is_ok());
    assert!(super::validate_task_count(super::MAX_TASKS).is_ok());
    assert!(super::validate_task_count(super::MAX_TASKS + 1).is_err());
}

#[test]
fn one_worker_completes_nested_wait_any() {
    let (program, errors) = fpas_parser::parse(
        r#"program SingleWorkerWaitAny;
uses Std.Task, Std.Time;
function Child(): integer;
begin
  Sleep(1);
  return 9
end;
function Parent(): integer;
begin
  var T: task := go Child();
  if WaitAny([T]) <> 0 then panic('child index');
  return Wait(T)
end;
begin
  var T: task := go Parent();
  if WaitAny([T]) <> 0 then panic('parent index');
  if Wait(T) <> 9 then panic('result')
end."#,
    );
    assert!(errors.is_empty(), "{errors:?}");
    let mut vm = crate::vm::Vm::new(fpas_compiler::compile(&program).expect("compile"));
    vm.pool_size = 1;
    vm.run().expect("single-worker progress");
}
