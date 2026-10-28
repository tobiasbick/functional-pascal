//! Instantiate each complete error key once, including finite Cause cycles.
//! Documentation: `docs/pascal/program-structure/compiled-programs.md`.

use super::invalid;
use crate::LinkError;
use fpas_code::ErrorInstanceKey;
use fpas_unit::{
    errors::{ErrorConstructorRequest, ErrorSchema},
    object::{ObjectEnumLayout, ObjectEnumVariant, ObjectErrorVariant, RelocatableObject},
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn expand<'a>(
    objects: impl Iterator<Item = &'a RelocatableObject>,
    schemas: &BTreeMap<String, super::SchemaSource<'_>>,
    requests: &BTreeSet<ErrorConstructorRequest>,
) -> Result<BTreeMap<ErrorInstanceKey, ObjectEnumLayout>, LinkError> {
    let objects = objects.collect::<Vec<_>>();
    let declarations = fpas_unit::equality::EqualityDeclarations::new(
        objects
            .iter()
            .flat_map(|object| object.equality_declarations.iter().cloned()),
    )
    .map_err(invalid)?;
    let mut layouts = BTreeMap::new();
    for object in objects {
        for layout in &object.enums {
            let Some(key) = &layout.error else {
                continue;
            };
            let mut layout = layout.clone();
            for variant in &mut layout.variants {
                variant.field_types.fill(0);
            }
            if let Some(previous) = layouts.insert(key.clone(), layout.clone())
                && previous != layout
            {
                return Err(invalid("conflicting copies of an error instance"));
            }
        }
    }
    let mut pending = layouts
        .keys()
        .cloned()
        .chain(requests.iter().map(|request| request.instance.clone()))
        .collect::<BTreeSet<_>>();
    let mut visited = BTreeSet::new();
    let mut budget = fpas_unit::errors::ErrorInstantiationBudget::default();
    while let Some(key) = pending.pop_first() {
        if !visited.insert(key.clone()) {
            continue;
        }
        if visited.len() > fpas_code::limits::MAX_ENUM_LAYOUTS {
            return Err(invalid("error instance expansion exceeds the layout limit"));
        }
        key.validate(0).map_err(invalid)?;
        if let Some(schema) = schemas.get(&key.declaration) {
            let instantiated = schema
                .schema
                .instantiate_with_declarations(&key, &mut budget, &declarations)
                .map_err(invalid)?;
            let declared = layout(key.clone(), &instantiated);
            if let Some(existing) = layouts.get(&key) {
                if existing != &declared {
                    return Err(invalid(
                        "concrete error layout disagrees with its declaration schema",
                    ));
                }
            } else {
                layouts.insert(key.clone(), declared);
            }
        } else if !layouts.contains_key(&key) {
            return Err(invalid(format!(
                "missing error schema `{}`",
                key.declaration
            )));
        }
        let current = layouts
            .get(&key)
            .ok_or_else(|| invalid("error instance disappeared during expansion"))?;
        for variant in &current.variants {
            if let Some(error) = &variant.error
                && let Some(cause) = error.cause
            {
                let family = error
                    .field_types
                    .get(usize::from(cause))
                    .and_then(fpas_code::TypeIdentity::error_key)
                    .ok_or_else(|| invalid("Cause does not name a concrete error instance"))?;
                pending.insert(family);
            }
        }
    }
    Ok(layouts)
}

fn layout(key: ErrorInstanceKey, schema: &ErrorSchema) -> ObjectEnumLayout {
    ObjectEnumLayout {
        error: Some(key),
        name: schema.display_name.clone(),
        variants: schema
            .variants
            .iter()
            .map(|variant| ObjectEnumVariant {
                error: Some(ObjectErrorVariant {
                    message: variant.message.clone(),
                    cause: variant.cause,
                    field_types: variant
                        .fields
                        .iter()
                        .map(|field| field.ty.clone())
                        .collect(),
                }),
                name: variant.name.clone(),
                fields: variant
                    .fields
                    .iter()
                    .map(|field| field.name.clone())
                    .collect(),
                field_types: vec![0; variant.fields.len()],
            })
            .collect(),
    }
}
