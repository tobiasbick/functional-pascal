#[test]
fn source_debug_projection_rejects_a_contradictory_operational_scalar() {
    let mut program = all_operations_program();
    program.globals[0].projection = Some(fpas_ir::DebugProjection {
        ty: BOOLEAN,
        type_name: None,
        kind: fpas_ir::DebugProjectionKind::Scalar,
    });
    assert!(matches!(program.validate(), Err(fpas_ir::validate::ValidationError {
        kind: fpas_ir::validate::ValidationErrorKind::DebugProjection { .. }, ..
    })));
}

#[test]
fn source_debug_projection_rejects_excessive_depth_before_recursive_encoding() {
    let mut program = all_operations_program();
    let mut projection = fpas_ir::DebugProjection {
        ty: INTEGER,
        type_name: None,
        kind: fpas_ir::DebugProjectionKind::Scalar,
    };
    let mut ty = INTEGER;
    for _ in 0..34 {
        let parent = TypeId::new(program.types.len() as u32);
        program.types.push(TypeDefinition { id: parent, kind: IrType::Array(ty) });
        projection = fpas_ir::DebugProjection { ty: parent, type_name: None,
            kind: fpas_ir::DebugProjectionKind::Array(Box::new(projection)) };
        ty = parent;
    }
    program.globals.push(Global { id: GlobalId::new(2), name: "deep".into(), ty,
        type_name: None, projection: Some(projection), mutable: true, initializer: None });
    assert!(matches!(program.validate(), Err(fpas_ir::validate::ValidationError {
        kind: fpas_ir::validate::ValidationErrorKind::DebugProjection { .. }, ..
    })));
}

#[test]
fn source_debug_projection_keeps_known_scalars_and_bounded_nominal_names() {
    let mut program = all_operations_program();
    program.globals[0].projection = Some(fpas_ir::DebugProjection {
        ty: INTEGER,
        type_name: Some("Dollars".into()),
        kind: fpas_ir::DebugProjectionKind::Scalar,
    });
    assert!(program.validate().is_ok(), "a compatible nominal scalar projection");
    program.globals[0].projection = Some(fpas_ir::DebugProjection {
        ty: INTEGER,
        type_name: Some("x".repeat(4097)),
        kind: fpas_ir::DebugProjectionKind::Scalar,
    });
    assert!(matches!(program.validate(), Err(fpas_ir::validate::ValidationError {
        kind: fpas_ir::validate::ValidationErrorKind::DebugProjection { .. }, ..
    })));
}
