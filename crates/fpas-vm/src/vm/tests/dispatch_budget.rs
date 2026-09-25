//! Batched production execution keeps exact yield budgets and instruction counts.

use std::sync::Arc;

use fpas_bytecode::{Constant, Instruction, Opcode, Value, VerifiedExecutable};

use super::support::{abc, abx, verified};
use crate::vm::worker::Worker;

#[test]
fn scheduled_execution_yields_at_the_instruction_budget_and_resumes() {
    let executable = verified(
        vec![
            abc(Opcode::LoadUnit, 0, 0, 0),
            abc(Opcode::LoadUnit, 0, 0, 0),
            abc(Opcode::LoadUnit, 0, 0, 0),
            super::support::return_unit(),
        ],
        Vec::new(),
        vec!["root", "test.fpas"],
        1,
    );
    let mut worker = Worker::new(Arc::new(executable)).expect("worker");
    worker.task_id = 1;
    worker.instructions_until_yield = 3;

    assert!(worker.run_task().expect("first timeslice").is_none());
    assert_eq!(worker.ip, 3);
    assert_eq!(worker.instruction_count, 3);

    worker.instructions_until_yield = crate::vm::TIMESLICE;
    assert_eq!(
        worker.run_task().expect("resumed timeslice"),
        Some(Value::Unit)
    );
    assert_eq!(worker.instruction_count, 4);
}

/// Count from zero to `limit` with a backward branch: 3 setup, 3 per iteration, 1 return.
fn counting_loop(limit: i64) -> VerifiedExecutable {
    let code: Vec<Instruction> = vec![
        abx(Opcode::LoadConstant, 0, 0),
        abx(Opcode::LoadConstant, 1, 1),
        abx(Opcode::LoadConstant, 2, 2),
        abc(Opcode::AddInteger, 0, 0, 1),
        abc(Opcode::LessInteger, 3, 0, 2),
        abx(Opcode::BranchIfTrue, 3, 3),
        super::support::return_unit(),
    ];
    verified(
        code,
        vec![
            Constant::Integer(0),
            Constant::Integer(1),
            Constant::Integer(limit),
        ],
        vec!["root", "test.fpas"],
        4,
    )
}

#[test]
fn root_batches_run_backward_loops_with_exact_instruction_counts() {
    let mut worker = Worker::new(Arc::new(counting_loop(1000))).expect("worker");
    let execution = worker.run_in_place().expect("loop completes");
    assert_eq!(execution.value, Value::Unit);
    assert_eq!(execution.instruction_count, 3 + 3 * 1000 + 1);
    assert_eq!(worker.registers[0], Value::Integer(1000));
}

#[test]
fn scheduled_budgets_spanning_several_batches_yield_exactly() {
    let mut worker = Worker::new(Arc::new(counting_loop(1000))).expect("worker");
    worker.task_id = 1;
    worker.instructions_until_yield = 300;

    assert!(worker.run_task().expect("first timeslice").is_none());
    assert_eq!(worker.instruction_count, 300);
    assert_eq!(worker.instructions_until_yield, 0);
}
