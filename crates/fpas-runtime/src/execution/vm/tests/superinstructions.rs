//! Execution of fused compare-and-branch and counting-loop instructions.

use crate::{Constant, Opcode, Value};

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

/// Compare two operands and record which branch path ran.
fn branch_image(
    opcode: Opcode,
    branch: Opcode,
    destination: u16,
    operands: [Constant; 2],
) -> crate::VerifiedExecutable {
    verified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abx(Opcode::LoadConstant, 1, 1),
            abc(opcode, destination, 0, 1),
            abx(branch, destination, 6),
            abx(Opcode::LoadConstant, 3, 2),
            abx(Opcode::Jump, 0, 7),
            abx(Opcode::LoadConstant, 3, 3),
            return_unit(),
        ],
        vec![
            operands[0],
            operands[1],
            Constant::Integer(10),
            Constant::Integer(20),
        ],
        vec!["root", "test.fpas"],
        4,
    )
}

#[test]
fn fused_integer_branches_match_separate_comparisons_with_aliased_destinations() {
    let comparisons = [
        (Opcode::BranchIfEqualInteger, Opcode::EqualInteger),
        (Opcode::BranchIfNotEqualInteger, Opcode::NotEqualInteger),
        (Opcode::BranchIfLessInteger, Opcode::LessInteger),
        (Opcode::BranchIfGreaterInteger, Opcode::GreaterInteger),
        (Opcode::BranchIfLessEqualInteger, Opcode::LessEqualInteger),
        (
            Opcode::BranchIfGreaterEqualInteger,
            Opcode::GreaterEqualInteger,
        ),
    ];
    let cases = [
        [Constant::Integer(i64::MIN), Constant::Integer(i64::MAX)],
        [Constant::Integer(i64::MAX), Constant::Integer(i64::MIN)],
        [Constant::Integer(0), Constant::Integer(0)],
        [
            Constant::Real(1.5_f64.to_bits()),
            Constant::Real(2.5_f64.to_bits()),
        ],
        [Constant::Integer(1), Constant::Boolean(true)],
        [Constant::Unit, Constant::Integer(1)],
    ];
    for (fused, separate) in comparisons {
        for branch in [Opcode::BranchIfFalse, Opcode::BranchIfTrue] {
            for destination in [0, 1, 2] {
                for operands in &cases {
                    let expected = execute(branch_image(separate, branch, destination, *operands));
                    let actual = execute(branch_image(fused, branch, destination, *operands));
                    match (expected, actual) {
                        (Ok((value, registers, count)), Ok((actual, slots, steps))) => {
                            assert_eq!(actual, value);
                            assert_eq!(slots, registers, "{fused:?} {branch:?} {destination}");
                            assert_eq!(steps + 1, count, "the payload is not a separate step");
                        }
                        (Err(expected), Err(actual)) => {
                            assert_eq!(actual.code, expected.code);
                            assert_eq!(actual.message, expected.message);
                        }
                        results => panic!("different outcomes for {fused:?}: {results:?}"),
                    }
                }
            }
        }
    }
}
