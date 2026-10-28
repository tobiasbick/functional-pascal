//! Main-task callbacks preserve dispatch bounds, local roots and immediate host results.

use std::{num::NonZeroUsize, sync::Arc};

use crate::{ArrayIntrinsic, CollectionSchedule, FunctionId, Intrinsic, Value};

use crate::execution::vm::{dispatch::DispatchStep, worker::Worker};

fn image(source: &str) -> crate::VerifiedExecutable {
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    fpas_compiler::compile(&program).expect("callback fixture")
}

#[test]
fn main_callbacks_obey_single_instruction_dispatch_budgets() {
    let mut worker = Worker::new(Arc::new(image(
        "program T; function Double(N: integer): integer; begin return N * 2; end function; begin const Values := [1, 2, 3].Map(Double); if (Values.Length() <> 3) or (Values[0] <> 2) or (Values[1] <> 4) or (Values[2] <> 6) then panic('values'); end if; end.",
    )))
    .unwrap();
    for _ in 0..1000 {
        let before = worker.instruction_count + worker.callback_instruction_count.get();
        let batch = worker.dispatch_batch::<false>(1).unwrap();
        let after = worker.instruction_count + worker.callback_instruction_count.get();
        assert!(after - before <= 1, "callback exceeded its dispatch budget");
        assert!(batch.steps <= 1);
        match batch.step {
            DispatchStep::Return(value) => {
                assert_eq!(value, Value::Unit);
                return;
            }
            DispatchStep::Continue => {}
            DispatchStep::Suspend => panic!("main callback cannot suspend without a scheduler"),
        }
    }
    panic!("callback fixture did not finish");
}

#[test]
fn nested_callbacks_preserve_results_and_counts_across_batch_boundaries() {
    let executable = Arc::new(image(
        r#"program T;
begin
  var Calls := 0;
  const Values := [1, 2, 3].Map(function(N: integer): integer begin
    const Inner := [N, N + 1].Map(function(M: integer): integer begin
      Calls := Calls + 1;
      return M * 2;
    end function);
    return Inner[0] + Inner[1];
  end function);
  if Calls <> 6 then panic('callback count'); end if;
  if (Values[0] <> 6) or (Values[1] <> 10) or (Values[2] <> 14) then
    panic('callback results');
  end if;
end."#,
    ));
    let mut expected = None;
    for budget in [1, 2, 3, 7, 256] {
        let mut worker = Worker::new(Arc::clone(&executable)).unwrap();
        let mut scheduling_steps = 0;
        loop {
            assert!(scheduling_steps < 10_000, "callback did not finish");
            let before = worker.instruction_count;
            let batch = worker.dispatch_batch::<false>(budget).unwrap();
            assert!(worker.instruction_count - before <= u64::from(batch.steps));
            assert!(batch.steps <= budget);
            scheduling_steps += batch.steps;
            match batch.step {
                DispatchStep::Return(value) => {
                    assert_eq!(value, Value::Unit);
                    break;
                }
                DispatchStep::Continue => {}
                DispatchStep::Suspend => panic!("unexpected suspension"),
            }
        }
        let counts = (worker.instruction_count, scheduling_steps);
        assert_eq!(counts, *expected.get_or_insert(counts));
    }
}

#[test]
fn nested_main_callbacks_keep_partial_results_and_live_capture_aliases_rooted() {
    let mut worker = Worker::new(Arc::new(image(
        r#"program Roots;
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
procedure Exercise();
begin
  var Writes := 0;
  const Counters := [1, 2, 3].Map(function(N: integer): function(): integer begin
    Writes := Writes + N;
    const Inner := [N, N + 1].Map(function(M: integer): integer begin
      Writes := Writes + 1;
      return M;
    end function);
    return Make(Inner[0]);
  end function);
  if Writes <> 12 then panic('callback order'); end if;
  const Results := Counters.Map(function(Counter: function(): integer): integer begin
    return Counter();
  end function);
  if (Results.Length() <> 3) or (Results[0] <> 2) or (Results[1] <> 3) or (Results[2] <> 4) then
    panic('partial results');
  end if;
  const Selected := Results.Filter(function(N: integer): boolean begin return N >= 3; end function);
  const Total := Selected.Reduce(0, function(A: integer; B: integer): integer begin return A + B; end function);
  if Total <> 7 then panic('reduction'); end if;
  const Alias := Counters[0];
  if Alias() <> 3 then panic('alias'); end if;
  if Counters[0]() <> 4 then panic('retained state'); end if;
end procedure;
begin Exercise(); end."#,
    )))
    .unwrap();
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
    assert!(domain.collection_stats().allocated_cells >= 6);
    assert!(domain.collection_stats().completed_passes > 1);
    worker.retire_execution();
    assert_eq!(domain.collection_stats().live_cells, 0);
}

#[test]
fn immediate_debug_intrinsics_return_completed_callback_results() {
    let mut worker = Worker::new(Arc::new(image(
        "program T; function Double(N: integer): integer; begin return N * 2; end function; begin end.",
    )))
    .unwrap();
    let index = worker
        .executable
        .executable()
        .functions
        .iter()
        .position(|info| {
            worker
                .executable
                .executable()
                .strings
                .get(info.name)
                .is_some_and(|name| name.eq_ignore_ascii_case("Double"))
        })
        .unwrap();
    let function = Value::function(
        worker
            .function_target(FunctionId::new(index.try_into().unwrap()))
            .unwrap(),
        Vec::new(),
    );
    let result = worker
        .execute_debug_intrinsic(
            Intrinsic::Array(ArrayIntrinsic::Map),
            &[
                Value::Array(vec![Value::Integer(3), Value::Integer(5)].into()),
                function,
            ],
        )
        .unwrap();
    assert_eq!(
        result,
        Value::Array(vec![Value::Integer(6), Value::Integer(10)].into())
    );
}
