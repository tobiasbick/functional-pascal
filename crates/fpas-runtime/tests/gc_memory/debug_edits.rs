//! Repeated debugger edits balance memory while stopped and after session teardown.

use crate::measurement::{Footprint, completed_threads, steady_state};
use crate::workloads::{image, schedule};
use fpas_runtime::execution::{
    DebugAssignmentTarget, DebugErrorKind, DebugEvaluationLimits, DebugExpression, DebugFrame,
    DebugRunResult, DebugScopeKind, DebugSession, SourceBreakpoint,
};

const SOURCE: &str = include_str!("fixtures/debugger_edits.fpas");

fn call(name: &str) -> DebugExpression {
    DebugExpression::Call {
        callee: Box::new(DebugExpression::Callable(name.into())),
        arguments: Vec::new(),
    }
}

fn target(name: &str) -> DebugAssignmentTarget {
    DebugAssignmentTarget {
        root: name.into(),
        selectors: Vec::new(),
    }
}

fn frame(session: &DebugSession, name: &str) -> DebugFrame {
    session
        .stack(0, 16)
        .unwrap()
        .items
        .into_iter()
        .find(|frame| frame.name.eq_ignore_ascii_case(name))
        .unwrap_or_else(|| panic!("missing frame {name}"))
}

fn locals(session: &DebugSession, frame: u64) -> u64 {
    session
        .scopes(frame)
        .unwrap()
        .into_iter()
        .find(|scope| scope.kind == DebugScopeKind::Locals)
        .unwrap()
        .variables_reference
}

fn stop(session: &mut DebugSession) {
    assert!(matches!(
        session.continue_execution().unwrap(),
        DebugRunResult::Stopped(_)
    ));
}

fn inspect(session: &mut DebugSession, frame: u64, name: &str) -> u64 {
    let value = session
        .evaluate(&DebugExpression::Name(name.into()), Some(frame))
        .unwrap();
    assert_ne!(value.variables_reference, 0, "{value:?}");
    assert!(
        !session
            .variables(value.variables_reference, 0, 2)
            .unwrap()
            .items
            .is_empty()
    );
    value.variables_reference
}

fn bind(session: &mut DebugSession, marker: &str) -> u64 {
    let line = SOURCE
        .lines()
        .position(|line| line.contains(marker))
        .unwrap()
        + 1;
    let breakpoint = session
        .set_breakpoint(SourceBreakpoint {
            source: "<memory>".into(),
            line: line as u32,
            column: None,
        })
        .unwrap();
    assert!(breakpoint.instruction.is_some());
    breakpoint.id
}

fn warm_inspection_registry(session: &mut DebugSession) {
    let root = frame(session, "DebuggerEditMemory").id;
    // Expand live variables: evaluate() would detach each result into a separate
    // domain and leave the stopped worker's weak-root registry unwarmed.
    let reference = locals(session, root);
    let warmers = session
        .variables(reference, 0, 16)
        .unwrap()
        .items
        .into_iter()
        .find(|value| value.name.eq_ignore_ascii_case("InspectionRoots"))
        .unwrap();
    let children = session
        .variables(warmers.variables_reference, 0, 128)
        .unwrap();
    assert_eq!(children.items.len(), 128);
    assert!(
        children
            .items
            .iter()
            .all(|child| child.variables_reference != 0)
    );
    // These handles exceed the failed-call root peak. Drop them before measuring
    // so bounded HashMap growth from tombstone churn cannot mimic retained data.
    drop(children);
    session
        .set_expression(
            &target("Done"),
            &DebugExpression::Boolean(false),
            Some(root),
        )
        .unwrap();
}

pub fn restarts() {
    let executable = image(SOURCE);
    for config in [fpas_runtime::CollectionSchedule::default(), schedule()] {
        for disconnect in [false, true] {
            completed_threads("debug restart teardown", 2, || {
                let mut session =
                    DebugSession::with_args(executable.clone(), vec!["restart".into()])
                        .unwrap()
                        .with_collection_schedule(config);
                let breakpoint = bind(&mut session, "const Check := Kept()");
                steady_state("debug live frame restart", 32, || {
                    stop(&mut session);
                    let leaf = frame(&session, "Leaf");
                    let selected = frame(&session, "Restartable");
                    let reference = inspect(&mut session, leaf.id, "Inner");
                    let before = Footprint::now();
                    assert_eq!(
                        session.restart_frame(u64::MAX).unwrap_err().kind,
                        DebugErrorKind::UnknownFrame
                    );
                    assert_eq!(
                        Footprint::now(),
                        before,
                        "rejected restart retained storage"
                    );
                    assert_eq!(frame(&session, "Leaf").id, leaf.id);
                    let restarted = session.restart_frame(selected.id).unwrap();
                    assert_eq!(restarted.discarded_frames, 1);
                    assert_eq!(restarted.frame.name, "restartable");
                    assert_eq!(
                        session.restart_frame(selected.id).unwrap_err().kind,
                        DebugErrorKind::UnknownFrame
                    );
                    assert_eq!(
                        session.variables(reference, 0, 1).unwrap_err().kind,
                        DebugErrorKind::UnknownVariablesReference
                    );
                });
                if disconnect {
                    session.disconnect();
                } else {
                    session.clear_breakpoint(breakpoint).unwrap();
                    assert!(matches!(
                        session.continue_execution().unwrap(),
                        DebugRunResult::Terminated(_)
                    ));
                }
            });
        }
    }
}

pub fn forced_returns() {
    let executable = image(SOURCE);
    let replacement = call("NewData");
    let wrong = call("WrongReturn");
    let limits = DebugEvaluationLimits {
        max_output_bytes: 0,
        ..DebugEvaluationLimits::default()
    };
    for config in [fpas_runtime::CollectionSchedule::default(), schedule()] {
        completed_threads("debug forced-return teardown", 2, || {
            let mut session = DebugSession::with_args(executable.clone(), vec!["return".into()])
                .unwrap()
                .with_collection_schedule(config);
            bind(&mut session, "const Check := Kept()");
            stop(&mut session);
            warm_inspection_registry(&mut session);
            let selected = frame(&session, "Returnable").id;
            let leaf = frame(&session, "Leaf").id;
            steady_state("rejected return preparation", 32, || {
                assert_eq!(
                    session
                        .force_return(selected, Some(&wrong))
                        .unwrap_err()
                        .kind,
                    DebugErrorKind::FrameReturnType
                );
                assert_eq!(
                    session
                        .force_return_with_limits(selected, Some(&replacement), limits)
                        .unwrap_err()
                        .kind,
                    DebugErrorKind::EvaluationLimit
                );
                assert_eq!(frame(&session, "Returnable").id, selected);
                assert_eq!(frame(&session, "Leaf").id, leaf);
            });
            // Establish the caller-only checkpoint, where returned payloads and
            // inspection handles have been released, before measuring repetitions.
            finish_return(&mut session, &replacement);
            steady_state("debug live forced returns", 32, || {
                stop(&mut session);
                finish_return(&mut session, &replacement);
            });
            finish_run(&mut session);
        });
    }
}

fn finish_return(session: &mut DebugSession, replacement: &DebugExpression) {
    let selected = frame(session, "Returnable");
    inspect(session, frame(session, "Leaf").id, "Inner");
    let returned = session
        .force_return(selected.id, Some(replacement))
        .unwrap();
    assert_eq!(returned.unwound_frames, 2);
    assert_eq!(returned.frame.as_ref().unwrap().name, "returnround");
    assert!(!returned.terminated);
    assert_eq!(returned.indexed_variables, 8192);
    assert_eq!(
        session
            .variables(returned.variables_reference, 0, 1)
            .unwrap()
            .items[0]
            .value,
        "23"
    );
    assert_eq!(
        session.force_return(selected.id, None).unwrap_err().kind,
        DebugErrorKind::UnknownFrame
    );
    assert!(matches!(
        session.step_out().unwrap(),
        DebugRunResult::Stopped(_)
    ));
    assert_eq!(
        session.stack(0, 1).unwrap().items[0].name,
        "debuggereditmemory"
    );
}

fn finish_run(session: &mut DebugSession) {
    let root = session.stack(0, 1).unwrap().items[0].id;
    session
        .set_expression(&target("Done"), &DebugExpression::Boolean(true), Some(root))
        .unwrap();
    // Let the program check its surviving caller alias outside detached evaluation.
    assert!(matches!(
        session.continue_execution().unwrap(),
        DebugRunResult::Terminated(_)
    ));
}

pub fn mutations() {
    let executable = image(SOURCE);
    let replacement = call("NewData");
    let limits = DebugEvaluationLimits {
        max_output_bytes: 0,
        ..DebugEvaluationLimits::default()
    };
    for config in [fpas_runtime::CollectionSchedule::default(), schedule()] {
        completed_threads("debug mutation teardown", 2, || {
            let mut session = DebugSession::with_args(executable.clone(), vec!["mutate".into()])
                .unwrap()
                .with_collection_schedule(config);
            bind(&mut session, "const Marker := Number");
            stop(&mut session);
            warm_inspection_registry(&mut session);
            let selected = frame(&session, "MutationRound").id;
            steady_state("rejected mutation preparation", 32, || {
                assert_eq!(
                    session
                        .set_expression(&target("Number"), &replacement, Some(selected))
                        .unwrap_err()
                        .kind,
                    DebugErrorKind::VariableValueType
                );
                assert_eq!(
                    session
                        .set_expression_with_limits(
                            &target("Items"),
                            &replacement,
                            Some(selected),
                            limits
                        )
                        .unwrap_err()
                        .kind,
                    DebugErrorKind::EvaluationLimit
                );
                assert_eq!(frame(&session, "MutationRound").id, selected);
            });
            let unchanged = session
                .evaluate(
                    &DebugExpression::Index {
                        base: Box::new(DebugExpression::Name("Items".into())),
                        index: Box::new(DebugExpression::Integer(0)),
                    },
                    Some(selected),
                )
                .unwrap();
            assert_eq!(unchanged.value, "11");
            finish_mutation(&mut session, &replacement);
            steady_state("debug live variable and expression edits", 32, || {
                stop(&mut session);
                finish_mutation(&mut session, &replacement);
            });
            finish_run(&mut session);
        });
    }
}

fn finish_mutation(session: &mut DebugSession, replacement: &DebugExpression) {
    let selected = frame(session, "MutationRound");
    let reference = locals(session, selected.id);
    inspect(session, selected.id, "Current");
    let updated = session
        .set_variable(reference, "Current", &DebugExpression::Name("Other".into()))
        .unwrap();
    assert!(updated.value.starts_with("<function"));
    assert_eq!(
        session
            .set_variable(reference, "Current", &DebugExpression::Name("Other".into()))
            .unwrap_err()
            .kind,
        DebugErrorKind::VariableTargetExpired
    );
    let selected = frame(session, "MutationRound");
    let updated = session
        .set_expression(&target("Items"), replacement, Some(selected.id))
        .unwrap();
    assert_eq!(updated.indexed_variables, 8192);
    assert_eq!(
        session
            .variables(updated.variables_reference, 0, 1)
            .unwrap()
            .items[0]
            .value,
        "23"
    );
    assert!(matches!(
        session.step_out().unwrap(),
        DebugRunResult::Stopped(_)
    ));
    assert_eq!(
        session.stack(0, 1).unwrap().items[0].name,
        "debuggereditmemory"
    );
}
