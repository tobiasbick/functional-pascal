use fpas_bytecode::{
    CodeRange, Constant, DebugType, DebugTypeId, EnumLayout, EnumTypeId, EnumVariant, Executable,
    FunctionFlags, FunctionId, FunctionInfo, GlobalInfo, Instruction, InstructionAddress,
    Intrinsic, NO_REGISTER, Opcode, RecordField, RecordLayout, RecordProperty, ReturnConvention,
    SourceId, SourceMap, SourceRun, StringId, StringTable, intrinsic::ConsoleIntrinsic,
};

pub fn abc(opcode: Opcode, a: u16, b: u16, c: u16, auxiliary: u8) -> Instruction {
    Instruction::abc(opcode, a, b, c, auxiliary).expect("test opcode must use ABC")
}

pub fn abx(opcode: Opcode, a: u16, bx: u32) -> Instruction {
    Instruction::abx(opcode, a, bx).expect("test opcode must use ABx")
}

pub fn return_unit() -> Instruction {
    abc(Opcode::Return, NO_REGISTER, 0, 0, 0)
}

pub fn minimal_executable() -> Executable {
    Executable {
        code: vec![return_unit()],
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
        strings: StringTable::new(vec!["root".to_string(), "test.fpas".to_string()]),
        globals: Vec::new(),
        records: Vec::new(),
        enums: Vec::new(),
        enum_variants: Vec::new(),
        debug_types: vec![DebugType::Dynamic],
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
}

pub fn replace_root_code(executable: &mut Executable, code: Vec<Instruction>) {
    executable.code = code;
    let end = u32::try_from(executable.code.len()).expect("test code length must fit u32");
    executable.functions[0].code =
        CodeRange::new(InstructionAddress::new(0), InstructionAddress::new(end));
}

pub fn all_opcodes_executable() -> Executable {
    let mut code = Vec::new();
    for opcode in Opcode::ALL {
        let address = u32::try_from(code.len()).expect("test code length must fit u32");
        code.push(valid_instruction(opcode, address.saturating_add(1)));
    }
    let callee_start = u32::try_from(code.len()).expect("test code length must fit u32");
    code.push(return_unit());
    let code_end = u32::try_from(code.len()).expect("test code length must fit u32");

    let strings = vec![
        "root",
        "callee",
        "test.fpas",
        "record",
        "field",
        "enum",
        "variant",
        "global",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    Executable {
        code,
        functions: vec![
            FunctionInfo {
                name: StringId::new(0),
                code: CodeRange::new(
                    InstructionAddress::new(0),
                    InstructionAddress::new(callee_start),
                ),
                arity: 0,
                capture_count: 0,
                register_count: 16,
                return_convention: ReturnConvention::Unit,
                flags: FunctionFlags {
                    uses_spawn_tasks: true,
                },
                debug: fpas_bytecode::FunctionDebugInfo::default(),
            },
            FunctionInfo {
                name: StringId::new(1),
                code: CodeRange::new(
                    InstructionAddress::new(callee_start),
                    InstructionAddress::new(code_end),
                ),
                arity: 0,
                capture_count: 0,
                register_count: 0,
                return_convention: ReturnConvention::Unit,
                flags: FunctionFlags::default(),
                debug: fpas_bytecode::FunctionDebugInfo::default(),
            },
        ],
        constants: vec![
            Constant::Unit,
            Constant::String(StringId::new(0)),
            Constant::Function {
                function: FunctionId::new(1),
                task_bound: false,
            },
        ],
        strings: StringTable::new(strings),
        globals: vec![GlobalInfo {
            name: StringId::new(7),
            ty: DebugTypeId::new(0),
            mutable: true,
        }],
        records: vec![RecordLayout {
            name: StringId::new(3),
            fields: vec![RecordField {
                name: StringId::new(4),
                ty: DebugTypeId::new(0),
            }],
            properties: vec![RecordProperty {
                name: StringId::new(4),
                getter: StringId::new(1),
            }],
            methods: Vec::new(),
        }],
        enums: vec![EnumLayout {
            name: StringId::new(5),
        }],
        enum_variants: vec![EnumVariant {
            owner: EnumTypeId::new(0),
            name: StringId::new(6),
            fields: vec![StringId::new(4)],
            field_types: vec![DebugTypeId::new(0)],
        }],
        debug_types: vec![DebugType::Dynamic],
        source_map: SourceMap {
            sources: vec![StringId::new(2)],
            runs: vec![
                SourceRun {
                    instruction_start: InstructionAddress::new(0),
                    source: SourceId::new(0),
                    line: 1,
                    column: 1,
                },
                SourceRun {
                    instruction_start: InstructionAddress::new(callee_start),
                    source: SourceId::new(0),
                    line: 2,
                    column: 1,
                },
            ],
        },
        entry: FunctionId::new(0),
    }
}

fn valid_instruction(opcode: Opcode, next_address: u32) -> Instruction {
    match opcode {
        Opcode::LoadConstant => abx(opcode, 0, 0),
        Opcode::LoadGlobal | Opcode::StoreGlobal => abx(opcode, 0, 0),
        Opcode::Jump => abx(opcode, 0, next_address),
        Opcode::BranchIfFalse | Opcode::BranchIfTrue => abx(opcode, 0, next_address),
        Opcode::LoadUnit | Opcode::MakeNone => abc(opcode, 0, 0, 0, 0),
        Opcode::Move
        | Opcode::NegateInteger
        | Opcode::NegateReal
        | Opcode::NegateDynamic
        | Opcode::NotBoolean
        | Opcode::IntegerToReal
        | Opcode::MakeCell
        | Opcode::CellRead
        | Opcode::MakeOk
        | Opcode::MakeError
        | Opcode::MakeSome
        | Opcode::IsResultOk
        | Opcode::IsOptionSome
        | Opcode::UnwrapOk
        | Opcode::UnwrapError
        | Opcode::UnwrapSome => abc(opcode, 0, 1, 0, 0),
        Opcode::CellWrite => abc(opcode, 0, 1, 0, 0),
        Opcode::Return => return_unit(),
        Opcode::Panic => abc(opcode, 0, 0, 0, 0),
        Opcode::CallDirect => abc(opcode, NO_REGISTER, 1, 0, 0),
        Opcode::CallValue => abc(opcode, NO_REGISTER, 0, 0, 0),
        Opcode::MakeClosure => abc(opcode, 0, 1, 0, 0),
        Opcode::MakeArray | Opcode::MakeDictionary => abc(opcode, 0, 0, 0, 0),
        Opcode::MakeRecord | Opcode::MakeEnum => abc(opcode, 0, 0, 0, 0),
        Opcode::LoadField | Opcode::LoadEnumField => abc(opcode, 0, 1, 0, 0),
        Opcode::StoreField => abc(opcode, 0, 0, 1, 0),
        Opcode::UpdateRecord => abc(opcode, 0, 0, 0, 0),
        Opcode::TestVariant => abc(opcode, 0, 1, 0, 0),
        Opcode::Intrinsic => abc(
            opcode,
            NO_REGISTER,
            u16::from(Intrinsic::Console(ConsoleIntrinsic::ReadLn)),
            0,
            0,
        ),
        Opcode::SpawnTask => abc(opcode, 0, 1, 0, 0),
        Opcode::SpawnDetachedTask => abc(opcode, 0, 0, 0, 0),
        Opcode::Yield => abc(opcode, 0, 0, 0, 0),
        Opcode::ArrayPush => abc(opcode, 0, 1, 2, 0),
        Opcode::StoreGlobalIndexPath => abc(opcode, 0, 0, 0, 0),
        _ => abc(opcode, 0, 1, 2, 0),
    }
}
