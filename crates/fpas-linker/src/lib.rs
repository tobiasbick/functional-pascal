//! Links relocatable Functional Pascal objects into a verified executable.

mod constants;
mod debug_types;
mod error;
mod functions;
mod globals;
mod layouts;
mod plan;
mod relocation;
mod source_map;
mod strings;
mod symbols;

pub use error::LinkError;
use plan::LinkPlan;
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

    let LinkPlan {
        objects,
        symbols,
        ids,
        debug_type_ids,
        linked_debug_types,
        initializer_targets,
        code_layout,
    } = LinkPlan::build(units, program)?;
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

    for (linked, (object, local)) in linked_globals.iter_mut().zip(&ids.globals.order) {
        let Some(initializer) = objects[*object].globals[*local].initializer else {
            continue;
        };
        let function = ids.functions.maps[*object]
            .get(initializer.function as usize)
            .copied()
            .flatten()
            .ok_or(LinkError::Overflow("global initializer function ID"))?;
        let instruction = InstructionAddress::new(
            code_layout
                .base_for(function)?
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
    let mut code = Vec::with_capacity(code_layout.length as usize);
    let mut linked_functions = Vec::with_capacity(ids.functions.order.len());
    for (final_index, (object_index, function_index)) in
        ids.functions.order.iter().copied().enumerate()
    {
        let object = objects[object_index];
        let function = &object.functions[function_index];
        let code_start = code_layout.starts[final_index];
        let code_base = code_layout.bases[final_index];
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
                code_layout.initializer_count
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
        &code_layout.starts,
        &code_layout.bases,
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
