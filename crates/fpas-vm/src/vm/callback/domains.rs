//! Callback sessions keep local closures usable between ordinary invocations.

use super::*;
use fpas_diagnostics::codes::RUNTIME_INVALID_TASK;

fn executable() -> Arc<VerifiedExecutable> {
    let (program, errors) = fpas_parser::parse(
        r#"
program CallbackDomains;
function Counter(): function(): integer;
begin
  var Count := 0;
  return function(): integer begin
    Count := Count + 1;
    return Count;
  end function;
end function;
function Invoke(Next: function(): integer): integer;
begin
  return Next();
end function;
begin
end.
"#,
    );
    assert!(errors.is_empty(), "{errors:?}");
    Arc::new(fpas_compiler::compile(&program).expect("callback domain source"))
}

fn function(image: &VerifiedExecutable, name: &str) -> FunctionId {
    let index = image
        .executable()
        .functions
        .iter()
        .position(|info| image.executable().strings.get(info.name) == Some(name))
        .expect("callback function");
    FunctionId::try_from_index(index).expect("function id")
}

#[test]
fn reusable_session_preserves_mutable_closures_and_rejects_a_foreign_owner() {
    let image = executable();
    let make = function(&image, "counter");
    let invoke = function(&image, "invoke");
    let mut owner = CallbackSession::from_shared(Arc::clone(&image));
    let mut foreign = CallbackSession::from_shared(image);
    let counter = owner.invoke(make, vec![]).expect("create counter").value;
    for expected in [1, 2] {
        assert_eq!(
            owner
                .invoke(invoke, vec![counter.clone().into()])
                .expect("same session")
                .value
                .try_to_data()
                .expect("pure counter result"),
            Value::Integer(expected)
        );
    }
    assert_eq!(
        foreign
            .invoke(invoke, vec![counter.clone().into()])
            .expect_err("shared code does not grant cell ownership")
            .code,
        RUNTIME_INVALID_TASK
    );
    assert_eq!(
        owner
            .invoke(invoke, vec![counter.into()])
            .expect("owner remains usable")
            .value
            .try_to_data()
            .expect("pure counter result"),
        Value::Integer(3)
    );
}
