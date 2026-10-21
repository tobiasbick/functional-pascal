use super::*;

const SOURCE: &str = r#"
program HostCallables;
var Current: integer := 0;
procedure SetCurrent(Value: integer);
begin
  Current := Value;
end procedure;
function ReadCurrent(): integer;
begin
  return Current;
end function;
function Reader(): function(): integer;
begin
  return ReadCurrent;
end function;
function Invoke(Read: function(): integer): integer;
begin
  return Read();
end function;
function Touch(Value: integer): integer;
begin
  Current := Current + 1;
  return Current;
end function;
begin
end.
"#;

#[test]
fn shared_image_callable_uses_target_session_globals_and_fresh_image_is_rejected() {
    let image = image(SOURCE);
    let mut first = CallbackSession::from_shared(Arc::clone(&image));
    let mut second = CallbackSession::from_shared(Arc::clone(&image));
    invoke(
        &mut first,
        &image,
        "SetCurrent",
        vec![Value::Integer(11).into()],
    );
    invoke(
        &mut second,
        &image,
        "SetCurrent",
        vec![Value::Integer(42).into()],
    );
    let read = invoke(&mut first, &image, "Reader", vec![]);
    assert_eq!(
        invoke(&mut second, &image, "Invoke", vec![read.clone().into()])
            .try_to_data()
            .expect("integer"),
        Value::Integer(42)
    );
    // Pure extraction preserves the code token even when the envelope is discarded.
    assert_eq!(
        invoke(
            &mut second,
            &image,
            "Invoke",
            vec![read.try_to_data().expect("pure callable").into()]
        )
        .try_to_data()
        .expect("integer"),
        Value::Integer(42)
    );
    let independent = Arc::new((*image).clone());
    let mut foreign = CallbackSession::from_shared(Arc::clone(&independent));
    assert_eq!(
        foreign
            .invoke(function(&independent, "Invoke"), vec![read.into()])
            .expect_err("same numeric IDs do not identify the executable")
            .code,
        RUNTIME_INVALID_TASK
    );
}

#[test]
fn nested_and_bound_foreign_callables_fail_before_target_side_effects() {
    let image = image(SOURCE);
    let foreign = (*image).clone();
    let mut session = CallbackSession::from_shared(Arc::clone(&image));
    invoke(
        &mut session,
        &image,
        "SetCurrent",
        vec![Value::Integer(10).into()],
    );
    let target = foreign
        .function_target(function(&foreign, "ReadCurrent"))
        .expect("foreign target");
    let values = [
        Value::function(target.clone(), vec![]),
        Value::option_some(Value::Array(
            vec![Value::function(target.clone(), vec![])].into(),
        )),
        Value::bound_function(target, Value::Integer(1)),
        Value::function(
            image
                .function_target(function(&image, "ReadCurrent"))
                .expect("local target"),
            vec![Value::function(
                foreign
                    .function_target(function(&foreign, "ReadCurrent"))
                    .expect("foreign target"),
                vec![],
            )],
        ),
    ];
    for value in values {
        assert_eq!(
            session
                .invoke(function(&image, "Touch"), vec![value.into()])
                .expect_err("complete callable graph is checked before invocation")
                .code,
            RUNTIME_INVALID_TASK
        );
    }
    assert_eq!(
        invoke(&mut session, &image, "ReadCurrent", vec![])
            .try_to_data()
            .expect("integer"),
        Value::Integer(10)
    );
}

#[test]
fn raw_local_cells_cannot_enter_even_through_a_pure_looking_container() {
    let image = image(SOURCE);
    let mut session = CallbackSession::from_shared(Arc::clone(&image));
    let domain = MemoryDomain::new();
    let cell = domain.new_cell(Value::Integer(7)).expect("local cell");
    let target = image
        .function_target(function(&image, "ReadCurrent"))
        .expect("target");
    for value in [
        Value::Cell(cell.clone()),
        Value::result_ok(Value::Array(vec![Value::Cell(cell.clone())].into())),
        Value::bound_function(target, Value::Cell(cell)),
    ] {
        assert_eq!(
            session
                .invoke(function(&image, "Touch"), vec![value.into()])
                .expect_err("raw local graph")
                .code,
            RUNTIME_INVALID_TASK
        );
    }
}

#[test]
fn references_never_reenter_and_raw_cells_need_an_export_even_in_the_owner_domain() {
    use fpas_bytecode::{ReferenceRoot, VariableReference};

    let image = empty_image();
    let domain = MemoryDomain::new();
    let context = HostValues::new(domain.clone());
    let cell = domain.new_cell(Value::Integer(7)).expect("owned cell");
    assert_eq!(
        context
            .admit(vec![Value::Cell(cell.clone()).into()], &image)
            .expect_err("raw data cannot assert local session provenance")
            .code,
        RUNTIME_INVALID_TASK
    );
    let local = exported(&context, Value::Cell(cell.clone()));
    assert!(context.admit(vec![local.into()], &image).is_ok());
    let reference = Value::Reference(Arc::new(VariableReference {
        root: ReferenceRoot::Cell(cell),
        path: vec![],
    }));
    let reference = exported(&context, Value::option_some(reference));
    assert_eq!(reference.try_to_data(), Err(HostValueError::SessionBound));
    assert_eq!(
        context
            .admit(vec![reference.into()], &image)
            .expect_err("even an owner export cannot extend a var reference's lifetime")
            .code,
        RUNTIME_INVALID_TASK
    );
}
