//! Definition, import, and callable-implementation validation.

use std::collections::HashMap;

use fpas_unit::object::{DefinitionKind, ObjectDefinition, RelocatableObject};

use crate::LinkError;

struct DefinitionEntry<'a> {
    definition: &'a ObjectDefinition,
    object: &'a RelocatableObject,
}

pub(super) fn validate_definitions_and_imports(
    units: &[RelocatableObject],
    program: &RelocatableObject,
) -> Result<(), LinkError> {
    let objects = units.iter().chain(std::iter::once(program));
    let mut definition_indices = HashMap::<String, usize>::new();
    let mut definitions = Vec::<DefinitionEntry<'_>>::new();
    for object in objects.clone() {
        for definition in &object.definitions {
            let key = definition.name.to_ascii_lowercase();
            if definition_indices.insert(key, definitions.len()).is_some() {
                return Err(LinkError::DuplicateDefinition(definition.name.clone()));
            }
            definitions.push(DefinitionEntry { definition, object });
        }
    }

    for entry in &definitions {
        validate_callable_implementation(entry)?;
    }
    for object in objects {
        validate_imports(object, &definition_indices, &definitions)?;
    }
    Ok(())
}

fn validate_callable_implementation(entry: &DefinitionEntry<'_>) -> Result<(), LinkError> {
    if entry.definition.kind != DefinitionKind::Callable
        || entry
            .object
            .functions
            .keys()
            .any(|name| name.eq_ignore_ascii_case(&entry.definition.name))
    {
        return Ok(());
    }
    Err(LinkError::MissingFunctionImplementation {
        owner: entry.object.owner.clone(),
        name: entry.definition.name.clone(),
    })
}

fn validate_imports(
    object: &RelocatableObject,
    definition_indices: &HashMap<String, usize>,
    definitions: &[DefinitionEntry<'_>],
) -> Result<(), LinkError> {
    for import in &object.imports {
        let key = import.name.to_ascii_lowercase();
        let Some(&index) = definition_indices.get(&key) else {
            return Err(unresolved_import(object, import.name.clone(), import.kind));
        };
        let entry = &definitions[index];
        if !entry.definition.public || entry.definition.kind != import.kind {
            return Err(unresolved_import(object, import.name.clone(), import.kind));
        }
    }
    Ok(())
}

fn unresolved_import(object: &RelocatableObject, name: String, kind: DefinitionKind) -> LinkError {
    LinkError::UnresolvedImport {
        owner: object.owner.clone(),
        name,
        kind,
    }
}
