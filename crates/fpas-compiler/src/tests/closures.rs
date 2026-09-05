mod repeat;

use fpas_ir::{
    BasicBlock, BinaryOperation, BlockId, BlockTarget, CaptureDeclaration, CaptureKind, Constant,
    DebugBinding, DebugBindingId, DebugBindingKind, DebugCaptureSource, DebugScope, Function,
    FunctionId, FunctionSignature, Instruction, IrType, Local, LocalId, Operation, Program,
    Terminator, TypeDefinition, TypeId, ValueDefinition, ValueId,
};

const UNIT: TypeId = TypeId::new(0);
const BOOLEAN: TypeId = TypeId::new(1);
const INTEGER: TypeId = TypeId::new(2);
const STRING: TypeId = TypeId::new(3);
const CELL: TypeId = TypeId::new(4);
const FUNCTION: TypeId = TypeId::new(5);

use super::assert_succeeds;

#[test]
fn program_level_closure_initializer_is_discovered_before_lowering() {
    assert_succeeds(
        r#"
program ClosureInit;
var F: procedure() := procedure()
begin
end;
begin
  F()
end.
"#,
    );
}

#[test]
fn immutable_anonymous_capture_executes() {
    assert_succeeds(
        r#"
program ImmutableClosure;
function MakeAdder(Base: integer): function(Value: integer): integer;
begin
  return function(Value: integer): integer
  begin
    return Base + Value;
  end;
end;
begin
  var AddForty: function(Value: integer): integer := MakeAdder(40);
  if AddForty(2) <> 42 then
    panic('immutable closure mismatch');
end.
"#,
    );
}

#[test]
fn mutable_anonymous_capture_shares_cell_with_repeated_calls() {
    assert_succeeds(
        r#"
program MutableClosure;
function MakeCounter(): function(): integer;
begin
  mutable var Count: integer := 40;
  return function(): integer
  begin
    Count := Count + 1;
    return Count;
  end;
end;
begin
  var Next: function(): integer := MakeCounter();
  Next();
  if Next() <> 42 then
    panic('mutable closure mismatch');
end.
"#,
    );
}

#[test]
fn named_nested_routine_escapes_with_numeric_capture_target() {
    assert_succeeds(
        r#"
program NamedNestedClosure;
function MakeAdder(Base: integer): function(Value: integer): integer;
  function Add(Value: integer): integer;
  begin
    return Base + Value;
  end;
begin
  return Add;
end;
begin
  var AddForty: function(Value: integer): integer := MakeAdder(40);
  if AddForty(2) <> 42 then
    panic('named nested closure mismatch');
end.
"#,
    );
}

#[test]
fn closure_cells_compile_and_execute_through_register_bytecode() {
    let program = Program {
        types: vec![
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
                id: CELL,
                kind: IrType::Cell(INTEGER),
            },
            TypeDefinition {
                id: FUNCTION,
                kind: IrType::Function {
                    parameters: Vec::new(),
                    result: INTEGER,
                },
            },
        ],
        globals: Vec::new(),
        record_layouts: Vec::new(),
        enum_layouts: Vec::new(),
        intrinsics: Vec::new(),
        functions: vec![root_function(), increment_function()],
        entry: FunctionId::new(0),
    };
    let executable = crate::bytecode::compile_program(&program).expect("closure IR should compile");
    fpas_vm::Vm::new(executable)
        .run()
        .expect("closure register program should succeed");
}

fn root_function() -> Function {
    Function {
        id: FunctionId::new(0),
        name: "closure-root".to_string(),
        signature: FunctionSignature {
            parameters: Vec::new(),
            result: UNIT,
        },
        parameters: Vec::new(),
        locals: vec![Local {
            id: LocalId::new(0),
            ty: CELL,
            mutable: true,
            capture: None,
        }],
        captures: Vec::new(),
        debug: fpas_ir::FunctionDebugInfo {
            scopes: vec![DebugScope {
                id: 0,
                parent: None,
            }],
            bindings: vec![DebugBinding {
                local: LocalId::new(0),
                name: "cell".to_string(),
                kind: DebugBindingKind::Local,
                ty: INTEGER,
                mutable: true,
                scope: 0,
                declaration: None,
                hidden: false,
                cell_backed: true,
                initializer: None,
            }],
            ..fpas_ir::FunctionDebugInfo::default()
        },
        blocks: vec![
            BasicBlock {
                id: BlockId::new(0),
                parameters: Vec::new(),
                instructions: vec![
                    value(0, INTEGER, Operation::Const(Constant::Integer(40))),
                    value(1, CELL, Operation::MakeCell(ValueId::new(0))),
                    value(
                        2,
                        FUNCTION,
                        Operation::MakeClosure {
                            function: FunctionId::new(1),
                            captures: vec![ValueId::new(1)],
                        },
                    ),
                    value(
                        3,
                        INTEGER,
                        Operation::CallValue {
                            callee: ValueId::new(2),
                            arguments: Vec::new(),
                        },
                    ),
                    value(
                        4,
                        INTEGER,
                        Operation::CallValue {
                            callee: ValueId::new(2),
                            arguments: Vec::new(),
                        },
                    ),
                    value(5, INTEGER, Operation::Const(Constant::Integer(42))),
                    value(
                        6,
                        BOOLEAN,
                        Operation::Binary {
                            operation: BinaryOperation::Equal,
                            left: ValueId::new(4),
                            right: ValueId::new(5),
                        },
                    ),
                ],
                terminators: vec![Terminator::Branch {
                    condition: ValueId::new(6),
                    then_target: target(1),
                    else_target: target(2),
                }],
            },
            BasicBlock {
                id: BlockId::new(1),
                parameters: Vec::new(),
                instructions: Vec::new(),
                terminators: vec![Terminator::Return(None)],
            },
            BasicBlock {
                id: BlockId::new(2),
                parameters: Vec::new(),
                instructions: vec![value(
                    7,
                    STRING,
                    Operation::Const(Constant::String("closure cell mismatch".to_string())),
                )],
                terminators: vec![Terminator::Panic(ValueId::new(7))],
            },
        ],
        entry: BlockId::new(0),
        max_call_arguments: 0,
        can_spawn_tasks: false,
    }
}

fn increment_function() -> Function {
    Function {
        id: FunctionId::new(1),
        name: "increment".to_string(),
        signature: FunctionSignature {
            parameters: Vec::new(),
            result: INTEGER,
        },
        parameters: Vec::new(),
        locals: vec![Local {
            id: LocalId::new(0),
            ty: CELL,
            mutable: true,
            capture: Some(CaptureKind::Cell),
        }],
        captures: vec![CaptureDeclaration {
            ty: INTEGER,
            kind: CaptureKind::Cell,
        }],
        debug: fpas_ir::FunctionDebugInfo {
            scopes: vec![DebugScope {
                id: 0,
                parent: None,
            }],
            bindings: vec![DebugBinding {
                local: LocalId::new(0),
                name: "cell".to_string(),
                kind: DebugBindingKind::Capture,
                ty: INTEGER,
                mutable: true,
                scope: 0,
                declaration: None,
                hidden: false,
                cell_backed: true,
                initializer: None,
            }],
            lexical_owner: Some(FunctionId::new(0)),
            capture_sources: vec![DebugCaptureSource {
                binding: DebugBindingId::new(0),
                ty: INTEGER,
                kind: CaptureKind::Cell,
            }],
            ..fpas_ir::FunctionDebugInfo::default()
        },
        blocks: vec![BasicBlock {
            id: BlockId::new(0),
            parameters: Vec::new(),
            instructions: vec![
                value(0, CELL, Operation::ReadLocal(LocalId::new(0))),
                value(1, INTEGER, Operation::CellRead(ValueId::new(0))),
                value(2, INTEGER, Operation::Const(Constant::Integer(1))),
                value(
                    3,
                    INTEGER,
                    Operation::Binary {
                        operation: BinaryOperation::AddInteger,
                        left: ValueId::new(1),
                        right: ValueId::new(2),
                    },
                ),
                Instruction {
                    source: None,
                    result: None,
                    operation: Operation::CellWrite {
                        cell: ValueId::new(0),
                        value: ValueId::new(3),
                    },
                },
            ],
            terminators: vec![Terminator::Return(Some(ValueId::new(3)))],
        }],
        entry: BlockId::new(0),
        max_call_arguments: 0,
        can_spawn_tasks: false,
    }
}

fn value(id: u32, ty: TypeId, operation: Operation) -> Instruction {
    Instruction {
        source: None,
        result: Some(ValueDefinition {
            id: ValueId::new(id),
            ty,
        }),
        operation,
    }
}

fn target(id: u32) -> BlockTarget {
    BlockTarget {
        block: BlockId::new(id),
        arguments: Vec::new(),
    }
}
