use fpas_bytecode::{
    CodeRange, Constant, DebugBinding, DebugBindingId, DebugBindingKind, DebugCaptureKind,
    DebugCaptureSource, DebugScope, DebugTypeId, Executable, FunctionFlags, FunctionId,
    FunctionInfo, Instruction, InstructionAddress, Opcode, Register, ReturnConvention, SourceId,
    SourceMap, SourceRun, StringId, StringTable, Value,
};
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;

use crate::vm::Vm;

use super::support::abx;

#[derive(Clone, Copy)]
/// Compact function metadata used by hand-authored call bytecode tests.
pub(super) struct FunctionSpec {
    pub(super) start: u32,
    pub(super) end: u32,
    pub(super) arity: u8,
    pub(super) captures: u16,
    pub(super) registers: u16,
    pub(super) returns: ReturnConvention,
}

/// Encodes one ABC instruction for a hand-authored call test.
pub(super) fn abc(opcode: Opcode, a: u16, b: u16, c: u16, auxiliary: u8) -> Instruction {
    Instruction::abc(opcode, a, b, c, auxiliary).expect("test instruction must encode")
}

/// Builds and verifies an executable from compact call-test function specifications.
pub(super) fn image(
    code: Vec<Instruction>,
    constants: Vec<Constant>,
    specs: &[FunctionSpec],
) -> fpas_bytecode::VerifiedExecutable {
    let mut names = (0..specs.len())
        .map(|index| format!("f{index}"))
        .collect::<Vec<_>>();
    names.push("register-calls.fpas".to_string());
    let mut functions = specs
        .iter()
        .enumerate()
        .map(|(index, spec)| FunctionInfo {
            name: StringId::try_from_index(index).expect("test name id must fit"),
            code: CodeRange::new(
                InstructionAddress::new(spec.start),
                InstructionAddress::new(spec.end),
            ),
            arity: spec.arity,
            capture_count: spec.captures,
            register_count: spec.registers,
            return_convention: spec.returns,
            flags: FunctionFlags::default(),
            debug: fpas_bytecode::FunctionDebugInfo::default(),
        })
        .collect::<Vec<_>>();
    attach_cell_capture_provenance(&mut functions);
    Executable {
        code,
        functions,
        constants,
        strings: StringTable::new(names),
        globals: Vec::new(),
        records: Vec::new(),
        enums: Vec::new(),
        enum_variants: Vec::new(),
        debug_types: vec![fpas_bytecode::DebugType::Dynamic],
        source_map: SourceMap {
            sources: vec![StringId::try_from_index(specs.len()).expect("source name id must fit")],
            runs: specs
                .iter()
                .map(|spec| SourceRun {
                    instruction_start: InstructionAddress::new(spec.start),
                    source: SourceId::new(0),
                    line: spec.start.saturating_add(1),
                    column: 1,
                })
                .collect(),
        },
        entry: FunctionId::new(0),
    }
    .verify()
    .expect("test executable must verify")
}

fn attach_cell_capture_provenance(functions: &mut [FunctionInfo]) {
    let slots = functions
        .iter()
        .map(|function| function.capture_count)
        .max()
        .unwrap_or(0);
    if slots == 0 {
        return;
    }
    functions[0].register_count = functions[0].register_count.max(slots);
    functions[0].debug.scopes = vec![DebugScope {
        id: 0,
        parent: None,
    }];
    functions[0].debug.bindings = (0..slots)
        .map(|index| DebugBinding {
            name: StringId::new(0),
            type_name: StringId::new(0),
            ty: DebugTypeId::new(0),
            register: Register::new(index).expect("register"),
            kind: DebugBindingKind::Local,
            mutable: true,
            scope: 0,
            declaration: None,
            hidden: false,
            cell_backed: true,
            initializer: None,
        })
        .collect();
    for function in functions.iter_mut().skip(1) {
        if function.capture_count == 0 {
            continue;
        }
        function.debug.lexical_owner = Some(FunctionId::new(0));
        function.debug.capture_sources = (0..function.capture_count)
            .map(|index| DebugCaptureSource {
                binding: DebugBindingId::new(u32::from(index)),
                ty: DebugTypeId::new(0),
                kind: DebugCaptureKind::Cell,
            })
            .collect();
    }
}

#[test]
fn direct_call_uses_argument_window_and_return_destination() {
    let executable = image(
        vec![
            abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
            abx(Opcode::LoadConstant, 0, 0),
            abx(Opcode::LoadConstant, 1, 1),
            abc(Opcode::CallDirect, 2, 2, 0, 2),
            abc(Opcode::Return, 2, 0, 0, 0),
            abc(Opcode::AddInteger, 2, 0, 1, 0),
            abc(Opcode::Return, 2, 0, 0, 0),
        ],
        vec![Constant::Integer(20), Constant::Integer(22)],
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
                end: 5,
                arity: 0,
                captures: 0,
                registers: 3,
                returns: ReturnConvention::Value,
            },
            FunctionSpec {
                start: 5,
                end: 7,
                arity: 2,
                captures: 0,
                registers: 3,
                returns: ReturnConvention::Value,
            },
        ],
    );

    let result = Vm::new(executable)
        .call(FunctionId::new(1), Vec::new())
        .expect("direct call should succeed");
    assert_eq!(result.value, Value::Integer(42));
}

#[test]
fn recursive_call_and_early_return_restore_each_frame() {
    let executable = image(
        vec![
            abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
            abx(Opcode::LoadConstant, 1, 1),
            abc(Opcode::LessEqualInteger, 2, 0, 1, 0),
            abx(Opcode::BranchIfFalse, 2, 5),
            abc(Opcode::Return, 1, 0, 0, 0),
            abc(Opcode::SubtractInteger, 3, 0, 1, 0),
            abc(Opcode::CallDirect, 4, 1, 3, 1),
            abc(Opcode::MultiplyInteger, 5, 0, 4, 0),
            abc(Opcode::Return, 5, 0, 0, 0),
        ],
        vec![Constant::Integer(5), Constant::Integer(1)],
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
                end: 9,
                arity: 1,
                captures: 0,
                registers: 6,
                returns: ReturnConvention::Value,
            },
        ],
    );

    assert_eq!(
        Vm::new(executable)
            .call(FunctionId::new(1), vec![Value::Integer(5)])
            .expect("recursion should succeed")
            .value,
        Value::Integer(120)
    );
}

#[test]
fn numeric_function_value_calls_without_name_lookup() {
    let executable = image(
        vec![
            abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
            abx(Opcode::LoadConstant, 0, 0),
            abx(Opcode::LoadConstant, 1, 1),
            abc(Opcode::CallValue, 2, 0, 1, 1),
            abc(Opcode::Return, 2, 0, 0, 0),
            abc(Opcode::AddInteger, 1, 0, 0, 0),
            abc(Opcode::Return, 1, 0, 0, 0),
        ],
        vec![
            Constant::Function {
                function: FunctionId::new(2),
                task_bound: false,
            },
            Constant::Integer(21),
        ],
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
                end: 5,
                arity: 0,
                captures: 0,
                registers: 3,
                returns: ReturnConvention::Value,
            },
            FunctionSpec {
                start: 5,
                end: 7,
                arity: 1,
                captures: 0,
                registers: 2,
                returns: ReturnConvention::Value,
            },
        ],
    );

    assert_eq!(
        Vm::new(executable)
            .call(FunctionId::new(1), Vec::new())
            .expect("value call should succeed")
            .value,
        Value::Integer(42)
    );
}

#[test]
fn bound_function_value_prepends_receiver_before_visible_arguments() {
    let executable = image(
        vec![
            abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
            abc(Opcode::CallValue, 2, 0, 1, 1),
            abc(Opcode::Return, 2, 0, 0, 0),
            abc(Opcode::AddInteger, 2, 0, 1, 0),
            abc(Opcode::Return, 2, 0, 0, 0),
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
                arity: 2,
                captures: 0,
                registers: 3,
                returns: ReturnConvention::Value,
            },
            FunctionSpec {
                start: 3,
                end: 5,
                arity: 2,
                captures: 0,
                registers: 3,
                returns: ReturnConvention::Value,
            },
        ],
    );
    let bound = Value::bound_function(
        FunctionId::new(2),
        "Counter.Add".to_string(),
        Value::Integer(10),
    );

    assert_eq!(
        Vm::new(executable)
            .call(FunctionId::new(1), vec![bound, Value::Integer(5)])
            .expect("bound value call should succeed")
            .value,
        Value::Integer(15)
    );
}

#[test]
fn mutable_capture_cell_is_shared_in_semantic_capture_order() {
    let executable = image(
        vec![
            abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
            abx(Opcode::LoadConstant, 0, 0),
            abc(Opcode::MakeCell, 0, 0, 0, 0),
            abc(Opcode::MakeClosure, 1, 2, 0, 1),
            abc(Opcode::CallValue, 2, 1, 0, 0),
            abc(Opcode::CallValue, 3, 1, 0, 0),
            abc(Opcode::Return, 3, 0, 0, 0),
            abc(Opcode::CellRead, 1, 0, 0, 0),
            abx(Opcode::LoadConstant, 2, 1),
            abc(Opcode::AddInteger, 3, 1, 2, 0),
            abc(Opcode::CellWrite, 0, 3, 0, 0),
            abc(Opcode::Return, 3, 0, 0, 0),
        ],
        vec![Constant::Integer(40), Constant::Integer(1)],
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
                end: 7,
                arity: 0,
                captures: 0,
                registers: 4,
                returns: ReturnConvention::Value,
            },
            FunctionSpec {
                start: 7,
                end: 12,
                arity: 0,
                captures: 1,
                registers: 4,
                returns: ReturnConvention::Value,
            },
        ],
    );

    assert_eq!(
        Vm::new(executable)
            .call(FunctionId::new(1), Vec::new())
            .expect("closure should succeed")
            .value,
        Value::Integer(42)
    );
}

#[test]
fn recursion_limit_fails_deterministically() {
    let executable = image(
        vec![
            abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
            abc(Opcode::CallDirect, fpas_bytecode::NO_REGISTER, 1, 0, 0),
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
                captures: 0,
                registers: 0,
                returns: ReturnConvention::Unit,
            },
        ],
    );

    let error = Vm::new(executable)
        .call(FunctionId::new(1), Vec::new())
        .expect_err("infinite recursion must fail");
    assert_eq!(error.code, RUNTIME_INTRINSIC_STACK_STATE_ERROR);
}

#[test]
fn method_receiver_is_an_ordinary_first_register_argument() {
    let executable = image(
        vec![
            abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
            abc(Opcode::Return, 1, 0, 0, 0),
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
                end: 2,
                arity: 2,
                captures: 0,
                registers: 2,
                returns: ReturnConvention::Value,
            },
        ],
    );
    let receiver = Value::Record(fpas_bytecode::SharedRecord::new(
        std::sync::Arc::new(fpas_bytecode::RuntimeRecordLayout {
            record: fpas_bytecode::RecordTypeId::new(0),
            type_name: "Point".to_string(),
            fields: vec!["x".to_string()],
        }),
        vec![Value::Integer(1)],
    ));
    let result = Vm::new(executable)
        .call(FunctionId::new(1), vec![receiver, Value::Integer(42)])
        .expect("method-shaped call should succeed");
    assert_eq!(result.value, Value::Integer(42));
}

#[test]
fn callback_entry_rejects_wrong_arity_and_invalid_function_id() {
    let executable = image(
        vec![
            abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
            abc(Opcode::Return, 0, 0, 0, 0),
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
                end: 2,
                arity: 1,
                captures: 0,
                registers: 1,
                returns: ReturnConvention::Value,
            },
        ],
    );
    assert!(
        Vm::new(executable.clone())
            .call(FunctionId::new(1), Vec::new())
            .is_err()
    );
    assert!(
        Vm::new(executable)
            .call(FunctionId::new(99), Vec::new())
            .is_err()
    );
}
