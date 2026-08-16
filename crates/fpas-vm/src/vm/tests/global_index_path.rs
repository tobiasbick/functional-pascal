use fpas_bytecode::{Constant, GlobalInfo, Opcode, StringId, Value};
use fpas_diagnostics::codes::RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS;

use super::*;

#[test]
fn global_index_path_updates_nested_arrays_and_preserves_aliases() {
    let mut image = unverified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abx(Opcode::LoadConstant, 1, 1),
            abx(Opcode::LoadConstant, 2, 2),
            abc(Opcode::MakeArray, 3, 0, 2),
            abc(Opcode::MakeArray, 4, 3, 1),
            abx(Opcode::StoreGlobal, 4, 0),
            abx(Opcode::LoadGlobal, 5, 0),
            abc(Opcode::Move, 6, 5, 0),
            abc(Opcode::Move, 7, 0, 0),
            abc(Opcode::Move, 8, 1, 0),
            abc(Opcode::Move, 9, 2, 0),
            abc_aux(Opcode::StoreGlobalIndexPath, 5, 0, 7, 2),
            abx(Opcode::LoadGlobal, 10, 0),
            abc(Opcode::IndexGet, 11, 6, 0),
            abc(Opcode::IndexGet, 12, 11, 1),
            abc(Opcode::IndexGet, 13, 10, 0),
            abc(Opcode::IndexGet, 14, 13, 1),
            return_unit(),
        ],
        vec![
            Constant::Integer(0),
            Constant::Integer(1),
            Constant::Integer(9),
        ],
        vec!["root", "test.fpas", "surface"],
        15,
    );
    image.globals = vec![GlobalInfo {
        name: StringId::new(2),
        ty: fpas_bytecode::DebugTypeId::new(0),
        mutable: true,
        initializer: None,
    }];
    let (_, registers, _) = execute(image.verify().expect("global path image must verify"))
        .expect("global path update must run");
    assert_eq!(registers[12], Value::Integer(1));
    assert_eq!(registers[14], Value::Integer(9));
}

#[test]
fn global_index_path_rejects_out_of_bounds_indexes() {
    let mut image = unverified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abx(Opcode::LoadConstant, 1, 1),
            abc(Opcode::MakeArray, 2, 0, 1),
            abx(Opcode::StoreGlobal, 2, 0),
            abx(Opcode::LoadGlobal, 3, 0),
            abc(Opcode::Move, 4, 1, 0),
            abc(Opcode::Move, 5, 0, 0),
            abc_aux(Opcode::StoreGlobalIndexPath, 3, 0, 4, 1),
            return_unit(),
        ],
        vec![Constant::Integer(4), Constant::Integer(9)],
        vec!["root", "test.fpas", "surface"],
        6,
    );
    image.globals = vec![GlobalInfo {
        name: StringId::new(2),
        ty: fpas_bytecode::DebugTypeId::new(0),
        mutable: true,
        initializer: None,
    }];
    let error = execute(image.verify().expect("global path image must verify"))
        .expect_err("out-of-bounds global update must fail");
    assert_eq!(error.code, RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS);
}
