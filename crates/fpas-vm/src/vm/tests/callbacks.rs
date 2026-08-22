use std::sync::Arc;

use fpas_bytecode::{
    CodeRange, Constant, Executable, FunctionFlags, FunctionId, FunctionInfo, Instruction,
    InstructionAddress, Opcode, ReturnConvention, SourceId, SourceMap, SourceRun, StringId,
    StringTable, Value,
};
use fpas_diagnostics::codes::{RUNTIME_INVALID_TASK, RUNTIME_PROGRAM_PANIC, RUNTIME_VM_SHUTDOWN};

use crate::vm::{CallbackSession, worker::Worker};

use super::calls::{FunctionSpec, abc, image};

fn callback_image() -> fpas_bytecode::VerifiedExecutable {
    let code = vec![
        Instruction::abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0).expect("root return"),
        Instruction::abc(Opcode::AddInteger, 1, 0, 0, 0).expect("double"),
        Instruction::abc(Opcode::Return, 1, 0, 0, 0).expect("callback return"),
        Instruction::abx(Opcode::LoadConstant, 0, 0).expect("panic message"),
        Instruction::abc(Opcode::Panic, 0, 0, 0, 0).expect("panic"),
    ];
    Executable {
        code,
        functions: vec![
            FunctionInfo {
                name: StringId::new(0),
                code: CodeRange::new(InstructionAddress::new(0), InstructionAddress::new(1)),
                arity: 0,
                capture_count: 0,
                register_count: 0,
                return_convention: ReturnConvention::Unit,
                flags: FunctionFlags::default(),
                debug: fpas_bytecode::FunctionDebugInfo::default(),
            },
            FunctionInfo {
                name: StringId::new(1),
                code: CodeRange::new(InstructionAddress::new(1), InstructionAddress::new(3)),
                arity: 1,
                capture_count: 0,
                register_count: 2,
                return_convention: ReturnConvention::Value,
                flags: FunctionFlags::default(),
                debug: fpas_bytecode::FunctionDebugInfo::default(),
            },
            FunctionInfo {
                name: StringId::new(2),
                code: CodeRange::new(InstructionAddress::new(3), InstructionAddress::new(5)),
                arity: 0,
                capture_count: 0,
                register_count: 1,
                return_convention: ReturnConvention::Unit,
                flags: FunctionFlags::default(),
                debug: fpas_bytecode::FunctionDebugInfo::default(),
            },
        ],
        constants: vec![Constant::String(StringId::new(3))],
        strings: StringTable::new(vec![
            "root".into(),
            "double".into(),
            "fail".into(),
            "boom".into(),
            "callbacks.fpas".into(),
        ]),
        globals: Vec::new(),
        records: Vec::new(),
        enums: Vec::new(),
        enum_variants: Vec::new(),
        debug_types: vec![fpas_bytecode::DebugType::Dynamic],
        source_map: SourceMap {
            sources: vec![StringId::new(4)],
            runs: vec![0_u32, 1, 3]
                .into_iter()
                .map(|start| SourceRun {
                    instruction_start: InstructionAddress::new(start),
                    source: SourceId::new(0),
                    line: start + 1,
                    column: 1,
                })
                .collect(),
        },
        entry: FunctionId::new(0),
    }
    .verify()
    .expect("callback image must verify")
}

#[test]
fn array_style_callbacks_use_numeric_targets_repeatedly() {
    let mut callbacks = CallbackSession::new(callback_image());
    let output = [1_i64, 2, 3]
        .into_iter()
        .map(|value| {
            callbacks
                .invoke(FunctionId::new(1), vec![Value::Integer(value)])
                .expect("callback")
                .value
        })
        .collect::<Vec<_>>();
    assert_eq!(
        output,
        vec![Value::Integer(2), Value::Integer(4), Value::Integer(6)]
    );
}

#[test]
fn callback_panic_unwinds_only_current_invocation() {
    let mut callbacks = CallbackSession::new(callback_image());
    let error = callbacks
        .invoke(FunctionId::new(2), Vec::new())
        .expect_err("callback must panic");
    assert_eq!(error.code, RUNTIME_PROGRAM_PANIC);
    assert_eq!(
        callbacks
            .invoke(FunctionId::new(1), vec![Value::Integer(4)])
            .expect("session remains usable")
            .value,
        Value::Integer(8)
    );
}

#[test]
fn cancellation_and_shutdown_reject_later_callbacks() {
    let mut cancelled = CallbackSession::new(callback_image());
    cancelled.cancel();
    assert_eq!(
        cancelled
            .invoke(FunctionId::new(1), vec![Value::Integer(1)])
            .expect_err("cancelled")
            .code,
        RUNTIME_VM_SHUTDOWN
    );

    let mut shutdown = CallbackSession::new(callback_image());
    shutdown.shutdown();
    assert_eq!(
        shutdown
            .invoke(FunctionId::new(1), vec![Value::Integer(1)])
            .expect_err("shutdown")
            .code,
        RUNTIME_VM_SHUTDOWN
    );
}

#[test]
fn hosted_callback_rejects_a_task_owned_function_from_a_foreign_task() {
    let worker = Worker::new(Arc::new(callback_image())).expect("worker");
    let function =
        Value::task_owned_function(FunctionId::new(1), "double".to_string(), Vec::new(), 7);

    let error = worker
        .call_callback_sync(&function, &[Value::Integer(3)])
        .expect_err("foreign task callback must fail");

    assert_eq!(error.code, RUNTIME_INVALID_TASK);
    assert!(error.message.contains("foreign task"), "{}", error.message);
}

#[test]
fn hosted_callback_prepends_a_bound_receiver() {
    let worker = Worker::new(Arc::new(callback_image())).expect("worker");
    let function = Value::bound_function(
        FunctionId::new(1),
        "Counter.Double".to_string(),
        Value::Integer(3),
    );

    assert_eq!(
        worker
            .call_callback_sync(&function, &[])
            .expect("bound callback"),
        Value::Integer(6)
    );
}

#[test]
fn hosted_callback_worker_keeps_the_owner_task_when_reused() {
    let executable = image(
        vec![
            abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
            abc(Opcode::CallValue, fpas_bytecode::NO_REGISTER, 0, 0, 0),
            abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
            abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
        ],
        Vec::new(),
        &[
            FunctionSpec {
                start: 0,
                end: 1,
                arity: 0,
                captures: 0,
                registers: 0,
                returns: ReturnConvention::Unit,
            },
            FunctionSpec {
                start: 1,
                end: 3,
                arity: 0,
                captures: 1,
                registers: 1,
                returns: ReturnConvention::Unit,
            },
            FunctionSpec {
                start: 3,
                end: 4,
                arity: 0,
                captures: 0,
                registers: 0,
                returns: ReturnConvention::Unit,
            },
        ],
    );
    let mut worker = Worker::new(Arc::new(executable)).expect("worker");
    worker.task_id = 7;
    let inner = Value::task_owned_function(FunctionId::new(2), "inner".to_string(), Vec::new(), 7);
    let outer = Value::task_owned_function(FunctionId::new(1), "outer".to_string(), vec![inner], 7);

    assert_eq!(
        worker
            .call_callback_sync(&outer, Vec::new())
            .expect("first same-task callback"),
        Value::Unit
    );
    assert_eq!(
        worker
            .call_callback_sync(&outer, Vec::new())
            .expect("reused same-task callback"),
        Value::Unit
    );
}
