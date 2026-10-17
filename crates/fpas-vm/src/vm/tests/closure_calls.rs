//! Copying closure captures from the caller's function register into the callee frame.

use fpas_bytecode::{Constant, FunctionId, Opcode, ReturnConvention, Value};

use crate::vm::Vm;

use super::calls::{FunctionSpec, abc, image};
use super::support::abx;

#[test]
fn closure_passed_as_its_own_overlapping_argument_keeps_its_captures() {
    let executable = image(
        vec![
            abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
            abx(Opcode::LoadConstant, 0, 0),
            abc(Opcode::MakeClosure, 1, 2, 0, 1),
            // The closure is also the last argument, so the callee frame starts on it.
            abc(Opcode::CallValue, 0, 1, 1, 1),
            abc(Opcode::Return, 0, 0, 0, 0),
            abc(Opcode::AddIntegerImm, 1, 1, 2, 0),
            abc(Opcode::Return, 1, 0, 0, 0),
        ],
        vec![Constant::Integer(40)],
        &[
            FunctionSpec {
                start: 0,
                end: 1,
                arity: 0,
                captures: 0,
                registers: 0,
                returns: ReturnConvention::Unit,
            },
            FunctionSpec {
                start: 1,
                end: 5,
                arity: 0,
                captures: 0,
                registers: 2,
                returns: ReturnConvention::Value,
            },
            FunctionSpec {
                start: 5,
                end: 7,
                arity: 1,
                captures: 1,
                registers: 2,
                returns: ReturnConvention::Value,
            },
        ],
    );

    let result = Vm::new(executable)
        .call(FunctionId::new(1), Vec::new())
        .expect("closure call should succeed");
    assert_eq!(result.value, Value::Integer(42));
}
