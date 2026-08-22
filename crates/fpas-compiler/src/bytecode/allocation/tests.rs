use fpas_ir::{
    BasicBlock, BlockId, Constant, Function, FunctionId, FunctionSignature, GlobalId, Instruction,
    Local, LocalId, Operation, Terminator, TypeId, ValueDefinition, ValueId,
};

use super::{Allocation, coalesced_local_writes, largest_window};

#[test]
fn one_argument_calls_need_no_copy_window() {
    let one_argument = function_with(vec![Instruction {
        source: None,
        result: None,
        operation: Operation::CallDirect {
            function: FunctionId::new(1),
            arguments: vec![ValueId::new(0)],
        },
    }]);
    assert_eq!(largest_window(&one_argument), 0);

    let two_arguments = function_with(vec![Instruction {
        source: None,
        result: None,
        operation: Operation::CallDirect {
            function: FunctionId::new(1),
            arguments: vec![ValueId::new(0), ValueId::new(1)],
        },
    }]);
    assert_eq!(largest_window(&two_arguments), 2);
}

#[test]
fn local_write_coalescing_requires_one_use_in_the_same_block() {
    let value = ValueDefinition {
        id: ValueId::new(0),
        ty: TypeId::new(0),
    };
    let mut function = function_with(vec![
        Instruction {
            source: None,
            result: Some(value),
            operation: Operation::Const(Constant::Integer(1)),
        },
        Instruction {
            source: None,
            result: None,
            operation: Operation::WriteLocal {
                value: value.id,
                local: LocalId::new(0),
            },
        },
    ]);
    assert_eq!(
        coalesced_local_writes(&function).get(&value.id),
        Some(&LocalId::new(0))
    );

    function.blocks[0].instructions.push(Instruction {
        source: None,
        result: None,
        operation: Operation::StoreGlobal {
            global: GlobalId::new(0),
            value: value.id,
        },
    });
    assert!(!coalesced_local_writes(&function).contains_key(&value.id));
}

#[test]
fn non_overlapping_temporaries_reuse_registers() {
    let first = ValueDefinition {
        id: ValueId::new(0),
        ty: TypeId::new(0),
    };
    let second = ValueDefinition {
        id: ValueId::new(1),
        ty: TypeId::new(0),
    };
    let function = Function {
        id: FunctionId::new(0),
        name: "reuse".to_string(),
        signature: FunctionSignature {
            parameters: Vec::new(),
            result: TypeId::new(1),
        },
        parameters: Vec::new(),
        locals: Vec::new(),
        captures: Vec::new(),
        debug: fpas_ir::FunctionDebugInfo::default(),
        blocks: vec![BasicBlock {
            id: BlockId::new(0),
            parameters: Vec::new(),
            instructions: vec![
                Instruction {
                    source: None,
                    result: Some(first),
                    operation: Operation::Const(Constant::Integer(1)),
                },
                Instruction {
                    source: None,
                    result: None,
                    operation: Operation::StoreGlobal {
                        global: GlobalId::new(0),
                        value: first.id,
                    },
                },
                Instruction {
                    source: None,
                    result: Some(second),
                    operation: Operation::Const(Constant::Integer(2)),
                },
                Instruction {
                    source: None,
                    result: None,
                    operation: Operation::StoreGlobal {
                        global: GlobalId::new(0),
                        value: second.id,
                    },
                },
            ],
            terminators: vec![Terminator::Return(None)],
        }],
        entry: BlockId::new(0),
        max_call_arguments: 0,
        can_spawn_tasks: false,
    };
    let allocation = Allocation::build(&function).expect("allocate reused temporaries");
    let first_register = allocation
        .value(first.id)
        .expect("first temporary must have a register");
    let second_register = allocation
        .value(second.id)
        .expect("second temporary must have a register");
    assert_eq!(
        first_register, second_register,
        "an interior jump cannot assume a temporary still has its earlier type"
    );
}

fn function_with(instructions: Vec<Instruction>) -> Function {
    Function {
        id: FunctionId::new(0),
        name: "coalescing".to_string(),
        signature: FunctionSignature {
            parameters: Vec::new(),
            result: TypeId::new(1),
        },
        parameters: Vec::new(),
        locals: vec![Local {
            id: LocalId::new(0),
            ty: TypeId::new(0),
            mutable: true,
            capture: None,
        }],
        captures: Vec::new(),
        debug: fpas_ir::FunctionDebugInfo::default(),
        blocks: vec![BasicBlock {
            id: BlockId::new(0),
            parameters: Vec::new(),
            instructions,
            terminators: vec![Terminator::Return(None)],
        }],
        entry: BlockId::new(0),
        max_call_arguments: 0,
        can_spawn_tasks: false,
    }
}
