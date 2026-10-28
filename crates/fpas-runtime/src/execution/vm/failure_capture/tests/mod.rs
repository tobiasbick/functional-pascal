//! Observable original-cause and ownership contracts for normal VM execution.

mod catalog;
mod lifetime;
mod ownership;
mod termination;

use crate::execution::vm::{FailureFrame, FailureObservation, FailureScalar, Vm};
use std::sync::{Arc, Mutex};

fn vm(source: &str) -> Vm {
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    Vm::new(fpas_compiler::compile(&program).expect("fixture compiles"))
}

fn observe(vm: &mut Vm) -> Arc<Mutex<Vec<Arc<FailureObservation>>>> {
    let observations = Arc::new(Mutex::new(Vec::new()));
    let output = Arc::clone(&observations);
    assert!(vm.set_failure_observer(move |data| output.lock().unwrap().push(data)));
    observations
}

fn scalar<'a>(frame: &'a FailureFrame, name: &str) -> &'a Option<FailureScalar> {
    &frame
        .bindings
        .iter()
        .find(|binding| binding.name.eq_ignore_ascii_case(name))
        .unwrap_or_else(|| panic!("missing {name} in {frame:?}"))
        .value
        .scalar
}

#[test]
fn nested_root_fault_retains_parameters_and_callers_after_worker_disposal() {
    let source = "program Original;\nprocedure Leaf(Number: integer);\nbegin\n const Local := 'kept';\n panic('original');\nend procedure;\nprocedure Middle(Argument: integer);\nbegin\n const Caller := 19;\n Leaf(Argument); panic('unreachable');\nend procedure;\nbegin Middle(7); panic('unreachable'); end.";
    for pool_size in [0, 1] {
        let mut plain = vm(source);
        plain.pool_size = pool_size;
        let expected = plain.run().expect_err("ordinary panic");
        let mut enabled = vm(source);
        enabled.pool_size = pool_size;
        let output = observe(&mut enabled);
        let error = enabled.run().expect_err("observed panic");
        assert_eq!(
            error, expected,
            "capture must preserve the ordinary diagnostic"
        );
        let output = output.lock().unwrap();
        assert_eq!(output.len(), 1);
        let data = &output[0];
        assert_eq!(data.error.diagnostic, error);
        assert_eq!(data.cause.task_id, Some(0));
        assert_eq!(data.cause.total_frames, Some(3));
        assert_eq!(data.cause.reason, None);
        let catalog = data.task_catalog.as_ref().unwrap();
        assert_eq!(catalog.tasks.len(), 1);
        assert_eq!(catalog.tasks[0].id, 0);
        assert_eq!(catalog.tasks[0].observation, "fault");
        assert!(Arc::ptr_eq(
            catalog.tasks[0].snapshot.as_ref().unwrap(),
            &data.cause
        ));
        assert_eq!(
            scalar(&data.cause.frames[0], "Number"),
            &Some(FailureScalar::Integer("7".into()))
        );
        assert_eq!(
            scalar(&data.cause.frames[0], "Local"),
            &Some(FailureScalar::String("kept".into()))
        );
        assert_eq!(
            scalar(&data.cause.frames[1], "Caller"),
            &Some(FailureScalar::Integer("19".into()))
        );
        assert_eq!(
            data.cause
                .frames
                .iter()
                .map(|frame| frame.location.unwrap().line)
                .collect::<Vec<_>>(),
            [5, 10, 12]
        );
    }
}

#[test]
fn nested_hosted_callbacks_retain_each_real_fpas_continuation_once() {
    let mut vm = vm(
        "program Callback;\nfunction Fail(Item: integer): integer; begin const Before := 23; panic('callback'); end function;\nfunction MapInner(Item: integer): integer; begin const Mapped := [Item].Map(Fail); return Mapped[0]; end function;\nprocedure Caller(); begin const Values := [7].Map(MapInner); end procedure;\nbegin Caller(); panic('unreachable'); end.",
    );
    let output = observe(&mut vm);
    vm.run().expect_err("callback failure");
    let output = output.lock().unwrap();
    let frames = &output[0].cause.frames;
    assert_eq!(frames.len(), 4, "{frames:?}");
    assert_eq!(
        frames
            .iter()
            .map(|frame| frame.function.to_ascii_lowercase())
            .collect::<Vec<_>>(),
        ["fail", "mapinner", "caller", "callback"]
    );
    assert_eq!(
        scalar(&frames[0], "Item"),
        &Some(FailureScalar::Integer("7".into()))
    );
    assert_eq!(
        scalar(&frames[0], "Before"),
        &Some(FailureScalar::Integer("23".into()))
    );
}
