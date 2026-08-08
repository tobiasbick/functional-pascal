use fpas_bytecode::{
    CodeRange, Constant, Executable, FunctionFlags, FunctionId, FunctionInfo, Instruction,
    InstructionAddress, Opcode, ReturnConvention, SourceId, SourceMap, SourceRun, StringId,
    StringTable, Value,
};
use fpas_diagnostics::codes::{RUNTIME_PROGRAM_PANIC, RUNTIME_VM_SHUTDOWN};

use crate::vm::CallbackSession;

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
            },
            FunctionInfo {
                name: StringId::new(1),
                code: CodeRange::new(InstructionAddress::new(1), InstructionAddress::new(3)),
                arity: 1,
                capture_count: 0,
                register_count: 2,
                return_convention: ReturnConvention::Value,
                flags: FunctionFlags::default(),
            },
            FunctionInfo {
                name: StringId::new(2),
                code: CodeRange::new(InstructionAddress::new(3), InstructionAddress::new(5)),
                arity: 0,
                capture_count: 0,
                register_count: 1,
                return_convention: ReturnConvention::Unit,
                flags: FunctionFlags::default(),
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
