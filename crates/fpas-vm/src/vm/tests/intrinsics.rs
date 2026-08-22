use fpas_bytecode::{
    ArgsIntrinsic, ConsoleIntrinsic, Constant, Instruction, Intrinsic, Opcode, TestIntrinsic, Value,
};

use super::{abx, return_unit};
use crate::vm::Vm;

fn intrinsic(destination: u16, intrinsic: Intrinsic, base: u16, count: u8) -> Instruction {
    Instruction::abc(
        Opcode::Intrinsic,
        destination,
        u16::from(intrinsic),
        base,
        count,
    )
    .expect("intrinsic instruction")
}

#[test]
fn args_are_read_from_the_isolated_register_host() {
    let executable = super::verified(
        vec![
            intrinsic(0, Intrinsic::Args(ArgsIntrinsic::ParamCount), 0, 0),
            intrinsic(
                fpas_bytecode::NO_REGISTER,
                Intrinsic::Console(ConsoleIntrinsic::WriteLn),
                0,
                1,
            ),
            return_unit(),
        ],
        Vec::new(),
        vec!["main", "args.fpas"],
        1,
    );
    let mut vm = Vm::with_args(executable, vec!["one".into(), "two".into()]);
    vm.run().expect("args intrinsic");
    assert_eq!(vm.output().lines, vec!["2"]);
}

#[test]
fn console_and_test_input_share_one_deterministic_host() {
    let executable = super::verified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            intrinsic(
                fpas_bytecode::NO_REGISTER,
                Intrinsic::Test(TestIntrinsic::PushReadLn),
                0,
                1,
            ),
            intrinsic(1, Intrinsic::Console(ConsoleIntrinsic::ReadLn), 0, 0),
            intrinsic(
                fpas_bytecode::NO_REGISTER,
                Intrinsic::Console(ConsoleIntrinsic::WriteLn),
                1,
                1,
            ),
            return_unit(),
        ],
        vec![Constant::String(fpas_bytecode::StringId::new(2))],
        vec!["main", "console.fpas", "hello"],
        2,
    );
    let mut vm = Vm::new(executable);
    let execution = vm.run().expect("hosted console input/output");
    assert_eq!(execution.value, Value::Unit);
    assert_eq!(vm.output().lines, vec!["hello"]);
}

#[test]
fn queued_key_input_is_available_to_console_intrinsics() {
    let executable = super::verified(
        vec![
            intrinsic(0, Intrinsic::Console(ConsoleIntrinsic::ReadKey), 0, 0),
            intrinsic(
                fpas_bytecode::NO_REGISTER,
                Intrinsic::Console(ConsoleIntrinsic::WriteLn),
                0,
                1,
            ),
            return_unit(),
        ],
        Vec::new(),
        vec!["main", "key.fpas"],
        1,
    );
    let mut vm = Vm::new(executable);
    vm.push_readkey_input("Z");
    vm.run().expect("hosted key input");
    assert_eq!(vm.output().lines, vec!["Z"]);
}

#[test]
fn task_wait_rejects_a_non_task_operand() {
    let executable = super::verified(
        vec![
            intrinsic(0, Intrinsic::Task(fpas_bytecode::TaskIntrinsic::Wait), 0, 0),
            return_unit(),
        ],
        Vec::new(),
        vec!["main", "task.fpas"],
        1,
    );
    let error = Vm::new(executable)
        .run()
        .expect_err("Wait must validate its operand");
    assert!(error.message.contains("Expected task"), "{error:?}");
}
