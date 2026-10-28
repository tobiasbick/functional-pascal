//! Direct lengths preserve Std diagnostics, register lifetimes and seeded debugging.
use super::support::{return_unit, unverified};
use crate::execution::vm::{
    DebugExecutionLimits, DebugInspectionLimits, DebugRunResult, DebugSession, Vm, worker::Worker,
};
use crate::{
    AbcOperands, ArrayIntrinsic, DictIntrinsic, Instruction, Intrinsic, NO_REGISTER, Opcode,
    SourceId, SourceLocation, StrIntrinsic, StringId, Value,
};
use std::sync::Arc;

const LENGTHS: [Intrinsic; 3] = [
    Intrinsic::Array(ArrayIntrinsic::Length),
    Intrinsic::Str(StrIntrinsic::Length),
    Intrinsic::Dict(DictIntrinsic::Length),
];

fn operands(intrinsic: Intrinsic, destination: u16, count: u8) -> AbcOperands {
    AbcOperands {
        a: destination,
        b: intrinsic.into(),
        c: 2,
        auxiliary: count,
    }
}

fn worker(intrinsic: Intrinsic, destination: u16, count: u8) -> Worker {
    let op = operands(intrinsic, destination, count);
    let mut image = unverified(
        vec![
            Instruction::abc(Opcode::Intrinsic, op.a, op.b, op.c, op.auxiliary).unwrap(),
            return_unit(),
        ],
        vec![],
        vec!["main", "root.fpas", "length-call.fpas"],
        6,
    );
    image.source_map.sources.push(StringId::new(2));
    image.source_map.runs[0].source = SourceId::new(1);
    Worker::new(Arc::new(image.verify().unwrap())).unwrap()
}

fn inputs() -> Vec<(Intrinsic, Value, i64)> {
    vec![
        (LENGTHS[0], Value::Array(vec![].into()), 0),
        (
            LENGTHS[0],
            Value::Array(vec![Value::Integer(1), Value::Unit].into()),
            2,
        ),
        (LENGTHS[1], Value::Str("".into()), 0),
        (LENGTHS[1], Value::Str("aé𝄞e\u{301}".into()), 5),
        (LENGTHS[2], Value::dict(vec![]), 0),
        (
            LENGTHS[2],
            Value::dict(vec![
                (Value::Integer(1), Value::Unit),
                (Value::Integer(2), Value::Unit),
            ]),
            2,
        ),
    ]
}

#[test]
fn lengths_read_cached_sizes_and_initialize_or_replace_destinations() {
    for (intrinsic, input, expected) in inputs() {
        for destination in [0, 2] {
            let mut worker = worker(intrinsic, destination, 1);
            worker.store_register(0, Value::Integer(99)).unwrap();
            worker.store_register(2, input.clone()).unwrap();
            worker.dispatch_one().unwrap();
            assert_eq!(
                worker.registers[usize::from(destination)],
                Value::Integer(expected)
            );
            assert!(worker.register_is_initialized(usize::from(destination)));
            if destination != 2 {
                let expected = crate::stdlib::run_intrinsic_borrowed(
                    intrinsic,
                    std::slice::from_ref(&input),
                    SourceLocation::new(1, 1),
                    worker.layouts.as_ref(),
                )
                .unwrap()
                .unwrap();
                assert_eq!(worker.registers[0], expected);
                assert_eq!(worker.registers[2], input);
                assert!(worker.register_is_initialized(2));
            }
        }
    }
}

#[test]
fn discarded_lengths_preserve_the_receiver_and_other_registers() {
    for (intrinsic, input, _) in inputs() {
        let mut worker = worker(intrinsic, NO_REGISTER, 1);
        worker.store_register(0, Value::Integer(99)).unwrap();
        worker.store_register(2, input.clone()).unwrap();
        worker.dispatch_one().unwrap();
        assert_eq!(worker.registers[0], Value::Integer(99));
        assert_eq!(worker.registers[2], input);
        assert!(worker.register_is_initialized(2));
    }
}

#[test]
fn overwriting_the_receiver_releases_its_values_but_preserves_shared_aliases() {
    for intrinsic in [LENGTHS[0], LENGTHS[2]] {
        let mut worker = worker(intrinsic, 2, 1);
        let cell = worker.domain.new_cell(Value::Integer(7)).unwrap();
        let weak = cell.downgrade();
        let receiver = if intrinsic == LENGTHS[0] {
            Value::Array(vec![Value::Cell(cell)].into())
        } else {
            Value::dict(vec![(Value::Integer(1), Value::Cell(cell))])
        };
        let alias = receiver.clone();
        worker.store_register(2, receiver).unwrap();
        worker.dispatch_one().unwrap();
        assert_eq!(worker.registers[2], Value::Integer(1));
        assert!(weak.upgrade().is_some());
        drop(alias);
        assert!(weak.upgrade().is_none());
    }
}

#[test]
fn invalid_length_calls_keep_exact_std_diagnostics_and_destination_contents() {
    for intrinsic in LENGTHS {
        let valid = inputs()
            .into_iter()
            .find(|(kind, _, _)| *kind == intrinsic)
            .unwrap()
            .1;
        let wrong_collection = inputs()
            .into_iter()
            .find(|(kind, _, _)| *kind != intrinsic)
            .unwrap()
            .1;
        for arguments in [
            vec![],
            vec![Value::Integer(9)],
            vec![Value::OptionNone],
            vec![wrong_collection],
            vec![valid.clone(), valid],
            vec![Value::Integer(9), Value::Integer(8)],
        ] {
            for destination in [0, 2, NO_REGISTER] {
                let count = u8::try_from(arguments.len()).unwrap();
                let mut worker = worker(intrinsic, destination, count);
                worker.store_register(0, Value::Integer(99)).unwrap();
                for (offset, value) in arguments.iter().enumerate() {
                    worker.store_register(2 + offset, value.clone()).unwrap();
                }
                let registers = worker.registers.clone();
                let expected = crate::stdlib::run_intrinsic_borrowed(
                    intrinsic,
                    &arguments,
                    SourceLocation::new_with_source(41, 7, 1),
                    worker.layouts.as_ref(),
                )
                .unwrap_err();
                let actual = worker
                    .execute_intrinsic::<true>(operands(intrinsic, destination, count))
                    .unwrap_err();
                assert_eq!(*actual, *expected);
                assert_eq!(worker.registers, registers);
            }
        }
    }
}

#[test]
fn immediate_debug_intrinsics_use_the_same_lengths_and_failures() {
    for (intrinsic, input, expected) in inputs() {
        let mut worker = worker(intrinsic, 0, 1);
        assert_eq!(
            worker.execute_debug_intrinsic(intrinsic, &[input]).unwrap(),
            Value::Integer(expected)
        );
        let expected = crate::stdlib::run_intrinsic_borrowed(
            intrinsic,
            &[Value::Integer(3)],
            SourceLocation::new_with_source(41, 7, 1),
            worker.layouts.as_ref(),
        )
        .unwrap_err();
        let actual = worker
            .execute_debug_intrinsic(intrinsic, &[Value::Integer(3)])
            .unwrap_err();
        assert_eq!(*actual, *expected);
    }
}

#[test]
fn lengths_in_nested_frames_preserve_seeded_scheduling() {
    let (program, errors) = fpas_parser::parse(
        r#"program Lengths;
function Size(Items: array of integer; Text: string; Entries: dict of string to integer): integer;
begin return Items.Length() + Text.Length() + Entries.Length(); end function;
begin
  const Items: array of integer := [1, 2];
  const Text := 'é😀';
  const Entries: dict of string to integer := ['x': 1];
  if Size(Items, Text, Entries) <> 5 then panic('lengths'); end if;
end."#,
    );
    assert!(errors.is_empty(), "{errors:?}");
    let image = fpas_compiler::compile(&program).unwrap();
    Vm::new(image.clone()).run().unwrap();
    let mut session = DebugSession::with_limits_and_schedule(
        image,
        vec![],
        DebugInspectionLimits::default(),
        DebugExecutionLimits::default(),
        Some(17),
    )
    .unwrap();
    assert!(matches!(
        session.continue_execution().unwrap(),
        DebugRunResult::Terminated(_)
    ));
    assert_eq!(session.schedule_report().status, "controlled");
}
