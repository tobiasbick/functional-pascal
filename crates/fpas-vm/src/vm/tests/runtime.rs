use std::sync::Arc;

use fpas_bytecode::{
    CodeRange, Constant, FunctionFlags, FunctionId, FunctionInfo, Instruction, InstructionAddress,
    Intrinsic, Opcode, ReturnConvention, SourceId, SourceRun, StringId, TaskIntrinsic,
    TimeIntrinsic, Value,
};
use fpas_diagnostics::codes::{
    RUNTIME_DIVISION_BY_ZERO, RUNTIME_MODULO_BY_ZERO, RUNTIME_NUMERIC_DOMAIN_ERROR,
    RUNTIME_VM_OPERAND_TYPE_MISMATCH, RUNTIME_VM_SHUTDOWN,
};

use super::*;
use crate::vm::Vm;

fn task_function(
    name: u32,
    start: u32,
    end: u32,
    register_count: u16,
    uses_spawn_tasks: bool,
) -> FunctionInfo {
    let mut flags = FunctionFlags::default();
    flags.uses_spawn_tasks = uses_spawn_tasks;
    FunctionInfo {
        name: StringId::new(name),
        code: CodeRange::new(InstructionAddress::new(start), InstructionAddress::new(end)),
        arity: 0,
        capture_count: 0,
        register_count,
        return_convention: ReturnConvention::Unit,
        flags,
        debug: fpas_bytecode::FunctionDebugInfo::default(),
    }
}

fn task_image(
    code: Vec<Instruction>,
    constants: Vec<Constant>,
    strings: Vec<&str>,
    functions: Vec<FunctionInfo>,
) -> fpas_bytecode::VerifiedExecutable {
    let mut executable = unverified(code, constants, strings, 1);
    executable.source_map.runs = functions
        .iter()
        .enumerate()
        .map(|(index, function)| SourceRun {
            instruction_start: function.code.start,
            source: SourceId::new(0),
            line: u32::try_from(index + 1).expect("fixture line"),
            column: 1,
        })
        .collect();
    executable.functions = functions;
    executable.verify().expect("task fixture must verify")
}

#[test]
fn scalar_failures_keep_codes_and_sparse_source_locations() {
    for (opcode, code) in [
        (Opcode::DivideInteger, RUNTIME_DIVISION_BY_ZERO),
        (Opcode::RemainderInteger, RUNTIME_MODULO_BY_ZERO),
    ] {
        let error = execute(verified(
            vec![
                abx(Opcode::LoadConstant, 0, 0),
                abx(Opcode::LoadConstant, 1, 1),
                abc(opcode, 2, 0, 1),
                return_unit(),
            ],
            vec![Constant::Integer(7), Constant::Integer(0)],
            vec!["root", "test.fpas"],
            3,
        ))
        .expect_err("zero divisor must fail");
        assert_eq!(error.code, code);
        assert_eq!(error.span.line(), 41);
        assert_eq!(error.span.column(), 7);
    }
}

#[test]
fn dynamic_type_mismatch_is_a_runtime_diagnostic() {
    let error = execute(verified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abx(Opcode::LoadConstant, 1, 1),
            abc(Opcode::AddDynamic, 2, 0, 1),
            return_unit(),
        ],
        vec![
            Constant::String(fpas_bytecode::StringId::new(2)),
            Constant::Integer(1),
        ],
        vec!["root", "test.fpas", "not numeric"],
        3,
    ))
    .expect_err("non-numeric dynamic operand must fail");
    assert_eq!(error.code, RUNTIME_VM_OPERAND_TYPE_MISMATCH);
}

#[test]
fn integer_domain_edges_fail_without_panicking() {
    for opcode in [
        Opcode::DivideInteger,
        Opcode::RemainderInteger,
        Opcode::NegateInteger,
        Opcode::NegateDynamic,
    ] {
        let operation = if matches!(opcode, Opcode::NegateInteger | Opcode::NegateDynamic) {
            abc(opcode, 2, 0, 0)
        } else {
            abc(opcode, 2, 0, 1)
        };
        let error = execute(verified(
            vec![
                abx(Opcode::LoadConstant, 0, 0),
                abx(Opcode::LoadConstant, 1, 1),
                operation,
                return_unit(),
            ],
            vec![Constant::Integer(i64::MIN), Constant::Integer(-1)],
            vec!["root", "test.fpas"],
            3,
        ))
        .expect_err("minimum-integer domain edge must fail");
        assert_eq!(error.code, RUNTIME_NUMERIC_DOMAIN_ERROR);
    }
}

#[test]
fn out_of_range_shift_is_a_numeric_domain_error() {
    let error = execute(verified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abx(Opcode::LoadConstant, 1, 1),
            abc(Opcode::ShiftLeftInteger, 2, 0, 1),
            return_unit(),
        ],
        vec![Constant::Integer(1), Constant::Integer(64)],
        vec!["root", "test.fpas"],
        3,
    ))
    .expect_err("shift amount 64 must fail");
    assert_eq!(error.code, RUNTIME_NUMERIC_DOMAIN_ERROR);
}

#[test]
fn main_task_yield_executes_without_a_pool() {
    let (value, _, count) = execute(verified(
        vec![abc(Opcode::Yield, 0, 0, 0), return_unit()],
        Vec::new(),
        vec!["root", "test.fpas"],
        1,
    ))
    .expect("main-task yield must be executable");
    assert_eq!(value, Value::Unit);
    assert_eq!(count, 2);
}

#[test]
fn shutdown_handle_cancels_register_execution_before_dispatch() {
    let executable = verified(
        vec![return_unit()],
        Vec::new(),
        vec!["root", "test.fpas"],
        1,
    );
    let mut vm = super::super::Vm::new(executable);
    vm.shutdown_handle().shutdown();
    let error = vm
        .run()
        .expect_err("pre-run cancellation must stop dispatch");
    assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN);
}

#[test]
fn shutdown_while_waiting_on_pending_task_returns_in_bounded_time() {
    let executable = task_image(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abc_aux(Opcode::SpawnTask, 1, 0, 0, 0),
            Instruction::abc(
                Opcode::Intrinsic,
                2,
                u16::from(Intrinsic::Task(TaskIntrinsic::Wait)),
                1,
                1,
            )
            .expect("wait instruction"),
            return_unit(),
            abx(Opcode::LoadConstant, 0, 1),
            Instruction::abc(
                Opcode::Intrinsic,
                fpas_bytecode::NO_REGISTER,
                u16::from(Intrinsic::Time(TimeIntrinsic::Sleep)),
                0,
                1,
            )
            .expect("sleep instruction"),
            return_unit(),
        ],
        vec![
            Constant::Function {
                function: FunctionId::new(1),
                task_bound: false,
            },
            Constant::Integer(60_000),
        ],
        vec!["root", "test.fpas", "sleeper"],
        vec![
            task_function(0, 0, 4, 3, true),
            task_function(2, 4, 7, 1, false),
        ],
    );
    let mut vm = Vm::new(executable);
    vm.pool_size = 0;
    let scheduler = Arc::clone(&vm.scheduler);
    let shutdown = vm.shutdown_handle();
    let (sender, receiver) = std::sync::mpsc::channel();
    let runner = std::thread::spawn(move || sender.send(vm.run()).expect("result receiver"));

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        match scheduler.poll_result(1) {
            crate::vm::TaskResultPoll::Pending => break,
            crate::vm::TaskResultPoll::Unknown if std::time::Instant::now() < deadline => {
                std::thread::yield_now();
            }
            crate::vm::TaskResultPoll::Unknown => panic!("task was not spawned before deadline"),
            _ => panic!("sleeper completed before shutdown"),
        }
    }
    shutdown.shutdown();

    let result = receiver
        .recv_timeout(std::time::Duration::from_secs(2))
        .expect("Wait must leave after shutdown");
    let error = result.expect_err("shutdown must fail the run");
    assert_eq!(error.code, RUNTIME_VM_SHUTDOWN);
    runner.join().expect("VM runner");
}

#[test]
fn inline_helped_task_failure_is_retained_before_run_abort() {
    let executable = task_image(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abc_aux(Opcode::SpawnTask, 2, 0, 0, 0),
            abx(Opcode::LoadConstant, 1, 1),
            abc_aux(Opcode::SpawnTask, 3, 1, 0, 0),
            Instruction::abc(
                Opcode::Intrinsic,
                4,
                u16::from(Intrinsic::Task(TaskIntrinsic::Wait)),
                2,
                1,
            )
            .expect("wait instruction"),
            return_unit(),
            abc(Opcode::Yield, 0, 0, 0),
            return_unit(),
            abx(Opcode::LoadConstant, 0, 2),
            abc(Opcode::Panic, 0, 0, 0),
            return_unit(),
        ],
        vec![
            Constant::Function {
                function: FunctionId::new(1),
                task_bound: false,
            },
            Constant::Function {
                function: FunctionId::new(2),
                task_bound: false,
            },
            Constant::String(StringId::new(4)),
        ],
        vec!["root", "test.fpas", "yielding", "failing", "helped panic"],
        vec![
            task_function(0, 0, 6, 5, true),
            task_function(2, 6, 8, 0, false),
            task_function(3, 8, 11, 1, false),
        ],
    );
    let mut vm = Vm::new(executable);
    vm.pool_size = 0;
    let scheduler = Arc::clone(&vm.scheduler);

    let error = vm.run().expect_err("helped task must fail the run");
    assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC);
    assert!(error.message.contains("helped panic"));
    assert!(matches!(
        scheduler.poll_result(2),
        crate::vm::TaskResultPoll::Failed(retained) if retained == error
    ));
    assert!(matches!(
        scheduler.poll_result(1),
        crate::vm::TaskResultPoll::Failed(retained) if retained == error
    ));
}

#[test]
fn shared_images_have_isolated_single_use_vm_instances() {
    let image = Arc::new(verified(
        vec![return_unit()],
        Vec::new(),
        vec!["root", "test.fpas"],
        0,
    ));
    let mut first = Vm::from_shared(Arc::clone(&image));
    let mut second = Vm::from_shared(image);

    assert_eq!(first.run().expect("first run").value, Value::Unit);
    assert_eq!(second.run().expect("isolated run").value, Value::Unit);
    let repeated = first.run().expect_err("VM instance must be single-use");
    assert_eq!(repeated.code, RUNTIME_VM_SHUTDOWN);
}

#[test]
fn control_flow_counts_only_dispatched_instructions() {
    let result = execute(verified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abx(Opcode::BranchIfFalse, 0, 4),
            abx(Opcode::LoadConstant, 1, 1),
            abx(Opcode::Jump, 0, 5),
            abx(Opcode::LoadConstant, 1, 0),
            return_unit(),
        ],
        vec![Constant::Boolean(true), Constant::Integer(9)],
        vec!["root", "test.fpas"],
        2,
    ))
    .expect("branch program should execute");
    assert_eq!(result.1[1], Value::Integer(9));
    assert_eq!(result.2, 5);
}
