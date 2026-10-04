// References are instruction operands and synchronous parameters, never retained data.

fn reject_reference_storage(
    program: &Program,
    ty: TypeId,
    context: &'static str,
) -> Result<(), ValidationError> {
    if contains_reference_type(program, ty) {
        Err(program_error(ValidationErrorKind::ReferenceEscape {
            context,
            ty: ty.get(),
        }))
    } else {
        Ok(())
    }
}

fn contains_reference_type(program: &Program, root: TypeId) -> bool {
    let mut pending = vec![root];
    let mut visited = BTreeSet::new();
    while let Some(ty) = pending.pop() {
        if !visited.insert(ty) {
            continue;
        }
        match program.ty(ty).map(|definition| &definition.kind) {
            Some(IrType::Reference(_)) => return true,
            Some(
                IrType::Array(inner)
                | IrType::Cell(inner)
                | IrType::Option(inner)
                | IrType::Task(inner)
                | IrType::Channel(inner),
            ) => pending.push(*inner),
            Some(
                IrType::Dictionary { key, value }
                | IrType::Result {
                    ok: key,
                    error: value,
                },
            ) => pending.extend([*key, *value]),
            Some(IrType::Record(layout)) => {
                if let Some(layout) = program.record_layout(*layout) {
                    pending.extend(layout.fields.iter().map(|field| field.ty));
                }
            }
            Some(IrType::Enum(layout)) => {
                if let Some(layout) = program.enum_layout(*layout) {
                    pending.extend(
                        layout
                            .variants
                            .iter()
                            .flat_map(|variant| variant.fields.iter().copied()),
                    );
                }
            }
            // Callable values may describe var parameters without retaining their authority.
            _ => {}
        }
    }
    false
}

fn reject_spawn_references(
    scope: OperandScope<'_>,
    callee: ValueId,
) -> Result<(), ValidationError> {
    let ty = value_type(
        scope.function,
        scope.block,
        scope.instruction,
        callee,
        scope.all_values,
        scope.available,
    )?;
    if let Some(IrType::Function { parameters, .. }) =
        scope.program.ty(ty).map(|definition| &definition.kind)
    {
        for parameter in parameters {
            reject_reference_storage(scope.program, *parameter, "task parameter")?;
        }
    }
    Ok(())
}
