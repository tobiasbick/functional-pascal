use std::sync::Arc;

use fpas_bytecode::{
    CodeRange, DebugType, Executable, FunctionFlags, FunctionId, FunctionInfo, Instruction,
    InstructionAddress, MemoryDomain, NO_REGISTER, Opcode, ReferenceRoot, ReturnConvention,
    SourceId, SourceMap, SourceRun, StringId, StringTable, Value, VariableReference,
    VerifiedExecutable, release_values,
};
use fpas_diagnostics::codes::RUNTIME_INVALID_TASK;

use super::{Worker, check_transfer};

fn executable() -> VerifiedExecutable {
    Executable {
        code: vec![Instruction::abc(Opcode::Return, NO_REGISTER, 0, 0, 0).unwrap()],
        functions: vec![FunctionInfo {
            name: StringId::new(0),
            code: CodeRange::new(InstructionAddress::new(0), InstructionAddress::new(1)),
            arity: 0,
            capture_count: 0,
            register_count: 0,
            return_convention: ReturnConvention::Unit,
            flags: FunctionFlags::default(),
            debug: fpas_bytecode::FunctionDebugInfo::default(),
        }],
        constants: Vec::new(),
        strings: StringTable::new(vec!["work".into(), "transfer.fpas".into()]),
        globals: Vec::new(),
        records: Vec::new(),
        enums: Vec::new(),
        enum_variants: Vec::new(),
        debug_types: vec![DebugType::Dynamic],
        distinct_types: Vec::new(),
        source_map: SourceMap {
            sources: vec![StringId::new(1)],
            runs: vec![SourceRun {
                instruction_start: InstructionAddress::new(0),
                source: SourceId::new(0),
                line: 1,
                column: 1,
            }],
        },
        entry: FunctionId::new(0),
    }
    .verify()
    .unwrap()
}

#[test]
fn immutable_data_callables_and_resource_handles_remain_transferable() {
    let target = executable().function_target(FunctionId::new(0)).unwrap();
    let values = [
        Value::Array(vec![Value::Integer(1), Value::option_some(Value::Unit)].into()),
        Value::OpaqueHandle(7),
        Value::Task(8),
        Value::function(target.clone(), vec![Value::Str("capture".into())]),
        Value::bound_function(target, Value::dict(vec![(Value::Integer(1), Value::Unit)])),
    ];
    assert_eq!(check_transfer(&values), Ok(()));
}

#[test]
fn every_contained_local_graph_is_rejected_before_transfer() {
    let owner = MemoryDomain::new();
    let cell = Value::Cell(owner.new_cell(Value::Integer(1)).unwrap());
    let target = executable().function_target(FunctionId::new(0)).unwrap();
    let values = [
        cell.clone(),
        Value::Array(vec![cell.clone()].into()),
        Value::dict(vec![(cell.clone(), Value::Unit)]),
        Value::option_some(Value::result_ok(cell.clone())),
        Value::function(target.clone(), vec![cell.clone()]),
        Value::bound_function(target.clone(), cell),
        Value::task_owned_function(target, Vec::new(), owner.id().clone()),
        Value::Reference(Arc::new(VariableReference {
            root: ReferenceRoot::Global {
                slot: 0,
                owner: owner.id().clone(),
            },
            path: Vec::new(),
        })),
    ];
    for value in values {
        let error = check_transfer(std::slice::from_ref(&value)).unwrap_err();
        assert!(error.contains("Cannot transfer value graph across a task boundary"));
    }
}

#[test]
fn worker_transfer_rejection_uses_the_existing_invalid_task_diagnostic() {
    let worker = Worker::new(Arc::new(executable())).unwrap();
    let value = Value::Cell(worker.domain.new_cell(Value::Unit).unwrap());
    assert!(worker.require_local_value(&value).is_ok());
    let error = worker.require_transferable_values(&[value]).unwrap_err();
    assert_eq!(error.code, RUNTIME_INVALID_TASK);
    assert!(error.message.contains("task boundary"));
}

#[test]
fn deeply_wrapped_transfer_rejections_use_a_bounded_stack() {
    std::thread::Builder::new()
        .stack_size(128 * 1024)
        .spawn(|| {
            let owner = MemoryDomain::new();
            let mut value = Value::Cell(owner.new_cell(Value::Unit).unwrap());
            for _ in 0..30_000 {
                value = Value::option_some(value);
            }
            assert!(check_transfer(std::slice::from_ref(&value)).is_err());
            release_values(vec![value]);
            assert_eq!(owner.registered_count(), 0);
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn owned_function_chain_rejection_drops_the_raw_graph_on_a_small_stack() {
    fn admit(value: Value) -> Result<(), String> {
        check_transfer(std::slice::from_ref(&value))?;
        Ok(())
    }

    std::thread::Builder::new()
        .stack_size(128 * 1024)
        .spawn(|| {
            let owner = MemoryDomain::new();
            let target = executable().function_target(FunctionId::new(0)).unwrap();
            let mut value = Value::Cell(owner.new_cell(Value::Unit).unwrap());
            for index in 0..30_000 {
                value = if index % 2 == 0 {
                    Value::function(target.clone(), vec![value])
                } else {
                    Value::bound_function(target.clone(), value)
                };
            }
            assert!(admit(value).unwrap_err().contains("Cannot transfer"));
            assert_eq!(owner.registered_count(), 0);
        })
        .unwrap()
        .join()
        .unwrap();
}
