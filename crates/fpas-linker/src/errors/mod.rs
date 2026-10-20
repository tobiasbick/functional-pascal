//! Bounded source-less instantiation and ordinary constructor function synthesis.
//! Documentation: `docs/pascal/program-structure/compiled-programs.md`.

mod factories;
mod instances;

use crate::LinkError;

pub(super) struct SchemaSource<'a> {
    pub schema: &'a fpas_unit::errors::ErrorSchema,
    pub source: &'a str,
    pub line: u32,
    pub column: u32,
}
use fpas_unit::object::{ObjectConstant, RelocatableObject};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn prepare(
    units: &[RelocatableObject],
    program: &RelocatableObject,
) -> Result<Option<(Vec<RelocatableObject>, RelocatableObject)>, LinkError> {
    let objects = units.iter().chain(std::iter::once(program));
    let mut requests = BTreeSet::new();
    for object in objects.clone() {
        for constant in &object.constants {
            if let ObjectConstant::ErrorConstructor(request) = constant {
                requests.insert(request.clone());
            }
        }
    }
    if requests.is_empty()
        && objects.clone().all(|object| {
            object.error_schemas.is_empty()
                && object.enums.iter().all(|layout| layout.error.is_none())
        })
    {
        return Ok(None);
    }
    for object in objects.clone() {
        object
            .validate()
            .map_err(|error| LinkError::InvalidObject {
                owner: object.owner.clone(),
                detail: error.to_string(),
            })?;
    }
    let mut schemas = BTreeMap::new();
    for object in objects.clone() {
        for retained in &object.error_schemas {
            let schema = &retained.schema;
            let provenance = SchemaSource {
                schema,
                source: &object.sources[retained.location.source as usize],
                line: retained.location.line,
                column: retained.location.column,
            };
            if let Some(previous) = schemas.insert(schema.declaration.clone(), provenance)
                && previous.schema != schema
            {
                return Err(invalid("conflicting copies of an error declaration"));
            }
        }
    }
    let layouts = instances::expand(objects.clone(), &schemas, &requests)?;
    let mut generated = factories::object(&layouts, &requests, &schemas)?;
    generated
        .define_all_private()
        .map_err(|error| invalid(error.to_string()))?;
    generated
        .validate()
        .map_err(|error| invalid(error.to_string()))?;
    if objects
        .clone()
        .any(|object| object.owner == generated.owner)
    {
        return Err(invalid(
            "input object uses the reserved error-constructor owner",
        ));
    }
    let mut units = units.to_vec();
    units.push(generated);
    Ok(Some((units, program.clone())))
}

pub(super) fn invalid(message: impl Into<String>) -> LinkError {
    LinkError::ErrorMetadata(message.into())
}
