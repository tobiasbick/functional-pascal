#[test]
fn global_initializer_rejects_store_to_another_global() {
    let mut program = all_operations_program();
    program.globals[1].initializer = Some(fpas_ir::GlobalInitializer {
        function: FunctionId::new(0),
        location: fpas_ir::DebugInstructionLocation {
            block: BlockId::new(0),
            instruction: 10,
        },
    });

    assert!(matches!(
        program.validate(),
        Err(error) if matches!(error.kind, fpas_ir::validate::ValidationErrorKind::InvalidInitializerTarget {
            owner: fpas_ir::validate::EntityKind::Global,
            target: fpas_ir::validate::EntityKind::Global,
            expected: 1,
            actual: 0
        })
    ));
}

#[test]
fn global_initializer_rejects_unknown_function() {
    let mut program = all_operations_program();
    program.globals[0].initializer = Some(global_initializer(99, 0, 10));

    assert!(matches!(
        program.validate(),
        Err(error) if matches!(error.kind, fpas_ir::validate::ValidationErrorKind::UnknownId {
            entity: fpas_ir::validate::EntityKind::Function,
            id: 99
        })
    ));
}

#[test]
fn global_initializer_rejects_unknown_block() {
    let mut program = all_operations_program();
    program.globals[0].initializer = Some(global_initializer(0, 99, 10));

    assert!(matches!(
        program.validate(),
        Err(error) if matches!(error.kind, fpas_ir::validate::ValidationErrorKind::UnknownId {
            entity: fpas_ir::validate::EntityKind::Block,
            id: 99
        })
    ));
}

#[test]
fn global_initializer_rejects_unknown_instruction() {
    let mut program = all_operations_program();
    program.globals[0].initializer = Some(global_initializer(0, 0, 99));

    assert!(matches!(
        program.validate(),
        Err(error) if matches!(error.kind, fpas_ir::validate::ValidationErrorKind::UnknownId {
            entity: fpas_ir::validate::EntityKind::Instruction,
            id: 99
        })
    ));
}

#[test]
fn global_initializer_rejects_non_store_operation() {
    let mut program = all_operations_program();
    program.globals[0].initializer = Some(global_initializer(0, 0, 9));

    assert!(matches!(
        program.validate(),
        Err(error) if matches!(error.kind, fpas_ir::validate::ValidationErrorKind::InvalidInitializerOperation {
            owner: fpas_ir::validate::EntityKind::Global,
            expected: "StoreGlobal"
        })
    ));
}

#[test]
fn binding_initializer_rejects_store_to_another_local() {
    let mut program = all_operations_program();
    add_binding(&mut program, LocalId::new(1), ARRAY, 4);

    assert!(matches!(
        program.validate(),
        Err(error) if matches!(error.kind, fpas_ir::validate::ValidationErrorKind::InvalidInitializerTarget {
            owner: fpas_ir::validate::EntityKind::DebugBinding,
            target: fpas_ir::validate::EntityKind::Local,
            expected: 1,
            actual: 0
        })
    ));
}

#[test]
fn binding_initializer_rejects_non_store_operation() {
    let mut program = all_operations_program();
    add_binding(&mut program, LocalId::new(0), INTEGER, 3);

    assert!(matches!(
        program.validate(),
        Err(error) if matches!(error.kind, fpas_ir::validate::ValidationErrorKind::InvalidInitializerOperation {
            owner: fpas_ir::validate::EntityKind::DebugBinding,
            expected: "WriteLocal"
        })
    ));
}

#[test]
fn exact_global_and_binding_initializer_stores_are_valid() {
    let mut program = all_operations_program();
    program.globals[0].initializer = Some(global_initializer(0, 0, 10));
    add_binding(&mut program, LocalId::new(0), INTEGER, 4);

    let result = program.validate();

    assert!(result.is_ok(), "exact initializer stores must remain valid: {result:?}");
}

fn global_initializer(function: u32, block: u32, instruction: usize) -> fpas_ir::GlobalInitializer {
    fpas_ir::GlobalInitializer {
        function: FunctionId::new(function),
        location: fpas_ir::DebugInstructionLocation {
            block: BlockId::new(block),
            instruction,
        },
    }
}

fn add_binding(program: &mut Program, local: LocalId, ty: TypeId, instruction: usize) {
    program.functions[0].debug.scopes = vec![fpas_ir::DebugScope {
        id: 0,
        parent: None,
    }];
    program.functions[0].debug.bindings = vec![fpas_ir::DebugBinding {
        local,
        name: "value".to_string(),
        kind: fpas_ir::DebugBindingKind::Local,
        ty,
        mutable: true,
        scope: 0,
        declaration: None,
        hidden: false,
        cell_backed: false,
        initializer: Some(fpas_ir::DebugInstructionLocation {
            block: BlockId::new(0),
            instruction,
        }),
    }];
}
