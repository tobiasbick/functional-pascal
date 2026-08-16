//! Function-breakpoint identity, replacement, and ordering contracts.

use super::*;

#[test]
fn function_breakpoint_binds_exact_identity_and_stops_at_first_sequence_point() {
    let mut session = DebugSession::new(call_executable()).expect("debug session");
    let bound = session
        .replace_function_breakpoints(vec![FunctionBreakpoint {
            name: "HELPER".to_string(),
        }])
        .expect("function breakpoint");

    assert_eq!(bound.len(), 1);
    assert_eq!(bound[0].functions, vec![FunctionId::new(1)]);
    assert_eq!(bound[0].instructions, vec![4]);
    assert_eq!(bound[0].locations[0].line, 10);

    let stop = stopped(session.continue_execution().expect("function stop"));
    assert_eq!(stop.reason, DebugStopReason::Breakpoint);
    assert_eq!(stop.breakpoint_ids, vec![bound[0].id]);
    assert_eq!(stop.instruction, 4);
}

#[test]
fn short_selector_binds_every_same_named_function_in_executable_order() {
    let executable = executable(
        vec![
            abc(Opcode::CallDirect, NO_REGISTER, 1, 0),
            abc(Opcode::CallDirect, NO_REGISTER, 2, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
        ],
        vec![
            function("root", 0, 3, 0, debug(&[(0, 1), (1, 2)])),
            function("helper", 3, 4, 0, debug(&[(3, 10)])),
            function("helper", 4, 5, 0, debug(&[(4, 20)])),
        ],
        Vec::new(),
        vec![(0, 1), (1, 2), (3, 10), (4, 20)],
    );
    let mut session = DebugSession::new(executable).expect("debug session");
    let bound = session
        .replace_function_breakpoints(vec![FunctionBreakpoint {
            name: "helper".to_string(),
        }])
        .expect("multi-match breakpoint");

    assert_eq!(
        bound[0].functions,
        vec![FunctionId::new(1), FunctionId::new(2)]
    );
    assert_eq!(bound[0].instructions, vec![3, 4]);

    let first = stopped(session.continue_execution().expect("first helper"));
    assert_eq!(first.breakpoint_ids, vec![bound[0].id]);
    let second = stopped(session.continue_execution().expect("second helper"));
    assert_eq!(second.breakpoint_ids, vec![bound[0].id]);
}

#[test]
fn missing_and_no_entry_selectors_remain_unverified() {
    let executable = executable(
        vec![
            abc(Opcode::Return, NO_REGISTER, 0, 0),
            abc(Opcode::Return, NO_REGISTER, 0, 0),
        ],
        vec![
            function("root", 0, 1, 0, debug(&[(0, 1)])),
            function("helper", 1, 2, 0, debug(&[])),
        ],
        Vec::new(),
        vec![(0, 1), (1, 10)],
    );
    let mut session = DebugSession::new(executable).expect("debug session");
    let bound = session
        .replace_function_breakpoints(vec![
            FunctionBreakpoint {
                name: "missing".to_string(),
            },
            FunctionBreakpoint {
                name: "helper".to_string(),
            },
        ])
        .expect("unverified breakpoints");

    assert!(bound[0].functions.is_empty());
    assert!(!bound[0].is_verified());
    assert_eq!(bound[1].functions, vec![FunctionId::new(1)]);
    assert!(!bound[1].is_verified());
}

#[test]
fn oversized_replace_is_atomic_and_preserves_existing_function_breakpoint() {
    let mut session = DebugSession::new(call_executable()).expect("debug session");
    let previous = session
        .replace_function_breakpoints(vec![FunctionBreakpoint {
            name: "helper".to_string(),
        }])
        .expect("initial function breakpoint");
    let limit = session.breakpoint_limits().max_breakpoints;
    let error = session
        .replace_function_breakpoints(
            (0..=limit)
                .map(|index| FunctionBreakpoint {
                    name: format!("missing{index}"),
                })
                .collect(),
        )
        .expect_err("logical breakpoint limit");

    assert_eq!(error.kind, DebugErrorKind::BreakpointLimit);
    let stop = stopped(session.continue_execution().expect("preserved breakpoint"));
    assert_eq!(stop.breakpoint_ids, vec![previous[0].id]);
}

#[test]
fn source_and_function_breakpoints_share_one_ordered_stop() {
    let mut session = DebugSession::new(call_executable()).expect("debug session");
    let source = session
        .set_breakpoint(SourceBreakpoint {
            source: "test.fpas".to_string(),
            line: 10,
            column: None,
        })
        .expect("source breakpoint");
    let function = session
        .replace_function_breakpoints(vec![FunctionBreakpoint {
            name: "helper".to_string(),
        }])
        .expect("function breakpoint");

    let stop = stopped(session.continue_execution().expect("shared stop"));
    assert_eq!(stop.breakpoint_ids, vec![source.id, function[0].id]);
    assert_eq!(stop.breakpoint_id, Some(source.id));
}

#[test]
fn recursive_and_task_entries_reuse_one_logical_breakpoint_id() {
    let mut recursive = DebugSession::new(recursive_executable()).expect("recursive session");
    let recursive_breakpoint = recursive
        .replace_function_breakpoints(vec![FunctionBreakpoint {
            name: "root".to_string(),
        }])
        .expect("recursive breakpoint")[0]
        .id;
    let first = stopped(recursive.continue_execution().expect("initial root entry"));
    let second = stopped(
        recursive
            .continue_execution()
            .expect("recursive root entry"),
    );
    assert_eq!(first.breakpoint_ids, vec![recursive_breakpoint]);
    assert_eq!(second.breakpoint_ids, vec![recursive_breakpoint]);
    assert!(second.call_depth > first.call_depth);

    let mut task = DebugSession::new(task_executable()).expect("task session");
    let task_breakpoint = task
        .replace_function_breakpoints(vec![FunctionBreakpoint {
            name: "helper".to_string(),
        }])
        .expect("task breakpoint")[0]
        .id;
    let task_stop = stopped(task.continue_execution().expect("task entry"));
    assert_eq!(task_stop.breakpoint_ids, vec![task_breakpoint]);
    assert_eq!(task_stop.task_id, 1);
}

#[test]
fn selector_and_physical_binding_limits_reject_atomically() {
    let function_count = DebugBreakpointLimits::default().max_function_bindings + 1;
    let mut code = vec![
        abc(Opcode::LoadUnit, 0, 0, 0),
        abc(Opcode::Return, NO_REGISTER, 0, 0),
    ];
    let mut functions = vec![function("root", 0, 2, 1, debug(&[(0, 1)]))];
    let mut runs = vec![(0, 1)];
    for index in 0..function_count {
        let start = u32::try_from(code.len()).expect("function address");
        code.push(abc(Opcode::Return, NO_REGISTER, 0, 0));
        functions.push(function(
            "many",
            start,
            start + 1,
            0,
            debug(&[(start, u32::try_from(index + 2).expect("line"))]),
        ));
        runs.push((start, u32::try_from(index + 2).expect("line")));
    }
    let executable = executable(code, functions, Vec::new(), runs);
    let mut session = DebugSession::new(executable).expect("bounded session");
    let previous = session
        .replace_function_breakpoints(vec![FunctionBreakpoint {
            name: "root".to_string(),
        }])
        .expect("initial breakpoint")[0]
        .id;

    for rejected in [
        FunctionBreakpoint {
            name: "helper".to_string(),
        },
        FunctionBreakpoint {
            name: "x".repeat(session.breakpoint_limits().max_function_name_bytes + 1),
        },
    ] {
        let error = session
            .replace_function_breakpoints(vec![rejected])
            .expect_err("bounded replacement rejection");
        assert_eq!(error.kind, DebugErrorKind::BreakpointLimit);
    }

    session
        .clear_breakpoint(previous)
        .expect("rejected replacements preserve the previous logical breakpoint");
    assert_eq!(
        session
            .clear_breakpoint(previous)
            .expect_err("cleared breakpoint is absent")
            .kind,
        DebugErrorKind::UnknownBreakpoint
    );
}
