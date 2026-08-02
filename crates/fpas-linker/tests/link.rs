#![expect(
    clippy::expect_used,
    reason = "linker fixtures use expect for compact result assertions"
)]

mod object_fixture;

use fpas_bytecode::Op;
use fpas_linker::link_objects;
use fpas_unit::object::{ObjectConstant, ObjectFunction};

use object_fixture::object;

#[test]
fn linker_rebases_jumps_functions_and_constant_indices() {
    let mut unit = object(
        "demo.unit",
        vec![Op::Jump(3), Op::Unit, Op::Return, Op::Halt],
        vec![ObjectConstant::String("shared".to_string())],
    );
    unit.functions.insert(
        "demo.unit.run".to_string(),
        ObjectFunction {
            code_start: 1,
            arity: 0,
        },
    );
    let program = object(
        "demo.program",
        vec![Op::Constant(0), Op::Halt],
        vec![ObjectConstant::String("shared".to_string())],
    );

    let linked = link_objects(&[unit], &program).expect("linking");
    assert_eq!(
        linked.code(),
        &[Op::Jump(3), Op::Unit, Op::Return, Op::Constant(0), Op::Halt]
    );
    assert_eq!(linked.constants().len(), 1);
    assert_eq!(linked.functions().get("demo.unit.run"), Some(&(1, 0)));
}

#[test]
fn linker_rebases_every_constant_operand_shape() {
    let prefix = object(
        "prefix",
        vec![Op::Unit, Op::Halt],
        vec![ObjectConstant::String("prefix".to_string())],
    );
    let mut unit = object(
        "unit",
        vec![
            Op::Jump(3),
            Op::Unit,
            Op::Return,
            Op::GetGlobal(0),
            Op::SetGlobal(0),
            Op::GlobalIndexSet(0, 2),
            Op::Call(0, 1),
            Op::MakeClosure(0, 0),
            Op::MakeRecord(0, 1),
            Op::FieldGet(0),
            Op::FieldSet(0),
            Op::MakeEnum(0, 1, 0),
            Op::IsVariant(0, 1),
            Op::Halt,
        ],
        vec![
            ObjectConstant::String("demo.name".to_string()),
            ObjectConstant::String("Demo.Variant".to_string()),
        ],
    );
    unit.functions.insert(
        "demo.name".to_string(),
        ObjectFunction {
            code_start: 1,
            arity: 1,
        },
    );
    let program = object("program", vec![Op::Halt], Vec::new());

    let linked = link_objects(&[prefix, unit], &program).expect("linking");

    assert_eq!(
        linked.code(),
        &[
            Op::Unit,
            Op::Jump(4),
            Op::Unit,
            Op::Return,
            Op::GetGlobal(1),
            Op::SetGlobal(1),
            Op::GlobalIndexSet(1, 2),
            Op::Call(1, 1),
            Op::MakeClosure(1, 0),
            Op::MakeRecord(1, 1),
            Op::FieldGet(1),
            Op::FieldSet(1),
            Op::MakeEnum(1, 2, 0),
            Op::IsVariant(1, 2),
            Op::Halt,
        ]
    );
    assert_eq!(linked.constants().len(), 3);
}

#[test]
fn startup_sections_are_concatenated_dependency_first_with_one_halt() {
    let first = object("first", vec![Op::Unit, Op::Halt], Vec::new());
    let second = object("second", vec![Op::Pop, Op::Halt], Vec::new());
    let program = object("program", vec![Op::Print, Op::Halt], Vec::new());

    let linked = link_objects(&[first, second], &program).expect("linking");
    assert_eq!(linked.code(), &[Op::Unit, Op::Pop, Op::Print, Op::Halt]);
    assert_eq!(
        linked
            .code()
            .iter()
            .filter(|op| matches!(op, Op::Halt))
            .count(),
        1
    );
}
