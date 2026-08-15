//! Shared executable and session helpers for forced-return tests.

use fpas_bytecode::{
    CodeRange, Constant, DebugBinding, DebugBindingKind, DebugScope, DebugType, DebugTypeId,
    Executable, FunctionDebugInfo, FunctionFlags, FunctionId, FunctionInfo, Instruction,
    InstructionAddress, Register, ReturnConvention, SourceId, SourceMap, SourceRun, StringId,
    StringTable, VerifiedExecutable,
};

pub(super) use super::super::{
    DebugBinaryOperation, DebugErrorKind, DebugEvaluationLimits, DebugExecutionLimits,
    DebugExpression, DebugInspectionLimits, DebugRunResult, DebugSession, DebugSessionState,
    DebugStopReason, abc, point,
};

pub(super) fn binding(
    name: u32,
    register: u16,
    ty: u32,
    kind: DebugBindingKind,
    mutable: bool,
) -> DebugBinding {
    DebugBinding {
        name: StringId::new(name),
        type_name: StringId::new(8),
        ty: DebugTypeId::new(ty),
        register: Register::new(register).expect("register"),
        kind,
        mutable,
        scope: 0,
        declaration: Some(fpas_bytecode::DebugSourceLocation {
            source: SourceId::new(0),
            line: 10,
            column: 3,
        }),
        hidden: false,
        cell_backed: false,
    }
}

pub(super) fn debug_info(result: u32, points: &[(u32, u32)]) -> FunctionDebugInfo {
    FunctionDebugInfo {
        scopes: vec![DebugScope {
            id: 0,
            parent: None,
        }],
        bindings: Vec::new(),
        sequence_points: points
            .iter()
            .map(|(instruction, line)| point(*instruction, *line))
            .collect(),
        result_type: Some(DebugTypeId::new(result)),
        ..Default::default()
    }
}

pub(super) fn routine(
    name: u32,
    start: u32,
    end: u32,
    arity: u8,
    registers: u16,
    convention: ReturnConvention,
    debug: FunctionDebugInfo,
) -> FunctionInfo {
    FunctionInfo {
        name: StringId::new(name),
        code: CodeRange::new(InstructionAddress::new(start), InstructionAddress::new(end)),
        arity,
        capture_count: 0,
        register_count: registers,
        return_convention: convention,
        flags: FunctionFlags::default(),
        debug,
    }
}

pub(super) fn executable(
    code: Vec<Instruction>,
    functions: Vec<FunctionInfo>,
    constants: Vec<Constant>,
    runs: Vec<(u32, u32)>,
    debug_types: Vec<DebugType>,
) -> VerifiedExecutable {
    Executable {
        code,
        functions,
        constants,
        strings: StringTable::new(
            [
                "root",
                "compute",
                "announce",
                "task",
                "test.fpas",
                "Answer",
                "Offset",
                "Value",
                "Integer",
                "Marker",
                "Items",
                "boom",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
        ),
        globals: Vec::new(),
        records: Vec::new(),
        enums: Vec::new(),
        enum_variants: Vec::new(),
        debug_types,
        source_map: SourceMap {
            sources: vec![StringId::new(4)],
            runs: runs
                .into_iter()
                .map(|(instruction, line)| SourceRun {
                    instruction_start: InstructionAddress::new(instruction),
                    source: SourceId::new(0),
                    line,
                    column: 3,
                })
                .collect(),
        },
        entry: FunctionId::new(0),
    }
    .verify()
    .expect("forced-return fixture executable")
}

pub(super) fn name(name: &str) -> DebugExpression {
    DebugExpression::Name(name.to_string())
}

pub(super) fn int_expr(value: i64) -> DebugExpression {
    DebugExpression::Integer(value)
}

pub(super) fn stopped(result: DebugRunResult) -> crate::DebugStop {
    let DebugRunResult::Stopped(stop) = result else {
        panic!("expected stopped debug result")
    };
    stop
}

pub(super) fn stop_in_callee(session: &mut DebugSession, name: &str) -> u64 {
    for _ in 0..64 {
        let stack = session.stack(0, 16).expect("stack");
        if stack
            .items
            .first()
            .is_some_and(|frame| frame.name == name && frame.depth == 0)
            && session.last_stop().call_depth >= 1
        {
            return stack.items[0].id;
        }
        let _ = stopped(session.step_into().expect("step into callee"));
    }
    panic!("{name} never became the active callee")
}

pub(super) fn frame_at_depth(session: &mut DebugSession, depth: usize) -> u64 {
    session
        .stack(0, 16)
        .expect("stack")
        .items
        .into_iter()
        .find(|frame| frame.depth == depth)
        .unwrap_or_else(|| panic!("stack should include depth {depth}"))
        .id
}

pub(super) fn compiled_fixture() -> VerifiedExecutable {
    const SOURCE: &str =
        include_str!("../../../../../../../tests/debugger/fixtures/forced_return.fpas");
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    fpas_compiler::compile(&program).expect("compile forced-return fixture")
}

pub(super) fn named<'a>(
    variables: &'a [crate::DebugVariable],
    name: &str,
) -> &'a crate::DebugVariable {
    variables
        .iter()
        .find(|variable| variable.name == name)
        .unwrap_or_else(|| panic!("{name} should exist"))
}

pub(super) fn scope_reference(session: &mut DebugSession, scope_name: &str) -> u64 {
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    session
        .scopes(frame)
        .expect("scopes")
        .into_iter()
        .find(|scope| scope.name == scope_name)
        .expect("requested scope")
        .variables_reference
}
