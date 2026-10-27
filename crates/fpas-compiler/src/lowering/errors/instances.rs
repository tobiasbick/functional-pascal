//! Concrete error layouts and readonly factories shared by erased generic callers.
//! Documentation: `docs/pascal/language/types/errors.md`.

use super::*;
use fpas_ir::{
    BasicBlock, EnumLayout, EnumLayoutId, EnumVariant, FunctionSignature, IrType, Terminator,
    ValueDefinition, VariantId,
};
use fpas_unit::errors::ErrorInstanceKey;

pub(super) fn ensure_instance(
    types: &mut TypeTable,
    key: &ErrorInstanceKey,
    span: fpas_lexer::Span,
) -> Result<EnumLayoutId, CompileError> {
    if let Some(id) = types.enum_layouts.iter().find_map(|layout| {
        (types.errors.concrete(layout.id).as_ref() == Some(key)).then_some(layout.id)
    }) {
        return Ok(id);
    }
    let schema = types
        .errors
        .schemas
        .get(&key.declaration)
        .ok_or_else(|| error(format!("error schema {} is unavailable", key.declaration)))?
        .instantiate_with_declarations(
            key,
            &mut fpas_unit::errors::ErrorInstantiationBudget::default(),
            &fpas_unit::equality::EqualityDeclarations::new(
                types.errors.equality_declarations().map_err(error)?,
            )
            .map_err(error)?,
        )
        .map_err(error)?;
    let id =
        EnumLayoutId::try_from_index(types.enum_layouts.len()).map_err(|e| error(e.to_string()))?;
    let variants = schema
        .variants
        .iter()
        .enumerate()
        .map(|(index, variant)| {
            Ok(EnumVariant {
                id: VariantId::try_from_index(index).map_err(|e| error(e.to_string()))?,
                name: variant.name.clone(),
                field_names: variant
                    .fields
                    .iter()
                    .map(|field| field.name.clone())
                    .collect(),
                fields: vec![types::DYNAMIC; variant.fields.len()],
            })
        })
        .collect::<Result<_, CompileError>>()?;
    types.enum_layouts.push(EnumLayout {
        id,
        name: schema.display_name.clone(),
        error_template: false,
        variants,
    });
    types
        .errors
        .instances
        .insert(id, (key.clone(), schema.clone()));
    types.intern_kind(IrType::Enum(id), span)?;
    for variant in &schema.variants {
        if let Some(cause) = variant.cause
            && let Some(key) = variant.fields[usize::from(cause)].ty.error_key()
        {
            ensure_instance(types, &key, span)?;
        }
    }
    Ok(id)
}

pub(super) fn factory_type(
    types: &mut TypeTable,
    request: &Request,
    span: fpas_lexer::Span,
) -> Result<TypeId, CompileError> {
    let schema = types
        .errors
        .schemas
        .get(&request.instance.declaration)
        .ok_or_else(|| {
            error(format!(
                "error schema {} is unavailable",
                request.instance.declaration
            ))
        })?;
    let variant = schema
        .variants
        .get(usize::from(request.variant))
        .ok_or_else(|| error("unknown error constructor variant"))?;
    types.function_type(
        vec![types::DYNAMIC; variant.fields.len()],
        types::DYNAMIC,
        span,
    )
}

pub(super) fn factory(
    functions: &mut Vec<Function>,
    types: &mut TypeTable,
    request: &Request,
    span: fpas_lexer::Span,
) -> Result<(FunctionId, TypeId), CompileError> {
    let ty = factory_type(types, request, span)?;
    if let Some((&id, _)) = types
        .errors
        .factories
        .iter()
        .find(|(_, existing)| *existing == request)
    {
        return Ok((id, ty));
    }
    let layout = ensure_instance(types, &request.instance, span)?;
    let stored = &types.enum_layouts[layout.get() as usize];
    let variant = stored
        .variants
        .get(usize::from(request.variant))
        .ok_or_else(|| error("unknown error variant"))?;
    let arity = variant.fields.len();
    let name = format!("{}.{}", stored.name, variant.name);
    let id = FunctionId::new(
        functions
            .iter()
            .map(|function| function.id.get())
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or_else(|| error("error factory function overflow"))?,
    );
    let result_type = types.intern_kind(IrType::Enum(layout), span)?;
    let parameters = (0..arity)
        .map(|index| {
            Ok(ValueDefinition {
                id: ValueId::try_from_index(index).map_err(|e| error(e.to_string()))?,
                ty: types::DYNAMIC,
            })
        })
        .collect::<Result<Vec<_>, CompileError>>()?;
    let result = ValueDefinition {
        id: ValueId::try_from_index(arity).map_err(|e| error(e.to_string()))?,
        ty: result_type,
    };
    let location = types
        .errors
        .locations
        .get(&request.instance.declaration)
        .copied()
        .unwrap_or(span);
    functions.push(Function {
        id,
        name: name.clone(),
        signature: FunctionSignature {
            parameters: vec![types::DYNAMIC; arity],
            result: types::DYNAMIC,
        },
        parameters: parameters.clone(),
        locals: vec![],
        captures: vec![],
        debug: Default::default(),
        blocks: vec![BasicBlock {
            id: BlockId::new(0),
            parameters: vec![],
            instructions: vec![Instruction {
                result: Some(result),
                source: Some(location.diagnostic_span_or_synthetic()),
                operation: Operation::MakeEnum {
                    layout,
                    variant: VariantId::new(u32::from(request.variant)),
                    fields: parameters.iter().map(|parameter| parameter.id).collect(),
                },
            }],
            terminators: vec![Terminator::Return(Some(result.id))],
        }],
        entry: BlockId::new(0),
        max_call_arguments: u32::try_from(arity)
            .map_err(|_| error("error factory arity overflow"))?,
        can_spawn_tasks: false,
    });
    types.errors.factories.insert(id, request.clone());
    types.errors.displays.insert(id, name);
    Ok((id, ty))
}
