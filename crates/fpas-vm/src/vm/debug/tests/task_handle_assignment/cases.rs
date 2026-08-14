//! Session-level task-handle assignment coverage.

use super::super::*;
use super::child::*;
use super::fixtures::*;
use super::support::*;
use fpas_bytecode::Value;

fn runtime(session: &DebugSession, expression: &DebugExpression, frame: u64) -> Value {
    session
        .evaluate_runtime_value(expression, Some(frame), DebugEvaluationLimits::default())
        .expect("runtime value")
}

#[test]
fn visible_task_binding_copies_the_exact_runtime_id() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    stop_with_tasks(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let source = runtime(&session, &name("Pending"), frame);
    let locals = scope_reference(&mut session, "Locals");
    let updated = session
        .set_variable(locals, "Current", &name("Pending"))
        .expect("copy Pending");
    assert!(updated.value.starts_with("<task"), "{updated:?}");
    let frame = session.stack(0, 1).expect("fresh").items[0].id;
    let copied = runtime(&session, &name("Current"), frame);
    assert_eq!(copied, source);
    match copied {
        Value::Task(_) => {}
        other => panic!("expected task handle, got {}", other.type_name()),
    }
}

#[test]
fn uninitialized_local_and_global_roots_accept_one_task_handle() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    stop_with_tasks(&mut session);
    let locals = scope_reference(&mut session, "Locals");
    assert_eq!(
        named(
            &session.variables(locals, 0, 40).expect("locals").items,
            "Slot"
        )
        .value,
        "<uninitialized>"
    );
    assert!(
        session
            .set_variable(locals, "Slot", &name("Pending"))
            .expect("init local")
            .value
            .starts_with("<task")
    );
    let frame = session.stack(0, 1).expect("after local").items[0].id;
    session
        .set_expression(&root("G"), &name("Pending"), Some(frame))
        .expect("init global");
    let globals = scope_reference(&mut session, "Globals");
    assert!(
        named(
            &session.variables(globals, 0, 10).expect("globals").items,
            "G"
        )
        .value
        .starts_with("<task")
    );
}

#[test]
fn parameter_capture_and_descendant_destinations_keep_ownership_rules() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    stop_with_tasks(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    session
        .set_expression(&field("Box", "Job"), &name("Pending"), Some(frame))
        .expect("record field");
    let frame = session.stack(0, 1).expect("fresh").items[0].id;
    session
        .set_expression(
            &DebugAssignmentTarget {
                root: "Items".to_string(),
                selectors: vec![DebugAssignmentSelector::Index(DebugExpression::Integer(0))],
            },
            &name("Pending"),
            Some(frame),
        )
        .expect("array element");
    let frame = session.stack(0, 1).expect("fresh").items[0].id;
    session
        .set_expression(
            &DebugAssignmentTarget {
                root: "Scores".to_string(),
                selectors: vec![DebugAssignmentSelector::Index(DebugExpression::String(
                    "a".to_string(),
                ))],
            },
            &name("Pending"),
            Some(frame),
        )
        .expect("dictionary value");
    let frame = session.stack(0, 1).expect("fresh").items[0].id;
    session
        .set_expression(&field("Optional", "value"), &name("Pending"), Some(frame))
        .expect("option payload");
    let frame = session.stack(0, 1).expect("fresh").items[0].id;
    session
        .set_expression(&field("Outcome", "value"), &name("Pending"), Some(frame))
        .expect("result payload");
    let captures = scope_reference(&mut session, "Captures");
    session
        .set_variable(captures, "CellSlot", &name("Pending"))
        .expect("capture cell");
    let frame = session.stack(0, 1).expect("fresh").items[0].id;
    session
        .set_expression(&root("G"), &name("Pending"), Some(frame))
        .expect("init global source for helper");
    stopped(session.step_into().expect("enter helper"));
    let parameters = scope_reference(&mut session, "Parameters");
    session
        .set_variable(parameters, "Arg", &name("G"))
        .expect("mutable parameter");
}

#[test]
fn source_lookup_follows_lexical_shadowing_and_globals_only_frames() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    stop_with_tasks(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    session
        .set_expression(&root("Current"), &name("Shared"), Some(frame))
        .expect("local Shared");
    let missing = session
        .set_expression(&root("G"), &name("Pending"), None)
        .expect_err("globals-only cannot see Pending");
    assert_eq!(missing.kind, DebugErrorKind::UnknownName);
    let frame = session.stack(0, 1).expect("fresh").items[0].id;
    session
        .set_expression(&root("G"), &name("Pending"), Some(frame))
        .expect("frame selects local Pending for a global destination");
}

#[test]
fn incompatible_dynamic_and_unsupported_sources_are_rejected_atomically() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    stop_with_tasks(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let locals = scope_reference(&mut session, "Locals");
    let generation = locals;
    let before = named(
        &session.variables(generation, 0, 40).expect("before").items,
        "Current",
    )
    .value
    .clone();
    let mismatch = session
        .set_expression(&root("Current"), &name("Wrong"), Some(frame))
        .expect_err("result type");
    assert_eq!(mismatch.kind, DebugErrorKind::VariableValueType);
    assert!(mismatch.message.contains("task"), "{mismatch:?}");
    assert!(!mismatch.hint.contains("<task"), "{}", mismatch.hint);
    let dynamic = session
        .set_expression(&root("Current"), &name("Loose"), Some(frame))
        .expect_err("dynamic source");
    assert_eq!(dynamic.kind, DebugErrorKind::VariableValueType);
    assert!(dynamic.hint.contains("Dynamic"), "{}", dynamic.hint);
    let integer = session
        .set_expression(&root("Current"), &name("Number"), Some(frame))
        .expect_err("non-task");
    assert_eq!(integer.kind, DebugErrorKind::VariableValueType);
    let dest = session
        .set_expression(&root("Loose"), &name("Pending"), Some(frame))
        .expect_err("dynamic dest");
    assert_eq!(dest.kind, DebugErrorKind::VariableValueType);
    let unknown = session
        .set_expression(&root("Current"), &name("MissingName"), Some(frame))
        .expect_err("unknown");
    assert_eq!(unknown.kind, DebugErrorKind::UnknownName);
    let call = session
        .set_expression(
            &root("Current"),
            &DebugExpression::Call {
                callee: Box::new(name("Pending")),
                arguments: Vec::new(),
            },
            Some(frame),
        )
        .expect_err("call");
    assert_eq!(call.kind, DebugErrorKind::VariableValueType);
    let display = session
        .set_expression(
            &root("Current"),
            &DebugExpression::String("<task 1>".to_string()),
            Some(frame),
        )
        .expect_err("display");
    assert_eq!(display.kind, DebugErrorKind::VariableValueType);
    let numeric = session
        .set_expression(&root("Current"), &DebugExpression::Integer(1), Some(frame))
        .expect_err("numeric");
    assert_eq!(numeric.kind, DebugErrorKind::VariableValueType);
    assert_eq!(
        named(
            &session
                .variables(generation, 0, 40)
                .expect("preserved")
                .items,
            "Current"
        )
        .value,
        before
    );
}

#[test]
fn inactive_payloads_and_whole_aggregates_containing_tasks_remain_rejected() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    stop_with_tasks(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let transition = session
        .set_expression(
            &DebugAssignmentTarget {
                root: "Missing".to_string(),
                selectors: vec![
                    DebugAssignmentSelector::Field("Some".to_string()),
                    DebugAssignmentSelector::Field("value".to_string()),
                ],
            },
            &name("Pending"),
            Some(frame),
        )
        .expect_err("inactive payload");
    assert_eq!(transition.kind, DebugErrorKind::VariableValueType);
    let aggregate = session
        .set_expression(&root("Items"), &name("Items"), Some(frame))
        .expect_err("array of tasks");
    assert_eq!(aggregate.kind, DebugErrorKind::VariableValueType);
    let record = session
        .set_expression(&root("Box"), &name("Box"), Some(frame))
        .expect_err("record with task field");
    assert_eq!(record.kind, DebugErrorKind::VariableValueType);
}

#[test]
fn immutable_hidden_stale_and_limit_failures_keep_existing_errors() {
    let mut session = DebugSession::new(assignment_executable()).expect("debug session");
    stop_with_tasks(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let locals = scope_reference(&mut session, "Locals");
    assert_eq!(
        session
            .set_expression(&root("Frozen"), &name("Pending"), Some(frame))
            .expect_err("immutable")
            .kind,
        DebugErrorKind::VariableNotMutable
    );
    assert_eq!(
        session
            .set_expression(&root("Hidden"), &name("Pending"), Some(frame))
            .expect_err("hidden")
            .kind,
        DebugErrorKind::VariableTargetUnknown
    );
    let exhausted = session
        .set_expression_with_limits(
            &root("Current"),
            &name("Pending"),
            Some(frame),
            DebugEvaluationLimits {
                max_detached_values: 0,
                ..DebugEvaluationLimits::default()
            },
        )
        .expect_err("value limit");
    assert_eq!(exhausted.kind, DebugErrorKind::EvaluationLimit);
    session
        .set_variable(locals, "Current", &name("Pending"))
        .expect("success");
    assert_eq!(
        session
            .set_variable(locals, "Current", &name("Pending"))
            .expect_err("stale handle")
            .kind,
        DebugErrorKind::VariableTargetExpired
    );
}

#[test]
fn selected_child_task_mutation_stays_bound_to_that_request_context() {
    let mut session = DebugSession::new(child_task_executable()).expect("debug session");
    session
        .set_breakpoint(SourceBreakpoint {
            source: "test.fpas".to_string(),
            line: 21,
            column: None,
        })
        .expect("child breakpoint");
    let stop = stopped(session.continue_execution().expect("run to child"));
    assert_eq!(stop.task_id, 1);
    let child = session.stack_for_task(1, 0, 1).expect("child").items[0].id;
    let source = runtime(&session, &name("Backup"), child);
    session
        .set_expression(&root("Current"), &name("Backup"), Some(child))
        .expect("child copy");
    let child = session.stack_for_task(1, 0, 1).expect("fresh child").items[0].id;
    assert_eq!(runtime(&session, &name("Current"), child), source);
    let main = session.stack_for_task(0, 0, 1).expect("main").items[0].id;
    let main_current = runtime(&session, &name("Pending"), main);
    match main_current {
        Value::Task(_) => {}
        other => panic!("main still holds its own handle, got {}", other.type_name()),
    }
}

#[test]
fn consumed_handle_copy_does_not_consult_the_scheduler() {
    let mut session = DebugSession::new(consumed_executable()).expect("debug session");
    stop_with_tasks(&mut session);
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let source = runtime(&session, &name("Pending"), frame);
    session
        .set_expression(&root("Current"), &name("Pending"), Some(frame))
        .expect("copy consumed handle");
    let frame = session.stack(0, 1).expect("fresh").items[0].id;
    assert_eq!(runtime(&session, &name("Current"), frame), source);
}

#[test]
fn compiled_fixture_retains_portable_task_result_metadata() {
    const SOURCE: &str =
        include_str!("../../../../../../../tests/debugger/fixtures/task_handle_assignment.fpas");
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile task-handle fixture");
    let image = executable.executable();
    let binding_type = |name: &str| {
        let binding = image.functions[0]
            .debug
            .bindings
            .iter()
            .find(|binding| image.strings.get(binding.name) == Some(name))
            .expect("named task binding");
        image
            .debug_types
            .get(binding.ty.get() as usize)
            .expect("task debug type")
    };
    let current = binding_type("Current");
    let wrong = binding_type("Wrong");
    let (
        fpas_bytecode::DebugType::Task(current_result),
        fpas_bytecode::DebugType::Task(wrong_result),
    ) = (current, wrong)
    else {
        panic!("expected task bindings, got {current:?} and {wrong:?}");
    };
    assert_ne!(
        current_result, wrong_result,
        "integer and string task handles must retain distinct result types"
    );
    assert_eq!(
        image.debug_types.get(current_result.get() as usize),
        Some(&fpas_bytecode::DebugType::Integer)
    );
    assert_eq!(
        image.debug_types.get(wrong_result.get() as usize),
        Some(&fpas_bytecode::DebugType::String)
    );
}
