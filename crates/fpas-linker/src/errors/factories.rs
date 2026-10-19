//! One noncapturing MakeEnum function per used concrete constructor.
//! Documentation: `docs/pascal/program-structure/compiled-programs.md`.

use super::invalid;
use crate::LinkError;
use fpas_bytecode::{ErrorInstanceKey, Instruction, NO_REGISTER, Opcode};
use fpas_unit::{
    errors::{ErrorConstructorRequest, constructor_symbol},
    object::{
        OBJECT_VERSION, ObjectDebugType, ObjectEnumLayout, ObjectFunction, ObjectReturn,
        ObjectSourceRun, RelocatableObject, Relocation, RelocationKind, SymbolReference,
    },
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn object(
    layouts: &BTreeMap<ErrorInstanceKey, ObjectEnumLayout>,
    requests: &BTreeSet<ErrorConstructorRequest>,
    schemas: &BTreeMap<String, super::SchemaSource<'_>>,
) -> Result<RelocatableObject, LinkError> {
    let mut result = RelocatableObject {
        error_schemas: vec![],
        version: OBJECT_VERSION,
        owner: "$error_constructors".to_owned(),
        entry: None,
        initializer: None,
        functions: vec![],
        constants: vec![],
        globals: vec![],
        records: vec![],
        enums: layouts.values().cloned().collect(),
        debug_types: vec![ObjectDebugType::Dynamic],
        distinct_types: vec![],
        sources: vec![],
        definitions: vec![],
        imports: vec![],
        relocations: vec![],
    };
    let indices = layouts
        .keys()
        .enumerate()
        .map(|(index, key)| (key, index))
        .collect::<BTreeMap<_, _>>();
    let mut symbols = BTreeMap::new();
    for request in requests {
        let provenance = schemas.get(&request.instance.declaration).ok_or_else(|| {
            invalid(
                "error constructor requires its retained declaration schema and source provenance",
            )
        })?;
        let source = match result
            .sources
            .iter()
            .position(|source| source == provenance.source)
        {
            Some(index) => index,
            None => {
                result.sources.push(provenance.source.to_owned());
                result.sources.len() - 1
            }
        };
        let index = *indices
            .get(&request.instance)
            .ok_or_else(|| invalid("missing concrete constructor layout"))?;
        let layout = &result.enums[index];
        let variant = layout
            .variants
            .get(usize::from(request.variant))
            .ok_or_else(|| invalid("error constructor variant is out of range"))?;
        let arity = u8::try_from(variant.fields.len()).map_err(|_| {
            invalid("error constructor exceeds the ordinary callable argument limit")
        })?;
        let name = constructor_symbol(request).map_err(invalid)?;
        if let Some(previous) = symbols.insert(name.clone(), request)
            && previous != request
        {
            return Err(invalid("error constructor symbol hash collision"));
        }
        let destination = u16::from(arity);
        let construct = Instruction::abc(
            Opcode::MakeEnum,
            destination,
            0,
            if arity == 0 { NO_REGISTER } else { 0 },
            0,
        )
        .map_err(|error| invalid(error.to_string()))?;
        let returns = Instruction::abc(Opcode::Return, destination, 0, 0, 0)
            .map_err(|error| invalid(error.to_string()))?;
        result.relocations.push(Relocation {
            function: u32::try_from(result.functions.len())
                .map_err(|_| invalid("too many error factories"))?,
            instruction: 0,
            kind: RelocationKind::EnumVariant {
                enumeration: SymbolReference::Local(
                    u32::try_from(index).map_err(|_| invalid("too many error instances"))?,
                ),
                variant: variant.name.to_ascii_lowercase(),
            },
        });
        result.functions.push(ObjectFunction {
            error_constructors: None,
            name,
            code: vec![construct.word(), returns.word()],
            arity,
            capture_count: 0,
            register_count: destination + 1,
            returns: ObjectReturn::Value,
            uses_spawn_tasks: false,
            source_runs: vec![ObjectSourceRun {
                instruction_start: 0,
                source: u32::try_from(source)
                    .map_err(|_| invalid("too many error declaration sources"))?,
                line: provenance.line,
                column: provenance.column,
            }],
            debug: Default::default(),
        });
    }
    Ok(result)
}
