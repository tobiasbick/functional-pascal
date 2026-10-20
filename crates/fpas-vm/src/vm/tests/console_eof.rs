//! Embedders can close queued console input without reading process stdin.

use fpas_bytecode::{ConsoleIntrinsic, Instruction, Intrinsic, NO_REGISTER, Opcode};

use super::{return_unit, verified};
use crate::vm::Vm;

#[test]
fn closed_host_input_returns_none_from_both_console_intrinsics() {
    for intrinsic in [ConsoleIntrinsic::ReadLn, ConsoleIntrinsic::Read] {
        let executable = verified(
            vec![
                Instruction::abc(
                    Opcode::Intrinsic,
                    0,
                    u16::from(Intrinsic::Console(intrinsic)),
                    0,
                    0,
                )
                .expect("console read"),
                Instruction::abc(
                    Opcode::Intrinsic,
                    NO_REGISTER,
                    u16::from(Intrinsic::Console(ConsoleIntrinsic::WriteLn)),
                    0,
                    1,
                )
                .expect("print EOF value"),
                return_unit(),
            ],
            Vec::new(),
            vec!["main", "eof.fpas"],
            1,
        );
        let mut vm = Vm::new(executable);
        vm.close_text_input();
        vm.run().expect("closed input");
        assert_eq!(vm.output().lines, vec!["None"]);
    }
}
