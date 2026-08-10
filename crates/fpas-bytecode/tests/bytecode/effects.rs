use fpas_bytecode::{
    CodeRange, DebugEffectSet, FunctionFlags, FunctionId, FunctionInfo, InstructionAddress,
    Intrinsic, NO_REGISTER, Opcode, ReturnConvention, SourceRun, StringId, analyze_debug_effects,
    intrinsic::ConsoleIntrinsic,
};

use super::support::{abc, minimal_executable, return_unit};

#[test]
fn direct_call_effects_close_transitively_across_recursive_graphs() {
    let mut executable = minimal_executable();
    executable.strings = fpas_bytecode::StringTable::new(vec![
        "root".into(),
        "middle".into(),
        "leaf".into(),
        "test.fpas".into(),
    ]);
    executable.code = vec![
        abc(Opcode::CallDirect, NO_REGISTER, 1, 0, 0),
        return_unit(),
        abc(Opcode::CallDirect, NO_REGISTER, 2, 0, 0),
        return_unit(),
        abc(Opcode::CallDirect, NO_REGISTER, 1, 0, 0),
        abc(
            Opcode::Intrinsic,
            0,
            u16::from(Intrinsic::Console(ConsoleIntrinsic::ReadLn)),
            0,
            0,
        ),
        return_unit(),
    ];
    executable.functions = [(0, 0, 2, 0), (1, 2, 4, 0), (2, 4, 7, 1)]
        .into_iter()
        .map(|(name, start, end, registers)| FunctionInfo {
            name: StringId::new(name),
            code: CodeRange::new(InstructionAddress::new(start), InstructionAddress::new(end)),
            arity: 0,
            capture_count: 0,
            register_count: registers,
            return_convention: ReturnConvention::Unit,
            flags: FunctionFlags::default(),
            debug: fpas_bytecode::FunctionDebugInfo::default(),
        })
        .collect();
    executable.source_map.sources = vec![StringId::new(3)];
    executable.source_map.runs = [0, 2, 4]
        .into_iter()
        .map(|instruction_start| SourceRun {
            instruction_start: InstructionAddress::new(instruction_start),
            source: fpas_bytecode::SourceId::new(0),
            line: instruction_start + 1,
            column: 1,
        })
        .collect();
    executable.entry = FunctionId::new(0);

    let verified = executable.verify().expect("effect graph fixture");
    let summaries = analyze_debug_effects(&verified);

    assert_eq!(summaries.len(), 3);
    assert!(!summaries[0].local.contains(DebugEffectSet::HOST_IO));
    assert!(!summaries[1].local.contains(DebugEffectSet::HOST_IO));
    assert!(summaries[2].local.contains(DebugEffectSet::HOST_IO));
    assert!(summaries.iter().all(|summary| {
        summary.transitive.contains(DebugEffectSet::HOST_IO) && !summary.transitive.is_debug_safe()
    }));
}
