use super::*;
use crate::execution::vm::debug::{DebugEvaluationLimits, DebugExpression, DebugSession};
use crate::{RootKind, Value};

fn session() -> DebugSession {
    let (program, errors) = fpas_parser::parse("program Roots; begin end.");
    assert!(errors.is_empty());
    DebugSession::new(fpas_compiler::compile(&program).expect("image")).expect("session")
}
#[test]
fn operation_handoffs_release_on_success_error_and_nested_scope_exit() {
    let roots = Arc::new(EvaluationRoots::default());
    let domain = MemoryDomain::new();
    let outer = roots.enter();
    roots.retain([domain.root_values(RootKind::Evaluation, vec![Value::Integer(1)])]);
    let inner = roots.enter();
    drop(inner);
    assert_eq!(domain.root_count(), 1);
    drop(outer);
    assert_eq!(domain.root_count(), 0);
    let failure = || -> Result<(), ()> {
        let _scope = roots.enter();
        roots.retain([domain.root_values(RootKind::Evaluation, vec![Value::Integer(2)])]);
        Err(())
    };
    assert!(failure().is_err());
    assert_eq!(domain.root_count(), 0);
}
#[test]
fn public_evaluation_does_not_accumulate_handoff_roots() {
    let mut session = session();
    for _ in 0..10 {
        session
            .evaluate(&DebugExpression::Integer(7), None)
            .expect("evaluation");
        assert!(session.evaluation_roots.domains().is_empty());
    }
    assert!(
        session
            .evaluate_with_limits(
                &DebugExpression::Name("Absent".into()),
                None,
                DebugEvaluationLimits::default()
            )
            .is_err()
    );
    assert!(session.evaluation_roots.domains().is_empty());
}
#[test]
fn debug_termination_returns_a_registered_value_after_session_destruction() {
    let mut session = session();
    let crate::execution::DebugRunResult::Terminated(termination) =
        session.continue_execution().expect("finish")
    else {
        panic!("termination");
    };
    drop(session);
    assert_eq!(
        termination.value.try_to_data().expect("retired unit"),
        Value::Unit
    );
}
