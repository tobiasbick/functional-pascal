//! Direct calls carry no captures; admission establishes that invariant for execution.

use fpas_bytecode::{CodeRange, InstructionAddress, NO_REGISTER, Opcode, ValidationErrorKind};

use super::support::{abc, minimal_executable, replace_root_code, return_unit};

#[test]
fn direct_and_tail_calls_reject_targets_requiring_captures() {
    for opcode in [Opcode::CallDirect, Opcode::TailCall] {
        let mut image = minimal_executable();
        replace_root_code(
            &mut image,
            vec![abc(opcode, NO_REGISTER, 1, 0, 0), return_unit()],
        );
        let mut callee = image.functions[0].clone();
        callee.code = CodeRange::new(InstructionAddress::new(2), InstructionAddress::new(3));
        callee.register_count = 1;
        image.functions.push(callee);
        image.code.push(return_unit());
        let mut source = image.source_map.runs[0];
        source.instruction_start = InstructionAddress::new(2);
        image.source_map.runs.push(source);
        image
            .clone()
            .verify()
            .expect("noncapturing target is valid");

        image.functions[1].capture_count = 1;
        let error = image
            .verify()
            .expect_err("direct calls cannot supply captures");
        assert_eq!(
            error.kind,
            ValidationErrorKind::CallCaptures {
                target: 1,
                captures: 1
            }
        );
    }
}
