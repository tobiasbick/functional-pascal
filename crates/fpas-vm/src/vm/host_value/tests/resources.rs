use super::*;
use fpas_bytecode::{RecordTypeId, RuntimeRecordLayout, SharedRecord};

fn record(name: &str, value: Value) -> Value {
    Value::Record(SharedRecord::new(
        Arc::new(RuntimeRecordLayout {
            record: RecordTypeId::new(0),
            type_name: name.into(),
            fields: vec!["slot".into()],
        }),
        vec![value],
    ))
}

#[test]
fn raw_and_foreign_resource_graphs_are_rejected_even_with_matching_numeric_ids() {
    let image = empty_image();
    let first = HostValues::new(MemoryDomain::new());
    let second = HostValues::new(MemoryDomain::new());
    for leaf in [
        Value::OpaqueHandle(1),
        Value::Task(1),
        record("Std.Http.Handles.BodyStream", Value::Integer(1)),
        record("std.http.handles.ssedecoder", Value::Integer(1)),
    ] {
        for value in [
            leaf.clone(),
            Value::option_some(Value::Array(vec![leaf.clone()].into())),
            Value::dict(vec![(leaf.clone(), Value::Unit)]),
            record("Container", Value::result_ok(leaf.clone())),
        ] {
            assert_eq!(
                first
                    .admit(vec![value.clone().into()], &image)
                    .expect_err("raw resources have no session origin")
                    .code,
                RUNTIME_INVALID_TASK
            );
            let local = exported(&first, value);
            assert_eq!(local.try_to_data(), Err(HostValueError::SessionBound));
            assert!(first.admit(vec![local.clone().into()], &image).is_ok());
            assert_eq!(
                second
                    .admit(vec![local.into()], &image)
                    .expect_err("equal numeric resource IDs do not grant ownership")
                    .code,
                RUNTIME_INVALID_TASK
            );
        }
    }
}

#[test]
fn resource_hidden_in_callable_capture_or_bound_receiver_is_not_raw_data() {
    let image = empty_image();
    let context = HostValues::new(MemoryDomain::new());
    let target = image
        .function_target(image.executable().entry)
        .expect("entry");
    for value in [
        Value::function(target.clone(), vec![Value::OpaqueHandle(1)]),
        Value::bound_function(target, Value::Task(1)),
    ] {
        assert_eq!(
            context
                .admit(vec![value.into()], &image)
                .expect_err("callable edges are part of the resource graph")
                .code,
            RUNTIME_INVALID_TASK
        );
    }
}

#[test]
fn callback_wait_case_can_roundtrip_after_close_without_tombstones() {
    let image = image(
        r#"
program HostWaitCase;
uses Std.Tasks as Tasks;
function NewCase(): Tasks.WaitCase;
begin
  var Seen := 0;
  return Tasks.TimerCase(0, procedure() begin Seen := Seen + 1; end procedure);
end function;
function CloseCase(CaseValue: Tasks.WaitCase): boolean;
begin
  return Tasks.CloseWaitCase(CaseValue);
end function;
begin
end.
"#,
    );
    let mut first = CallbackSession::from_shared(Arc::clone(&image));
    let mut second = CallbackSession::from_shared(Arc::clone(&image));
    let case = invoke(&mut first, &image, "NewCase", vec![]);
    assert_eq!(case.try_to_data(), Err(HostValueError::SessionBound));
    for expected in [true, false] {
        assert_eq!(
            invoke(&mut first, &image, "CloseCase", vec![case.clone().into()])
                .try_to_data()
                .expect("close outcome"),
            Value::Boolean(expected)
        );
    }
    assert_eq!(
        second
            .invoke(function(&image, "CloseCase"), vec![case.into()])
            .expect_err("closed handle still retains foreign origin")
            .code,
        RUNTIME_INVALID_TASK
    );
}
