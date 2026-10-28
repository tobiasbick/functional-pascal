//! Direct unwraps preserve values, diagnostics, owned aliases and debugger execution.
use super::support::{return_unit, unverified};
use crate::execution::vm::{
    DebugExecutionLimits, DebugInspectionLimits, DebugRunResult, DebugSession, Vm, worker::Worker,
};
use crate::{
    AbcOperands, CollectionSchedule, Instruction, Intrinsic, NO_REGISTER, Opcode, OptionIntrinsic,
    ResultIntrinsic, SourceId, SourceLocation, StringId, Value,
};
use std::{num::NonZeroUsize, sync::Arc};

const UNWRAPS: [Intrinsic; 2] = [
    Intrinsic::Option(OptionIntrinsic::Unwrap),
    Intrinsic::Result(ResultIntrinsic::Unwrap),
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
        vec!["main", "root.fpas", "unwrap-call.fpas"],
        6,
    );
    image.source_map.sources.push(StringId::new(2));
    image.source_map.runs[0].source = SourceId::new(1);
    Worker::new(Arc::new(image.verify().unwrap())).unwrap()
}

fn wrap(intrinsic: Intrinsic, value: Value) -> Value {
    match intrinsic {
        Intrinsic::Option(_) => Value::option_some(value),
        Intrinsic::Result(_) => Value::result_ok(value),
        _ => unreachable!("unwrap fixture"),
    }
}

fn payloads() -> Vec<Value> {
    vec![
        Value::Unit,
        Value::Integer(42),
        Value::Real(1.5),
        Value::Boolean(true),
        Value::Str("é😀".into()),
        Value::OptionNone,
        Value::Array(vec![Value::Integer(7), Value::Unit].into()),
        Value::dict(vec![(Value::Integer(1), Value::Str("value".into()))]),
        Value::option_some(Value::result_ok(Value::Array(vec![].into()))),
    ]
}

#[test]
fn successful_unwraps_preserve_payloads_and_initialize_or_replace_destinations() {
    for intrinsic in UNWRAPS {
        for payload in payloads() {
            for (destination, initialized) in [(0, false), (0, true), (2, true)] {
                let mut worker = worker(intrinsic, destination, 1);
                let input = wrap(intrinsic, payload.clone());
                worker.store_register(2, input.clone()).unwrap();
                if destination == 0 && initialized {
                    worker.store_register(0, Value::Integer(99)).unwrap();
                }
                assert_eq!(
                    worker.register_is_initialized(usize::from(destination)),
                    initialized
                );
                let expected = crate::stdlib::run_intrinsic_borrowed(
                    intrinsic,
                    std::slice::from_ref(&input),
                    SourceLocation::new(1, 1),
                    worker.layouts.as_ref(),
                )
                .unwrap()
                .unwrap();
                worker.dispatch_one().unwrap();
                assert_eq!(worker.registers[usize::from(destination)], expected);
                assert!(worker.register_is_initialized(usize::from(destination)));
                if destination != 2 {
                    assert_eq!(worker.registers[2], input);
                }
            }
        }
    }
}

#[test]
fn discarded_unwraps_preserve_receivers_and_other_registers() {
    for intrinsic in UNWRAPS {
        for payload in payloads() {
            let mut worker = worker(intrinsic, NO_REGISTER, 1);
            let input = wrap(intrinsic, payload);
            worker.store_register(0, Value::Integer(99)).unwrap();
            worker.store_register(2, input.clone()).unwrap();
            worker.dispatch_one().unwrap();
            assert_eq!(worker.registers[0], Value::Integer(99));
            assert_eq!(worker.registers[2], input);
            assert!(worker.register_is_initialized(2));
        }
    }
}

#[test]
fn unwrapped_arrays_keep_cell_identity_and_copy_on_write_and_release_overwritten_values() {
    for intrinsic in UNWRAPS {
        for destination in [0, 2] {
            let mut worker = worker(intrinsic, destination, 1);
            let old = worker.domain.new_cell(Value::Unit).unwrap();
            let old_weak = old.downgrade();
            worker.store_register(0, Value::Cell(old)).unwrap();
            let cell = worker.domain.new_cell(Value::Integer(7)).unwrap();
            let weak = cell.downgrade();
            let receiver = wrap(
                intrinsic,
                Value::Array(vec![Value::Cell(cell.clone())].into()),
            );
            let alias = receiver.clone();
            worker.store_register(2, receiver).unwrap();
            worker.dispatch_one().unwrap();
            assert_eq!(old_weak.upgrade().is_none(), destination == 0);
            let Value::Array(values) = &mut worker.registers[usize::from(destination)] else {
                panic!("array payload");
            };
            let Value::Cell(actual) = &values[0] else {
                panic!("cell payload");
            };
            assert!(actual.ptr_eq(&cell));
            values.replace_element(0, Value::Integer(9));
            let inner = match &alias {
                Value::OptionSome(inner) | Value::ResultOk(inner) => &**inner,
                _ => panic!("retained wrapper"),
            };
            let Value::Array(original) = inner else {
                panic!("retained array");
            };
            assert!(matches!(&original[0], Value::Cell(actual) if actual.ptr_eq(&cell)));
            drop(cell);
            assert!(weak.upgrade().is_some());
            drop(alias);
            worker.reset_registers(0);
            // Direct fixture mutations must also refresh the published execution roots.
            worker.publish_execution(&[]).unwrap();
            assert!(weak.upgrade().is_none());
            assert!(old_weak.upgrade().is_none());
        }
    }
}

#[test]
fn failed_unwraps_keep_exact_std_diagnostics_and_leave_registers_unchanged() {
    for intrinsic in UNWRAPS {
        let wrong_wrapper = wrap(
            if intrinsic == UNWRAPS[0] {
                UNWRAPS[1]
            } else {
                UNWRAPS[0]
            },
            Value::Integer(3),
        );
        let mut arguments = vec![
            vec![],
            vec![wrap(intrinsic, Value::Unit), wrap(intrinsic, Value::Unit)],
            vec![Value::Integer(3), Value::Integer(4)],
        ];
        arguments.extend(
            [
                Value::OptionNone,
                Value::result_error(Value::Str("failure é😀".into())),
                Value::result_error(Value::Array(vec![Value::Integer(5)].into())),
                Value::Integer(9),
                Value::Unit,
                wrong_wrapper,
            ]
            .into_iter()
            .map(|value| vec![value]),
        );
        for arguments in arguments {
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
fn out_of_bounds_unwrap_arguments_keep_the_intrinsic_window_diagnostic() {
    for intrinsic in UNWRAPS {
        let mut worker = worker(intrinsic, 0, 1);
        let mut op = operands(intrinsic, 0, 1);
        op.c = 6;
        let actual = worker.execute_intrinsic::<true>(op).unwrap_err();
        assert_eq!(
            actual.code,
            fpas_diagnostics::codes::INTERNAL_VM_INVARIANT_FAILURE
        );
        assert_eq!(
            actual.message,
            "Intrinsic register window 6..+1 is outside the current frame"
        );
        assert!(!worker.register_is_initialized(0));
    }
}

#[test]
fn immediate_debug_unwraps_preserve_values_and_complete_diagnostics() {
    for intrinsic in UNWRAPS {
        let mut worker = worker(intrinsic, 0, 1);
        for value in payloads() {
            let input = wrap(intrinsic, value.clone());
            assert_eq!(
                worker.execute_debug_intrinsic(intrinsic, &[input]).unwrap(),
                value
            );
        }
        for input in [
            Value::OptionNone,
            Value::result_error(Value::Integer(7)),
            Value::Unit,
        ] {
            let expected = crate::stdlib::run_intrinsic_borrowed(
                intrinsic,
                std::slice::from_ref(&input),
                SourceLocation::new_with_source(41, 7, 1),
                worker.layouts.as_ref(),
            )
            .unwrap_err();
            let actual = worker
                .execute_debug_intrinsic(intrinsic, &[input])
                .unwrap_err();
            assert_eq!(*actual, *expected);
        }
    }
}

#[test]
fn nested_unwraps_keep_captured_aliases_rooted_in_callbacks_and_seeded_debugging() {
    let (program, errors) = fpas_parser::parse(
        r#"program Unwraps;
function Make(Seed: integer): function(): integer;
begin
  var Count := Seed;
  var Current: function(): integer := function(): integer begin return 0; end function;
  Current := function(): integer begin
    Count := Count + 1;
    if Count < 0 then return Current(); end if;
    return Count;
  end function;
  return Current;
end function;
function Open(Entry: Option of Result of (function(): integer, string)): function(): integer;
begin return Entry.Unwrap().Unwrap(); end function;
procedure Exercise();
begin
  const Wrapped: Option of Result of (function(): integer, string) := Some(Ok(Make(6)));
  const First := Open(Wrapped);
  const Second := Open(Wrapped);
  if (First() <> 7) or (Second() <> 8) then panic('aliases'); end if;
  const Values := [1, 2, 3].Map(function(N: integer): integer begin
    const R: Result of (integer, string) := Ok(N);
    const O: Option of integer := Some(R.Unwrap());
    return O.Unwrap();
  end function);
  if (Values[0] <> 1) or (Values[1] <> 2) or (Values[2] <> 3) then panic('callbacks'); end if;
  discard Wrapped.Unwrap();
end procedure;
begin Exercise(); end."#,
    );
    assert!(errors.is_empty(), "{errors:?}");
    let image = fpas_compiler::compile(&program).unwrap();
    Vm::new(image.clone()).run().unwrap();
    let mut worker = Worker::new(Arc::new(image.clone())).unwrap();
    let domain = worker.domain.clone();
    crate::execution::vm::domain::configure(
        &domain,
        CollectionSchedule {
            allocations: NonZeroUsize::new(1).unwrap(),
            mutations: NonZeroUsize::new(1).unwrap(),
            safe_points: NonZeroUsize::new(1).unwrap(),
        },
    );
    drop(worker.run_in_place().unwrap());
    assert!(domain.collection_stats().completed_passes > 1);
    worker.retire_execution();
    assert_eq!(domain.registered_count(), 0);
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
