//! Execution of fused compare-and-branch and counting-loop instructions.

use fpas_bytecode::{Constant, Opcode, Value};

use super::*;

#[test]
fn descending_for_loop_stops_at_the_bound_without_advancing_past_it() {
    let executable = verified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abx(Opcode::LoadConstant, 1, 1),
            abx(Opcode::LoadConstant, 2, 2),
            abc(Opcode::AddInteger, 2, 2, 0),
            abc_aux(Opcode::ForLoop, 0, 1, 0, 1),
            abx(Opcode::Jump, 0, 3),
            abx(Opcode::Jump, 0, 7),
            return_unit(),
        ],
        vec![
            Constant::Integer(3),
            Constant::Integer(1),
            Constant::Integer(0),
        ],
        vec!["root", "test.fpas"],
        3,
    );
    let (_, registers, count) = execute(executable).expect("loop executes");
    assert_eq!(registers[2], Value::Integer(6));
    assert_eq!(registers[0], Value::Integer(1));
    // Three setup loads, three body adds, three ForLoop steps, and the return.
    assert_eq!(count, 10);
}

/// Branch on `left < right` (or `>`) through a fused head and its `BranchIfFalse` payload.
fn fused_branch(opcode: Opcode) -> (Value, Value) {
    let executable = verified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abx(Opcode::LoadConstant, 1, 1),
            abc(opcode, 2, 0, 1),
            abx(Opcode::BranchIfFalse, 2, 6),
            abx(Opcode::LoadConstant, 3, 2),
            abx(Opcode::Jump, 0, 7),
            abx(Opcode::LoadConstant, 3, 3),
            return_unit(),
        ],
        vec![
            Constant::Integer(1),
            Constant::Integer(2),
            Constant::Integer(10),
            Constant::Integer(20),
        ],
        vec!["root", "test.fpas"],
        4,
    );
    let (_, registers, _) = execute(executable).expect("fused branch executes");
    (registers[2].clone(), registers[3].clone())
}

#[test]
fn fused_integer_branches_write_the_condition_and_follow_the_payload() {
    assert_eq!(
        fused_branch(Opcode::BranchIfLessInteger),
        (Value::Boolean(true), Value::Integer(10))
    );
    assert_eq!(
        fused_branch(Opcode::BranchIfGreaterInteger),
        (Value::Boolean(false), Value::Integer(20))
    );
}
