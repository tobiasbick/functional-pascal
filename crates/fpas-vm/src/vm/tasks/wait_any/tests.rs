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
uses Std.Tasks as Tasks; uses Std.Time as Time;
function Child(): integer;
begin
  Time.Sleep(1);
  return 9;
end function;
function Parent(): integer;
begin
  const T: task := go Child();
  if Tasks.WaitAny([T]) <> 0 then panic('child index'); end if;
  return Tasks.Wait(T);
end function;
begin
  const T: task := go Parent();
  if Tasks.WaitAny([T]) <> 0 then panic('parent index'); end if;
  if Tasks.Wait(T) <> 9 then panic('result'); end if;
end program;"#,
    );
    assert!(errors.is_empty(), "{errors:?}");
    let mut vm = crate::vm::Vm::new(fpas_compiler::compile(&program).expect("compile"));
    vm.pool_size = 1;
    vm.run().expect("single-worker progress");
}
