fn value_type(
    function: &Function,
    block: BlockId,
    instruction: impl Into<Option<usize>>,
    value: ValueId,
    all_values: &BTreeMap<ValueId, TypeId>,
    available: &BTreeSet<ValueId>,
) -> Result<TypeId, ValidationError> {
    let instruction = instruction.into();
    let Some(ty) = all_values.get(&value).copied() else {
        return Err(unknown(
            function,
            block,
            instruction,
            EntityKind::Value,
            value.get(),
        ));
    };
    if !available.contains(&value) {
        return Err(function_error(
            function.id,
            Some(block),
            instruction,
            ValidationErrorKind::UseBeforeDefinition { value },
        ));
    }
    Ok(ty)
}

fn require_result_type(
    function: &Function,
    block: BlockId,
    instruction: usize,
    result: Option<ValueDefinition>,
    expected: TypeId,
) -> Result<(), ValidationError> {
    let result = result.ok_or_else(|| {
        function_error(
            function.id,
            Some(block),
            Some(instruction),
            ValidationErrorKind::MissingResult,
        )
    })?;
    require_exact(function, block, instruction, "result", expected, result.ty)
}

fn require_result_category(
    program: &Program,
    function: &Function,
    block: BlockId,
    instruction: usize,
    result: Option<ValueDefinition>,
    category: TypeCategory,
) -> Result<(), ValidationError> {
    let result = result.ok_or_else(|| {
        function_error(
            function.id,
            Some(block),
            Some(instruction),
            ValidationErrorKind::MissingResult,
        )
    })?;
    require_category(
        program,
        function,
        block,
        Some(instruction),
        "result",
        result.ty,
        category,
    )
}

fn require_exact(
    function: &Function,
    block: BlockId,
    instruction: impl Into<Option<usize>>,
    operand: &'static str,
    expected: TypeId,
    actual: TypeId,
) -> Result<(), ValidationError> {
    if expected == actual {
        return Ok(());
    }
    Err(function_error(
        function.id,
        Some(block),
        instruction.into(),
        ValidationErrorKind::OperandType {
            operand,
            expected: expected.get(),
            actual: actual.get(),
        },
    ))
}

fn require_category(
    program: &Program,
    function: &Function,
    block: BlockId,
    instruction: impl Into<Option<usize>>,
    operand: &'static str,
    ty: TypeId,
    category: TypeCategory,
) -> Result<(), ValidationError> {
    let valid = matches!((program.ty(ty).map(|definition| &definition.kind), category),
        (Some(IrType::Unit), TypeCategory::Unit)
        | (Some(IrType::Boolean), TypeCategory::Boolean)
        | (Some(IrType::Integer), TypeCategory::Integer)
        | (Some(IrType::Real), TypeCategory::Real)
        | (Some(IrType::String), TypeCategory::String)
        | (Some(IrType::Dynamic), TypeCategory::Dynamic)
        |
        (
            Some(
                IrType::Boolean | IrType::Integer | IrType::Real | IrType::String | IrType::Dynamic,
            ),
            TypeCategory::Comparable,
        )
    );
    if valid {
        return Ok(());
    }
    Err(function_error(
        function.id,
        Some(block),
        instruction.into(),
        ValidationErrorKind::TypeCategory {
            operand,
            expected: category_name(category),
            actual: ty.get(),
        },
    ))
}

fn require_record_layout(
    program: &Program,
    function: &Function,
    block: BlockId,
    instruction: usize,
    ty: TypeId,
    layout: RecordLayoutId,
) -> Result<(), ValidationError> {
    match program.ty(ty).map(|definition| &definition.kind) {
        Some(IrType::Record(actual)) if *actual == layout => Ok(()),
        _ => Err(function_error(
            function.id,
            Some(block),
            Some(instruction),
            ValidationErrorKind::LayoutReference {
                expected: layout.get(),
                actual: ty.get(),
            },
        )),
    }
}

fn require_enum_layout(
    program: &Program,
    function: &Function,
    block: BlockId,
    instruction: usize,
    ty: TypeId,
    layout: EnumLayoutId,
) -> Result<(), ValidationError> {
    match program.ty(ty).map(|definition| &definition.kind) {
        Some(IrType::Enum(actual)) if *actual == layout => Ok(()),
        _ => Err(function_error(
            function.id,
            Some(block),
            Some(instruction),
            ValidationErrorKind::LayoutReference {
                expected: layout.get(),
                actual: ty.get(),
            },
        )),
    }
}

fn record_field<'a>(
    program: &'a Program,
    function: &Function,
    block: BlockId,
    instruction: usize,
    layout: RecordLayoutId,
    field: crate::FieldId,
) -> Result<&'a crate::RecordField, ValidationError> {
    let layout = program.record_layout(layout).ok_or_else(|| {
        unknown(
            function,
            block,
            instruction,
            EntityKind::RecordLayout,
            layout.get(),
        )
    })?;
    layout
        .fields
        .iter()
        .find(|item| item.id == field)
        .ok_or_else(|| unknown(function, block, instruction, EntityKind::Field, field.get()))
}

fn validate_ir_type(program: &Program, ty: &IrType) -> Result<(), ValidationError> {
    match ty {
        IrType::Function { parameters, result } => {
            validate_signature_types(program, parameters, *result)
        }
        IrType::Record(layout) if program.record_layout(*layout).is_none() => {
            Err(program_error(ValidationErrorKind::UnknownId {
                entity: EntityKind::RecordLayout,
                id: layout.get(),
            }))
        }
        IrType::Enum(layout) if program.enum_layout(*layout).is_none() => {
            Err(program_error(ValidationErrorKind::UnknownId {
                entity: EntityKind::EnumLayout,
                id: layout.get(),
            }))
        }
        IrType::Cell(inner) | IrType::Task(inner) => require_type(program, *inner),
        _ => Ok(()),
    }
}

fn validate_signature_types(
    program: &Program,
    parameters: &[TypeId],
    result: TypeId,
) -> Result<(), ValidationError> {
    for parameter in parameters {
        require_type(program, *parameter)?;
    }
    require_type(program, result)
}

fn require_type(program: &Program, ty: TypeId) -> Result<(), ValidationError> {
    if program.ty(ty).is_some() {
        return Ok(());
    }
    Err(program_error(ValidationErrorKind::UnknownId {
        entity: EntityKind::Type,
        id: ty.get(),
    }))
}

fn require_type_at(
    program: &Program,
    function: FunctionId,
    block: Option<BlockId>,
    instruction: Option<usize>,
    ty: TypeId,
) -> Result<(), ValidationError> {
    if program.ty(ty).is_some() {
        return Ok(());
    }
    Err(function_error(
        function,
        block,
        instruction,
        ValidationErrorKind::UnknownId {
            entity: EntityKind::Type,
            id: ty.get(),
        },
    ))
}

fn validate_unique(
    ids: impl Iterator<Item = u32>,
    entity: EntityKind,
) -> Result<(), ValidationError> {
    let mut seen = BTreeSet::new();
    for id in ids {
        if !seen.insert(id) {
            return Err(program_error(ValidationErrorKind::DuplicateId {
                entity,
                id,
            }));
        }
    }
    Ok(())
}

fn unknown(
    function: &Function,
    block: BlockId,
    instruction: impl Into<Option<usize>>,
    entity: EntityKind,
    id: u32,
) -> ValidationError {
    function_error(
        function.id,
        Some(block),
        instruction.into(),
        ValidationErrorKind::UnknownId { entity, id },
    )
}

fn category_name(category: TypeCategory) -> &'static str {
    match category {
        TypeCategory::Same => "same type",
        TypeCategory::Unit => "Unit",
        TypeCategory::Boolean => "Boolean",
        TypeCategory::Integer => "Integer",
        TypeCategory::Real => "Real",
        TypeCategory::String => "String",
        TypeCategory::Dynamic => "Dynamic",
        TypeCategory::Comparable => "comparable scalar",
    }
}
