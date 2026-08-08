use fpas_bytecode::{Constant, FunctionId, Opcode, ReturnConvention, Value};

use crate::vm::Vm;

use super::calls::{FunctionSpec, abc, image};
use super::support::abx;

#[test]
fn reused_call_frame_registers_do_not_expose_previous_values() {
    let executable = image(
        vec![
            abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
            abc(Opcode::CallDirect, fpas_bytecode::NO_REGISTER, 2, 0, 0),
            abc(Opcode::CallDirect, 0, 3, 0, 0),
            abc(Opcode::Return, 0, 0, 0, 0),
            abx(Opcode::LoadConstant, 1, 0),
            abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
            abc(Opcode::Return, 1, 0, 0, 0),
        ],
        vec![Constant::Integer(99)],
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
                end: 4,
                arity: 0,
                captures: 0,
                registers: 1,
                returns: ReturnConvention::Value,
            },
            FunctionSpec {
                start: 4,
                end: 6,
                arity: 0,
                captures: 0,
                registers: 2,
                returns: ReturnConvention::Unit,
            },
            FunctionSpec {
                start: 6,
                end: 7,
                arity: 0,
                captures: 0,
                registers: 2,
                returns: ReturnConvention::Value,
            },
        ],
    );

    let result = Vm::new(executable)
        .call(FunctionId::new(1), Vec::new())
        .expect("reused call frame should return successfully");
    assert_eq!(result.value, Value::Unit);
}
