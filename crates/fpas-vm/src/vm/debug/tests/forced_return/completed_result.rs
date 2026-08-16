//! Replacement of unconsumed retained task results.

use super::fixtures::*;

#[test]
fn completed_retained_result_replacement_is_typed_repeatable_and_consumption_bound() {
    let mut session = DebugSession::new(spawn_value_task_executable()).expect("debug session");
    let _compute = stop_in_callee(&mut session, "compute");
    let _ = stopped(session.step_into_task(1).expect("child entry"));
    let _ = stopped(session.step_into_task(1).expect("child completion"));
    session.select_task(0).expect("root inspection");
    let root = session.stack(0, 1).expect("root stack").items[0].id;
    let instructions = session.test_instruction_count();

    let error = session
        .replace_completed_task_result(1, Some(root), Some(&DebugExpression::String("nope".into())))
        .expect_err("type mismatch");
    assert_eq!(error.kind, DebugErrorKind::TaskResultReplacementType);
    assert_eq!(session.test_instruction_count(), instructions);

    let result = session
        .replace_completed_task_result(1, Some(root), Some(&int_expr(9)))
        .expect("replace retained result");
    assert_eq!(result.task_id, 1);
    assert_eq!(result.value, "9");
    assert_eq!(session.test_instruction_count(), instructions);

    let root = session.stack(0, 1).expect("fresh root stack").items[0].id;
    let result = session
        .replace_completed_task_result(1, Some(root), Some(&int_expr(10)))
        .expect("replace retained result again before consumption");
    assert_eq!(result.value, "10");
    assert!(matches!(
        session.test_poll_task_result(1),
        crate::vm::TaskResultPoll::Available(fpas_bytecode::Value::Integer(10))
    ));

    let root = session.stack(0, 1).expect("current root stack").items[0].id;
    let error = session
        .replace_completed_task_result(1, Some(root), Some(&int_expr(11)))
        .expect_err("consumed result");
    assert_eq!(error.kind, DebugErrorKind::TaskResultReplacementUnsupported);
    assert!(error.message.contains("already consumed"), "{error:?}");
}

#[test]
fn completed_result_replacement_rejects_unknown_pending_and_root_tasks() {
    let mut session = DebugSession::new(spawn_value_task_executable()).expect("debug session");
    let _compute = stop_in_callee(&mut session, "compute");
    let root = session.stack(0, 1).expect("root stack").items[0].id;
    assert_eq!(
        session
            .replace_completed_task_result(99, Some(root), Some(&int_expr(1)))
            .expect_err("unknown task")
            .kind,
        DebugErrorKind::UnknownTask
    );
    for task_id in [0, 1] {
        let error = session
            .replace_completed_task_result(task_id, Some(root), Some(&int_expr(1)))
            .expect_err("not completed");
        assert_eq!(error.kind, DebugErrorKind::TaskResultReplacementUnsupported);
    }
}
