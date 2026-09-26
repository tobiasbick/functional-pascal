//! IR optimization passes and their effect on emitted register bytecode.

use fpas_bytecode::{Constant, Instruction, Opcode};

use super::{assert_succeeds, parse_ok};

fn code(source: &str) -> (Vec<Instruction>, Vec<Constant>) {
    let executable = crate::compile(&parse_ok(source)).expect("program compiles");
    let image = executable.executable();
    (image.code.clone(), image.constants.clone())
}

#[test]
fn constant_operands_fold_into_one_constant() {
    let source =
        "program Fold; begin var X: integer := 2 * 3 + 1; if X <> 7 then panic('wrong') end.";
    let (code, constants) = code(source);
    assert!(constants.contains(&Constant::Integer(7)));
    assert!(
        !code.iter().any(|word| matches!(
            word.opcode(),
            Ok(Opcode::MultiplyInteger | Opcode::AddInteger | Opcode::AddIntegerImm)
        )),
        "folded arithmetic must not execute at runtime"
    );
    assert_succeeds(source);
}

#[test]
fn failing_constant_operations_stay_runtime_errors() {
    let (code, _) = code("program Fail; begin var X: integer := 1 div 0; end.");
    assert!(
        code.iter().any(|word| matches!(
            word.opcode(),
            Ok(Opcode::DivideInteger | Opcode::DivideIntegerImm)
        )),
        "division by zero must still fail when the program runs"
    );
}

#[test]
fn jumps_to_the_next_block_are_not_emitted() {
    let (code, _) = code(
        "program Fallthrough; begin mutable var T: integer := 0; for I: integer := 1 to 3 do T := T + I; if T <> 6 then panic('wrong') end.",
    );
    for (address, word) in code.iter().enumerate() {
        if word.opcode() == Ok(Opcode::Jump) && address > 0 {
            let previous = code[address - 1].opcode();
            // ForLoop payload words are data, not dispatched jumps.
            if previous == Ok(Opcode::ForLoop)
                || address >= 2 && code[address - 2].opcode() == Ok(Opcode::ForLoop)
            {
                continue;
            }
            assert_ne!(
                word.abx_payload().bx as usize,
                address + 1,
                "jump at {address} only falls through"
            );
        }
    }
}

#[test]
fn loop_constants_load_once_before_the_loop() {
    let source = "program Hoist; begin mutable var Even: integer := 0; for I: integer := 1 to 10 do if I mod 2 = 0 then Even := Even + 1; if Even <> 5 then panic('wrong') end.";
    let (code, _) = code(source);
    let for_loop = code
        .iter()
        .position(|word| word.opcode() == Ok(Opcode::ForLoop))
        .expect("counting loop");
    let body = code[for_loop + 1].abx_payload().bx as usize;
    assert!(
        !code[body..for_loop]
            .iter()
            .any(|word| word.opcode() == Ok(Opcode::LoadConstant)),
        "loop body must not reload constants"
    );
    assert_succeeds(source);
}

#[test]
fn local_reads_use_local_registers_directly() {
    let (code, _) = code(
        "program Alias; begin mutable var A: integer := 1; mutable var B: integer := 2; B := A + B; if B <> 3 then panic('wrong') end.",
    );
    let add = code
        .iter()
        .find(|word| word.opcode() == Ok(Opcode::AddInteger))
        .expect("addition");
    let operands = add.abc_payload();
    assert_eq!(
        (operands.a, operands.b, operands.c),
        (1, 0, 1),
        "B := A + B reads A and B from their own registers and writes B"
    );
}
