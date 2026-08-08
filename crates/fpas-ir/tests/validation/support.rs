use super::*;

pub const UNIT: TypeId = TypeId::new(0);
pub const BOOLEAN: TypeId = TypeId::new(1);
pub const INTEGER: TypeId = TypeId::new(2);
pub const STRING: TypeId = TypeId::new(3);
pub const RECORD: TypeId = TypeId::new(4);
pub const ENUM: TypeId = TypeId::new(5);
pub const FUNCTION: TypeId = TypeId::new(6);
pub const CELL: TypeId = TypeId::new(7);
pub const TASK: TypeId = TypeId::new(8);
pub const DYNAMIC: TypeId = TypeId::new(9);
pub const REAL: TypeId = TypeId::new(10);
pub const ARRAY: TypeId = TypeId::new(11);
pub const DICTIONARY: TypeId = TypeId::new(12);
pub const RESULT: TypeId = TypeId::new(13);
pub const OPTION: TypeId = TypeId::new(14);

pub fn root(blocks: Vec<BasicBlock>) -> Function {
    Function {
        id: FunctionId::new(0),
        name: "root".to_string(),
        signature: FunctionSignature {
            parameters: Vec::new(),
            result: UNIT,
        },
        parameters: Vec::new(),
        locals: vec![
            Local {
                id: LocalId::new(0),
                ty: INTEGER,
                mutable: true,
                capture: None,
            },
            Local {
                id: LocalId::new(1),
                ty: ARRAY,
                mutable: true,
                capture: None,
            },
        ],
        captures: Vec::new(),
        blocks,
        entry: BlockId::new(0),
        max_call_arguments: 1,
        can_spawn_tasks: true,
    }
}

pub fn return_unit_block() -> BasicBlock {
    BasicBlock {
        id: BlockId::new(0),
        parameters: Vec::new(),
        instructions: Vec::new(),
        terminators: vec![Terminator::Return(None)],
    }
}

pub fn types() -> Vec<TypeDefinition> {
    vec![
        TypeDefinition {
            id: UNIT,
            kind: IrType::Unit,
        },
        TypeDefinition {
            id: BOOLEAN,
            kind: IrType::Boolean,
        },
        TypeDefinition {
            id: INTEGER,
            kind: IrType::Integer,
        },
        TypeDefinition {
            id: STRING,
            kind: IrType::String,
        },
        TypeDefinition {
            id: RECORD,
            kind: IrType::Record(RecordLayoutId::new(0)),
        },
        TypeDefinition {
            id: ENUM,
            kind: IrType::Enum(EnumLayoutId::new(0)),
        },
        TypeDefinition {
            id: FUNCTION,
            kind: IrType::Function {
                parameters: vec![INTEGER],
                result: INTEGER,
            },
        },
        TypeDefinition {
            id: CELL,
            kind: IrType::Cell(INTEGER),
        },
        TypeDefinition {
            id: TASK,
            kind: IrType::Task(INTEGER),
        },
        TypeDefinition {
            id: DYNAMIC,
            kind: IrType::Dynamic,
        },
        TypeDefinition {
            id: REAL,
            kind: IrType::Real,
        },
        TypeDefinition {
            id: ARRAY,
            kind: IrType::Array(INTEGER),
        },
        TypeDefinition {
            id: DICTIONARY,
            kind: IrType::Dictionary {
                key: STRING,
                value: INTEGER,
            },
        },
        TypeDefinition {
            id: RESULT,
            kind: IrType::Result {
                ok: INTEGER,
                error: STRING,
            },
        },
        TypeDefinition {
            id: OPTION,
            kind: IrType::Option(INTEGER),
        },
    ]
}

pub fn scalar_program() -> Program {
    Program {
        types: types(),
        globals: Vec::new(),
        record_layouts: vec![RecordLayout {
            id: RecordLayoutId::new(0),
            name: "Point".to_string(),
            fields: vec![RecordField {
                id: FieldId::new(0),
                name: "x".to_string(),
                ty: INTEGER,
            }],
        }],
        enum_layouts: vec![EnumLayout {
            id: EnumLayoutId::new(0),
            name: "Choice".to_string(),
            variants: vec![EnumVariant {
                id: VariantId::new(0),
                name: "Some".to_string(),
                field_names: vec!["value".to_string()],
                fields: vec![INTEGER],
            }],
        }],
        intrinsics: Vec::new(),
        functions: vec![root(vec![return_unit_block()])],
        entry: FunctionId::new(0),
    }
}

pub fn value(id: u32, ty: TypeId) -> ValueDefinition {
    ValueDefinition {
        id: ValueId::new(id),
        ty,
    }
}

pub fn all_operations_program() -> Program {
    let mut program = scalar_program();
    program.globals = vec![
        Global {
            id: GlobalId::new(0),
            name: "answer".to_string(),
            ty: INTEGER,
            mutable: true,
        },
        Global {
            id: GlobalId::new(1),
            name: "cell".to_string(),
            ty: CELL,
            mutable: false,
        },
    ];
    program.intrinsics = vec![IntrinsicSignature {
        id: IntrinsicId::new(0),
        parameters: vec![INTEGER],
        variadic: false,
        result: INTEGER,
    }];
    program.functions.push(Function {
        id: FunctionId::new(1),
        name: "increment".to_string(),
        signature: FunctionSignature {
            parameters: vec![INTEGER],
            result: INTEGER,
        },
        parameters: vec![value(100, INTEGER)],
        locals: Vec::new(),
        captures: Vec::new(),
        blocks: vec![BasicBlock {
            id: BlockId::new(10),
            parameters: Vec::new(),
            instructions: Vec::new(),
            terminators: vec![Terminator::Return(Some(ValueId::new(100)))],
        }],
        entry: BlockId::new(10),
        max_call_arguments: 1,
        can_spawn_tasks: false,
    });
    let instructions = vec![
        Instruction {
            source: None,
            result: Some(value(1, INTEGER)),
            operation: Operation::Const(Constant::Integer(1)),
        },
        Instruction {
            source: None,
            result: Some(value(2, BOOLEAN)),
            operation: Operation::Const(Constant::Boolean(true)),
        },
        Instruction {
            source: None,
            result: Some(value(3, STRING)),
            operation: Operation::Const(Constant::String("x".to_string())),
        },
        Instruction {
            source: None,
            result: Some(value(4, INTEGER)),
            operation: Operation::ReadLocal(LocalId::new(0)),
        },
        Instruction {
            source: None,
            result: None,
            operation: Operation::WriteLocal {
                value: ValueId::new(1),
                local: LocalId::new(0),
            },
        },
        Instruction {
            source: None,
            result: Some(value(5, INTEGER)),
            operation: Operation::Binary {
                operation: BinaryOperation::AddInteger,
                left: ValueId::new(1),
                right: ValueId::new(4),
            },
        },
        Instruction {
            source: None,
            result: Some(value(6, INTEGER)),
            operation: Operation::CallDirect {
                function: FunctionId::new(1),
                arguments: vec![ValueId::new(1)],
            },
        },
        Instruction {
            source: None,
            result: Some(value(7, FUNCTION)),
            operation: Operation::MakeClosure {
                function: FunctionId::new(1),
                captures: Vec::new(),
            },
        },
        Instruction {
            source: None,
            result: Some(value(8, INTEGER)),
            operation: Operation::CallValue {
                callee: ValueId::new(7),
                arguments: vec![ValueId::new(1)],
            },
        },
        Instruction {
            source: None,
            result: Some(value(9, INTEGER)),
            operation: Operation::LoadGlobal(GlobalId::new(0)),
        },
        Instruction {
            source: None,
            result: None,
            operation: Operation::StoreGlobal {
                global: GlobalId::new(0),
                value: ValueId::new(9),
            },
        },
        Instruction {
            source: None,
            result: Some(value(10, RECORD)),
            operation: Operation::MakeRecord {
                layout: RecordLayoutId::new(0),
                fields: vec![ValueId::new(1)],
            },
        },
        Instruction {
            source: None,
            result: Some(value(11, INTEGER)),
            operation: Operation::LoadField {
                record: ValueId::new(10),
                layout: RecordLayoutId::new(0),
                field: FieldId::new(0),
            },
        },
        Instruction {
            source: None,
            result: None,
            operation: Operation::StoreField {
                record: ValueId::new(10),
                layout: RecordLayoutId::new(0),
                field: FieldId::new(0),
                value: ValueId::new(11),
            },
        },
        Instruction {
            source: None,
            result: Some(value(12, ENUM)),
            operation: Operation::MakeEnum {
                layout: EnumLayoutId::new(0),
                variant: VariantId::new(0),
                fields: vec![ValueId::new(1)],
            },
        },
        Instruction {
            source: None,
            result: Some(value(13, BOOLEAN)),
            operation: Operation::TestVariant {
                value: ValueId::new(12),
                layout: EnumLayoutId::new(0),
                variant: VariantId::new(0),
            },
        },
        Instruction {
            source: None,
            result: Some(value(14, INTEGER)),
            operation: Operation::Intrinsic {
                intrinsic: IntrinsicId::new(0),
                arguments: vec![ValueId::new(1)],
            },
        },
        Instruction {
            source: None,
            result: Some(value(15, CELL)),
            operation: Operation::LoadGlobal(GlobalId::new(1)),
        },
        Instruction {
            source: None,
            result: Some(value(16, INTEGER)),
            operation: Operation::CellRead(ValueId::new(15)),
        },
        Instruction {
            source: None,
            result: None,
            operation: Operation::CellWrite {
                cell: ValueId::new(15),
                value: ValueId::new(16),
            },
        },
        Instruction {
            source: None,
            result: Some(value(17, TASK)),
            operation: Operation::SpawnTask {
                callee: ValueId::new(7),
                arguments: vec![ValueId::new(1)],
            },
        },
        Instruction {
            source: None,
            result: None,
            operation: Operation::SpawnDetachedTask {
                callee: ValueId::new(7),
                arguments: vec![ValueId::new(1)],
            },
        },
        Instruction {
            source: None,
            result: None,
            operation: Operation::Yield,
        },
        Instruction {
            source: None,
            result: Some(value(18, ARRAY)),
            operation: Operation::MakeArray(vec![ValueId::new(1)]),
        },
        Instruction {
            source: None,
            result: Some(value(19, INTEGER)),
            operation: Operation::IndexGet {
                collection: ValueId::new(18),
                index: ValueId::new(1),
            },
        },
        Instruction {
            source: None,
            result: Some(value(20, ARRAY)),
            operation: Operation::IndexSet {
                collection: ValueId::new(18),
                index: ValueId::new(1),
                value: ValueId::new(19),
            },
        },
        Instruction {
            source: None,
            result: Some(value(21, BOOLEAN)),
            operation: Operation::Contains {
                value: ValueId::new(1),
                collection: ValueId::new(18),
            },
        },
        Instruction {
            source: None,
            result: Some(value(22, DICTIONARY)),
            operation: Operation::MakeDictionary(vec![(ValueId::new(3), ValueId::new(1))]),
        },
        Instruction {
            source: None,
            result: Some(value(23, INTEGER)),
            operation: Operation::IndexGet {
                collection: ValueId::new(22),
                index: ValueId::new(3),
            },
        },
        Instruction {
            source: None,
            result: Some(value(24, RECORD)),
            operation: Operation::UpdateRecord {
                record: ValueId::new(10),
                layout: RecordLayoutId::new(0),
                fields: vec![(FieldId::new(0), ValueId::new(1))],
            },
        },
        Instruction {
            source: None,
            result: Some(value(25, RESULT)),
            operation: Operation::MakeOk(ValueId::new(1)),
        },
        Instruction {
            source: None,
            result: Some(value(26, RESULT)),
            operation: Operation::MakeError(ValueId::new(3)),
        },
        Instruction {
            source: None,
            result: Some(value(27, BOOLEAN)),
            operation: Operation::IsResultOk(ValueId::new(25)),
        },
        Instruction {
            source: None,
            result: Some(value(28, INTEGER)),
            operation: Operation::UnwrapOk(ValueId::new(25)),
        },
        Instruction {
            source: None,
            result: Some(value(29, OPTION)),
            operation: Operation::MakeSome(ValueId::new(1)),
        },
        Instruction {
            source: None,
            result: Some(value(30, OPTION)),
            operation: Operation::MakeNone,
        },
        Instruction {
            source: None,
            result: Some(value(31, BOOLEAN)),
            operation: Operation::IsOptionSome(ValueId::new(29)),
        },
        Instruction {
            source: None,
            result: Some(value(32, INTEGER)),
            operation: Operation::UnwrapSome(ValueId::new(29)),
        },
        Instruction {
            source: None,
            result: Some(value(33, INTEGER)),
            operation: Operation::LoadEnumField {
                value: ValueId::new(12),
                layout: EnumLayoutId::new(0),
                variant: VariantId::new(0),
                field: FieldId::new(0),
            },
        },
        Instruction {
            source: None,
            result: Some(value(34, UNIT)),
            operation: Operation::ArrayPush {
                local: LocalId::new(1),
                value: ValueId::new(1),
            },
        },
    ];
    program.functions[0] = root(vec![BasicBlock {
        id: BlockId::new(0),
        parameters: Vec::new(),
        instructions,
        terminators: vec![Terminator::Return(None)],
    }]);
    program
}
