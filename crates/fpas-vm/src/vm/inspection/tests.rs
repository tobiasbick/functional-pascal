//! Normal-worker lifecycle coverage that all-stop debugger fixtures cannot provide.

use std::collections::HashSet;
use std::sync::Arc;

use super::{
    bindings,
    frames::{self, Boundary},
    globals, location,
    values::{
        ValueLimits,
        budget::ValueBudget,
        render::RetainedValue,
        snapshot::{self, Scalar, ValueSnapshot},
    },
};
use crate::vm::worker::Worker;
use fpas_bytecode::{DebugBindingKind, Value};

const LIMITS: ValueLimits = ValueLimits {
    max_children: 256,
    max_string_chars: 4096,
    max_summary_chars: 4096,
    max_depth: 16,
};

fn worker(source: &str) -> Worker {
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    Worker::new(Arc::new(fpas_compiler::compile(&program).expect("compile")))
        .expect("worker")
        .with_scheduler(Some(Arc::new(crate::vm::tasks::TaskScheduler::new())))
}

fn detached(value: bindings::ReadValue) -> ValueSnapshot {
    let (value, reason) = value.into_parts();
    let retained = RetainedValue {
        name: String::new(),
        type_name: value
            .as_ref()
            .map_or("unknown", Value::type_name)
            .to_string(),
        value,
        presentation_hint: reason.map(|reason| format!("unavailable: {reason}")),
        depth: 0,
        visited_cells: HashSet::new(),
        debug_type: None,
        origin: (),
    };
    snapshot::capture(
        &retained,
        LIMITS,
        None,
        &mut ValueBudget::new(16384, 8 * 1024 * 1024),
    )
    .expect("snapshot")
}

#[test]
fn ordinary_fault_reads_visible_bindings_references_and_worker_local_globals() {
    let source = "program Inspect;\nvar Global: integer := 7;\nprocedure Fail(var Item: integer);\nbegin\n  const Before := 5;\n  begin\n    const Before := 9;\n    Item := Item + Before;\n    panic('fault');\n  end;\nend procedure;\nbegin\n  var Local := 3;\n  Fail(var Local);\n  Global := Local;\nend.";
    let mut worker = worker(source).with_local_globals(true);
    let error = worker.run_task().expect_err("runtime panic");
    assert_eq!(
        error.code,
        fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC,
        "{error:?}"
    );
    assert_eq!(
        detached(globals::read(&worker, 0)).scalar,
        Some(Scalar::Integer("7".to_string()))
    );
    assert!(
        worker.globals.read().expect("shared globals")[0].is_none(),
        "the shared copy must not accidentally supply the value"
    );
    let mut inspected = Vec::new();
    let total = frames::visit(&worker, Boundary::Fault, 256, |owner, frame, _| {
        let image = owner.executable.executable();
        let function = &image.functions[usize::from(frame.function.get())];
        let position =
            location::position_at(image, frame.function, frame.instruction).expect("position");
        let scopes = location::visible_scope_chain(
            function,
            location::scope_at(image, frame.function, frame.instruction).expect("scope"),
        );
        let values = scopes
            .iter()
            .flat_map(|scope| {
                function
                    .debug
                    .bindings
                    .iter()
                    .filter(move |binding| !binding.hidden && binding.scope == *scope)
            })
            .map(|binding| {
                (
                    image
                        .strings
                        .get(binding.name)
                        .expect("binding name")
                        .to_ascii_lowercase(),
                    detached(bindings::read(owner, frame.base, binding)),
                )
            })
            .collect::<Vec<_>>();
        inspected.push((position.line, values));
    });
    assert_eq!(total.total, Some(2));
    assert_eq!(total.unavailable_reason, None);
    assert_eq!(
        inspected.iter().map(|(line, _)| *line).collect::<Vec<_>>(),
        [9, 14]
    );
    let values = &inspected[0].1;
    assert_eq!(
        values
            .iter()
            .find(|(name, _)| name == "before")
            .expect("inner binding")
            .1
            .scalar,
        Some(Scalar::Integer("9".to_string()))
    );
    assert_eq!(
        values
            .iter()
            .find(|(name, _)| name == "item")
            .expect("var parameter")
            .1
            .scalar,
        Some(Scalar::Integer("12".to_string()))
    );
    assert_eq!(
        inspected[1]
            .1
            .iter()
            .find(|(name, _)| name == "local")
            .expect("caller cell")
            .1
            .scalar,
        Some(Scalar::Integer("12".to_string()))
    );
}

#[test]
fn hosted_callback_faults_keep_the_callback_and_calling_continuation() {
    let source = "program Callback;\n\nfunction Fail(Item: integer): integer;\nbegin\n  panic('callback');\nend function;\nbegin\n  const Items := [7];\n  const Mapped := Items.Map(Fail);\nend.";
    for task_id in [0, 1] {
        let mut worker = worker(source);
        worker.task_id = task_id;
        let error = worker.run_task().expect_err("callback failure");
        assert_eq!(
            error.code,
            fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC,
            "{error:?}"
        );
        let mut lines = Vec::new();
        let total = frames::visit(&worker, Boundary::Fault, 256, |owner, frame, _| {
            let image = owner.executable.executable();
            lines.push(
                location::position_at(image, frame.function, frame.instruction)
                    .expect("position")
                    .line,
            );
            if lines.len() == 1 {
                let parameter = image.functions[usize::from(frame.function.get())]
                    .debug
                    .bindings
                    .iter()
                    .find(|binding| binding.kind == DebugBindingKind::Parameter)
                    .expect("callback parameter");
                assert_eq!(
                    detached(bindings::read(owner, frame.base, parameter)).scalar,
                    Some(Scalar::Integer("7".to_string()))
                );
            }
        });
        assert_eq!(total.total, Some(2), "task {task_id}");
        assert_eq!(lines, [5, 9], "task {task_id}");
        let mut bounded = 0;
        assert_eq!(
            frames::visit(&worker, Boundary::Fault, 1, |_, _, _| bounded += 1).total,
            Some(2)
        );
        assert_eq!(bounded, 1);
        if task_id == 0 {
            let _guard = worker.callback_worker.borrow_mut();
            let blocked = frames::visit(&worker, Boundary::Fault, 256, |_, _, _| {
                panic!("unavailable callback must not invent an active caller frame")
            });
            assert_eq!(
                (blocked.total, blocked.unavailable_reason),
                (None, Some("busy"))
            );
        }
    }
}

#[test]
fn global_contention_and_empty_storage_have_different_statuses() {
    let worker = worker("program Empty; var Item: integer := 1; begin end.");
    assert_eq!(detached(globals::read(&worker, 0)).status, "uninitialized");
    let guard = worker.globals.write().expect("exclusive global access");
    let busy = detached(globals::read(&worker, 0));
    assert_eq!((busy.status, busy.reason), ("unavailable", Some("busy")));
    drop(guard);
    assert_eq!(
        detached(globals::read(&worker, usize::MAX)).reason,
        Some("storage_unavailable")
    );
    let cell = Arc::new(std::sync::Mutex::new(Value::Integer(8)));
    let reference = fpas_bytecode::VariableReference {
        root: fpas_bytecode::ReferenceRoot::Cell(Arc::clone(&cell)),
        path: Vec::new(),
    };
    let guard = cell.lock().expect("hold referent");
    assert_eq!(
        detached(bindings::read_reference(&worker, &reference)).reason,
        Some("busy")
    );
    drop(guard);
    assert_eq!(
        detached(bindings::read_reference(&worker, &reference)).scalar,
        Some(Scalar::Integer("8".to_string()))
    );
}

#[test]
fn failing_escaped_closures_expose_the_current_capture_cell() {
    let mut worker = worker(
        "program Captured; function Make(): function(): integer; function Fail(): integer; begin Captured := Captured + 2; panic('capture'); end function; begin var Captured := 40; return Fail; end function; begin const Action := Make(); const Value := Action(); end.",
    );
    let error = worker.run_task().expect_err("closure failure");
    assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC);
    let mut captured = None;
    frames::visit(&worker, Boundary::Fault, 1, |owner, frame, _| {
        let image = owner.executable.executable();
        let binding = image.functions[usize::from(frame.function.get())]
            .debug
            .bindings
            .iter()
            .find(|binding| binding.kind == DebugBindingKind::Capture && !binding.hidden)
            .expect("capture metadata");
        captured = Some(detached(bindings::read(owner, frame.base, binding)));
    });
    assert_eq!(
        captured.expect("captured value").scalar,
        Some(Scalar::Integer("42".to_string()))
    );
}
