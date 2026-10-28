use super::*;

#[test]
fn retained_exports_survive_shutdown_without_retaining_the_session() {
    let image = image("program P;
        function Make(): function(): integer;
        begin var Count := 7; return function(): integer begin return Count; end function; end function;
        begin end.");
    let mut session = CallbackSession::from_shared(Arc::clone(&image));
    let value = invoke(&mut session, &image, "Make", vec![]);
    let cell = value
        .child(HostChild::Capture(0))
        .expect("capture")
        .expect("cell");
    let domain = cell.0.origin.domain.clone();
    assert!(domain.root_count() >= 3);
    session.shutdown();
    assert!(session.invoke(function(&image, "Make"), vec![]).is_err());
    drop(session);
    let before = domain.maintenance_generation();
    drop(value);
    assert!(domain.maintenance_generation() > before);
    cell.replace_cell(Value::Integer(42).into())
        .expect("retired cell mutation");
    assert_eq!(
        cell.read_cell()
            .expect("retired read")
            .try_to_data()
            .expect("integer"),
        Value::Integer(42)
    );
    let mut seen = 0;
    domain
        .trace_registered_roots(&mut |_, _| seen += 1)
        .expect("root view");
    assert!(seen > 0);
    drop(cell);
    assert_eq!(domain.root_count(), 0);
}

#[test]
fn failed_host_mutation_preserves_contents_and_valid_mutation_requests_maintenance() {
    let image = empty_image();
    let domain = MemoryDomain::new();
    let context = HostValues::for_executable(
        domain.clone(),
        &image,
        Arc::new(std::sync::RwLock::new(Vec::new())),
    );
    let cell = domain.new_cell(Value::Integer(7)).expect("cell");
    let value = exported(&context, Value::Cell(cell));
    let foreign_domain = MemoryDomain::new();
    let foreign = foreign_domain.new_cell(Value::Unit).expect("foreign");
    let before = domain.maintenance_generation();
    assert_eq!(
        value.replace_cell(Value::Cell(foreign).into()),
        Err(HostValueError::SessionBound)
    );
    assert_eq!(domain.maintenance_generation(), before);
    assert_eq!(
        value
            .read_cell()
            .expect("read")
            .try_to_data()
            .expect("data"),
        Value::Integer(7)
    );
    let before = domain.maintenance_generation();
    value
        .replace_cell(Value::Integer(8).into())
        .expect("replace");
    assert!(domain.maintenance_generation() > before);
}

#[test]
fn replacing_one_retained_root_preserves_an_independent_clone() {
    let context = HostValues::new(MemoryDomain::new());
    let mut first = exported(&context, Value::Integer(1));
    let second = first.clone();
    let before = context.origin.domain.maintenance_generation();
    first
        .replace(Value::Integer(2).into())
        .expect("replace root");
    assert_eq!(first.try_to_data().expect("first"), Value::Integer(2));
    assert_eq!(second.try_to_data().expect("second"), Value::Integer(1));
    assert!(context.origin.domain.maintenance_generation() > before);
}

#[test]
fn host_clones_are_independent_registered_roots_even_after_retirement() {
    let context = HostValues::new(MemoryDomain::new());
    let a = exported(&context, Value::Integer(1));
    let b = a.clone();
    let domain = context.origin.domain.clone();
    drop(context);
    assert_eq!(domain.root_count(), 2);
    let permit = domain.access();
    drop(a);
    assert_eq!(domain.maintenance_generation(), 0);
    drop(permit);
    assert_eq!(domain.maintenance_generation(), 1);
    assert_eq!(domain.root_count(), 1);
    assert_eq!(b.try_to_data().expect("surviving clone"), Value::Integer(1));
}

#[test]
fn shutdown_before_a_vm_call_cannot_reenter_a_closed_runtime() {
    let image = empty_image();
    let mut vm = crate::execution::Vm::new(image.as_ref().clone());
    vm.shutdown_handle().shutdown();
    let error = vm
        .call(image.executable().entry, vec![])
        .expect_err("closed VM");
    assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN);
}
