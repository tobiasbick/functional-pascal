//! Links relocatable Functional Pascal objects into a verified executable.

mod constants;
mod debug_types;
mod error;
mod functions;
mod globals;
mod layouts;
mod relocation;
mod source_map;
mod strings;
mod symbols;

pub use error::LinkError;
use functions::FunctionIds;
use globals::GlobalIds;
use layouts::LayoutIds;

struct LinkIds {
    functions: FunctionIds,
    globals: GlobalIds,
    layouts: LayoutIds,
}
/// Link dependency-first unit objects and one root object into a verified register executable.
///
/// IDs are assigned by dependency order and canonical symbol order, except that the root entry is
/// always function zero. Local registers are copied unchanged and are never relocated.
///
/// # Errors
///
/// Returns [`LinkError`] before executable publication for invalid objects, symbols,
/// visibility, ABI/layout mismatches, relocations, overflows, or final verifier rejection.
pub fn link_objects(
    units: &[fpas_unit::object::RelocatableObject],
    program: &fpas_unit::object::RelocatableObject,
) -> Result<fpas_bytecode::VerifiedExecutable, LinkError> {
    use std::collections::BTreeMap;

    use fpas_bytecode::{
        CodeRange, EnumLayout, EnumVariant, Executable, FunctionFlags, FunctionInfo, GlobalInfo,
        Instruction, InstructionAddress, NO_REGISTER, Opcode, RecordField, RecordLayout,
        RecordProperty, ReturnConvention,
    };
    use fpas_unit::object::ObjectReturn;

    let objects = units
        .iter()
        .chain(std::iter::once(program))
        .collect::<Vec<_>>();
    for unit in units {
        unit.validate().map_err(|error| LinkError::InvalidObject {
            owner: unit.owner.clone(),
            detail: error.to_string(),
        })?;
        if unit.entry.is_some() {
            return Err(LinkError::UnitEntry(unit.owner.clone()));
        }
    }
    program
        .validate()
        .map_err(|error| LinkError::InvalidObject {
            owner: program.owner.clone(),
            detail: error.to_string(),
        })?;
    let entry = program
        .entry
        .and_then(|index| usize::try_from(index).ok())
        .ok_or(LinkError::MissingProgramEntry)?;
    validate_initializers(units)?;
    let program_index = units.len();
    let symbols = symbols::SymbolTable::build(&objects)?;
    let functions = functions::assign(&objects, program_index, entry, &symbols)?;
    let globals = globals::assign(&objects, &symbols)?;
    let layouts = layouts::assign(&objects, &symbols)?;
    let ids = LinkIds {
        functions,
        globals,
        layouts,
    };
    let (debug_type_ids, linked_debug_types) = debug_types::merge(&objects, &ids, &symbols)?;
    let initializer_targets = units
        .iter()
        .enumerate()
        .filter_map(|(object_index, object)| {
            object
                .initializer
                .map(|initializer| (object_index, initializer as usize))
        })
        .map(|(object_index, initializer)| {
            ids.functions.maps[object_index][initializer]
                .ok_or(LinkError::Overflow("unit initializer function ID"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let initializer_count = u32::try_from(initializer_targets.len())
        .map_err(|_| LinkError::Overflow("unit initializer call count"))?;
    let mut strings = strings::StringInterner::default();

    let mut linked_globals = ids
        .globals
        .order
        .iter()
        .map(|(object, local)| {
            let global = &objects[*object].globals[*local];
            Ok(GlobalInfo {
                name: strings.intern(&global.name)?,
                ty: debug_type_ids.translate(*object, global.ty)?,
                mutable: global.mutable,
                initializer: None,
            })
        })
        .collect::<Result<Vec<_>, LinkError>>()?;
    let linked_records = ids
        .layouts
        .record_order
        .iter()
        .map(|(object, local)| {
            let record = &objects[*object].records[*local];
            Ok(RecordLayout {
                name: strings.intern(&record.name)?,
                fields: record
                    .fields
                    .iter()
                    .zip(&record.field_types)
                    .map(|(field, ty)| {
                        Ok(RecordField {
                            name: strings.intern(field)?,
                            ty: debug_type_ids.translate(*object, *ty)?,
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                properties: record
                    .properties
                    .iter()
                    .map(|property| {
                        Ok(RecordProperty {
                            name: strings.intern(&property.name)?,
                            getter: strings.intern(&property.getter)?,
                        })
                    })
                    .collect::<Result<Vec<_>, LinkError>>()?,
                methods: record
                    .methods
                    .iter()
                    .map(|method| {
                        Ok(fpas_bytecode::RecordMethod {
                            name: strings.intern(&method.name)?,
                            routine: strings.intern(&method.routine)?,
                        })
                    })
                    .collect::<Result<Vec<_>, LinkError>>()?,
            })
        })
        .collect::<Result<Vec<_>, LinkError>>()?;
    let mut linked_enums = Vec::with_capacity(ids.layouts.enum_order.len());
    let mut linked_variants = Vec::new();
    for (object, local) in &ids.layouts.enum_order {
        let enumeration = &objects[*object].enums[*local];
        linked_enums.push(EnumLayout {
            name: strings.intern(&enumeration.name)?,
        });
        let owner = ids.layouts.enums[*object][*local].ok_or(LinkError::Overflow("enum owner"))?;
        for variant in &enumeration.variants {
            linked_variants.push(EnumVariant {
                owner,
                name: strings.intern(&variant.name)?,
                fields: variant
                    .fields
                    .iter()
                    .map(|field| strings.intern(field))
                    .collect::<Result<Vec<_>, _>>()?,
                field_types: variant
                    .field_types
                    .iter()
                    .map(|ty| debug_type_ids.translate(*object, *ty))
                    .collect::<Result<Vec<_>, _>>()?,
            });
        }
    }
    let constants = constants::merge(&objects, &symbols, &ids, &mut strings)?;

    let mut code_starts = Vec::with_capacity(ids.functions.order.len());
    let mut code_bases = Vec::with_capacity(ids.functions.order.len());
    let mut code_length = 0_u32;
    for (final_index, (object, function)) in ids.functions.order.iter().enumerate() {
        code_starts.push(code_length);
        let prefix = if final_index == 0 {
            initializer_count
        } else {
            0
        };
        code_bases.push(
            code_length
                .checked_add(prefix)
                .ok_or(LinkError::Overflow("instruction addresses"))?,
        );
        code_length = code_length
            .checked_add(prefix)
            .ok_or(LinkError::Overflow("instruction addresses"))?
            .checked_add(
                u32::try_from(objects[*object].functions[*function].code.len())
                    .map_err(|_| LinkError::Overflow("function code length"))?,
            )
            .ok_or(LinkError::Overflow("instruction addresses"))?;
    }
    for (linked, (object, local)) in linked_globals.iter_mut().zip(&ids.globals.order) {
        let Some(initializer) = objects[*object].globals[*local].initializer else {
            continue;
        };
        let function = ids.functions.maps[*object]
            .get(initializer.function as usize)
            .copied()
            .flatten()
            .ok_or(LinkError::Overflow("global initializer function ID"))?;
        let final_function = ids
            .functions
            .order
            .iter()
            .position(|entry| *entry == (*object, initializer.function as usize))
            .ok_or(LinkError::Overflow("global initializer function order"))?;
        let instruction = InstructionAddress::new(
            code_bases[final_function]
                .checked_add(initializer.instruction_start)
                .ok_or(LinkError::Overflow(
                    "global initializer instruction address",
                ))?,
        );
        linked.initializer = Some(fpas_bytecode::GlobalInitializer {
            function,
            instruction,
        });
    }
    let mut code = Vec::with_capacity(code_length as usize);
    let mut linked_functions = Vec::with_capacity(ids.functions.order.len());
    for (final_index, (object_index, function_index)) in
        ids.functions.order.iter().copied().enumerate()
    {
        let object = objects[object_index];
        let function = &object.functions[function_index];
        let code_start = code_starts[final_index];
        let code_base = code_bases[final_index];
        if final_index == 0 {
            for target in &initializer_targets {
                code.push(
                    Instruction::abc(Opcode::CallDirect, NO_REGISTER, target.get(), 0, 0)
                        .map_err(|error| LinkError::Instruction(error.to_string()))?,
                );
            }
        }
        let relocation_map = object
            .relocations
            .iter()
            .filter(|relocation| relocation.function as usize == function_index)
            .map(|relocation| (relocation.instruction, relocation))
            .collect::<BTreeMap<_, _>>();
        for (instruction_index, word) in function.code.iter().copied().enumerate() {
            let instruction = Instruction::from_word(word);
            let instruction = if let Some(relocation) = relocation_map.get(
                &u32::try_from(instruction_index)
                    .map_err(|_| LinkError::Overflow("function-local instruction index"))?,
            ) {
                relocation::relocate(
                    &objects,
                    object_index,
                    function_index,
                    instruction,
                    relocation,
                    code_base,
                    &symbols,
                    &ids,
                    &constants,
                )?
            } else {
                instruction
            };
            code.push(instruction);
        }
        let end = code_start
            .checked_add(if final_index == 0 {
                initializer_count
            } else {
                0
            })
            .ok_or(LinkError::Overflow("function code range"))?
            .checked_add(
                u32::try_from(function.code.len())
                    .map_err(|_| LinkError::Overflow("function code range"))?,
            )
            .ok_or(LinkError::Overflow("function code range"))?;
        linked_functions.push(FunctionInfo {
            name: strings.intern(&function.name)?,
            code: CodeRange::new(
                InstructionAddress::new(code_start),
                InstructionAddress::new(end),
            ),
            arity: function.arity,
            capture_count: function.capture_count,
            register_count: function.register_count,
            return_convention: match function.returns {
                ObjectReturn::Unit => ReturnConvention::Unit,
                ObjectReturn::Value => ReturnConvention::Value,
            },
            flags: FunctionFlags {
                uses_spawn_tasks: function.uses_spawn_tasks,
            },
            debug: fpas_bytecode::FunctionDebugInfo::default(),
        });
    }
    let (source_map, function_debug) = source_map::merge(
        &objects,
        &ids.functions.order,
        &ids.functions.maps,
        &code_starts,
        &code_bases,
        &debug_type_ids,
        &mut strings,
    )?;
    for (function, debug) in linked_functions.iter_mut().zip(function_debug) {
        function.debug = debug;
    }
    let executable = Executable {
        code,
        functions: linked_functions,
        constants: constants.values,
        strings: strings.finish(),
        globals: linked_globals,
        records: linked_records,
        enums: linked_enums,
        enum_variants: linked_variants,
        debug_types: linked_debug_types,
        source_map,
        entry: fpas_bytecode::FunctionId::new(0),
    };
    executable.verify().map_err(LinkError::InvalidExecutable)
}

fn validate_initializers(units: &[fpas_unit::object::RelocatableObject]) -> Result<(), LinkError> {
    use fpas_unit::object::ObjectReturn;

    for object in units {
        let Some(initializer) = object.initializer else {
            continue;
        };
        let function = &object.functions[initializer as usize];
        let detail = if function.arity != 0 {
            Some("expected zero parameters")
        } else if function.capture_count != 0 {
            Some("expected zero captures")
        } else if function.returns != ObjectReturn::Unit {
            Some("expected Unit return convention")
        } else {
            None
        };
        if let Some(detail) = detail {
            return Err(LinkError::InvalidInitializer {
                owner: object.owner.clone(),
                detail,
            });
        }
    }
    Ok(())
}
