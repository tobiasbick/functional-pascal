//! Links relocatable Functional Pascal objects into the VM's executable [`Chunk`].

mod append;
mod constants;
mod definitions;
mod functions;
mod globals;
mod layouts;
mod operands;
mod register_error;
mod relocation;
mod source_map;
mod strings;
mod symbols;

use std::fmt;

use fpas_bytecode::{Chunk, ExecutableError, validate_executable};
use fpas_unit::object::{ChunkDefinitionKind as DefinitionKind, ChunkObject, ChunkObjectError};

use append::{append_object, validate_retained_function_entries};
use definitions::validate_definitions_and_imports;

use functions::FunctionIds;
use globals::GlobalIds;
use layouts::LayoutIds;
pub use register_error::RegisterLinkError;

struct LinkIds {
    functions: FunctionIds,
    globals: GlobalIds,
    layouts: LayoutIds,
}

/// Object-linking failure with a deterministic diagnostic message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkError {
    /// An input object failed its structural validation.
    InvalidObject {
        /// Object owner.
        owner: String,
        /// Validation detail.
        detail: String,
    },
    /// Two objects define the same canonical runtime name.
    DuplicateDefinition(String),
    /// An import has no public compatible definition.
    UnresolvedImport {
        /// Importing object.
        owner: String,
        /// Missing definition.
        name: String,
        /// Required category.
        kind: DefinitionKind,
    },
    /// Two callable tables define the same name.
    DuplicateFunction(String),
    /// A callable definition has no implementation in its owning object.
    MissingFunctionImplementation {
        /// Object that declares the callable definition.
        owner: String,
        /// Callable definition without a matching function-table entry.
        name: String,
    },
    /// A Unit callable points at code removed during startup-section concatenation.
    StrippedFunctionEntry {
        /// Object containing the invalid function entry.
        owner: String,
        /// Callable name.
        name: String,
        /// Object-local callable offset.
        offset: u32,
        /// Number of instructions retained from the object.
        retained_code: usize,
    },
    /// A local or final index exceeds the bytecode representation.
    Overflow(&'static str),
    /// Relocation metadata does not match its opcode.
    InvalidRelocation {
        /// Object owner.
        owner: String,
        /// Object-local instruction.
        instruction: u32,
    },
    /// Constant insertion failed.
    ConstantPool(String),
    /// The completed linked executable violates a bytecode invariant.
    InvalidExecutable(ExecutableError),
    /// No executable root object was supplied.
    MissingProgram,
}

impl fmt::Display for LinkError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidObject { owner, detail } => {
                write!(formatter, "cannot link invalid object `{owner}`: {detail}")
            }
            Self::DuplicateDefinition(name) => {
                write!(formatter, "duplicate linked definition `{name}`")
            }
            Self::UnresolvedImport { owner, name, kind } => write!(
                formatter,
                "object `{owner}` requires missing public {kind:?} definition `{name}`"
            ),
            Self::DuplicateFunction(name) => {
                write!(formatter, "duplicate linked callable `{name}`")
            }
            Self::MissingFunctionImplementation { owner, name } => write!(
                formatter,
                "object `{owner}` defines callable `{name}` without a matching function implementation"
            ),
            Self::StrippedFunctionEntry {
                owner,
                name,
                offset,
                retained_code,
            } => write!(
                formatter,
                "callable `{name}` in object `{owner}` starts at {offset}, but only {retained_code} instructions remain after removing the Unit terminator"
            ),
            Self::Overflow(field) => write!(formatter, "linked {field} exceeds bytecode limits"),
            Self::InvalidRelocation { owner, instruction } => write!(
                formatter,
                "object `{owner}` has invalid relocation at instruction {instruction}"
            ),
            Self::ConstantPool(detail) => {
                write!(formatter, "cannot construct linked constant pool: {detail}")
            }
            Self::InvalidExecutable(error) => {
                write!(formatter, "linked executable validation failed: {error}")
            }
            Self::MissingProgram => write!(formatter, "cannot link without a program object"),
        }
    }
}

impl std::error::Error for LinkError {}

/// Link dependency-first unit objects followed by one root program object.
///
/// # Errors
///
/// Returns [`LinkError`] when an input object, symbol relationship, relocation,
/// or the completed executable image is invalid.
pub fn link_objects(units: &[ChunkObject], program: &ChunkObject) -> Result<Chunk, LinkError> {
    validate_objects(units, program)?;
    validate_definitions_and_imports(units, program)?;

    let mut chunk = Chunk::new();
    for object in units {
        append_object(&mut chunk, object, false)?;
    }
    append_object(&mut chunk, program, true)?;
    validate_executable(&chunk).map_err(LinkError::InvalidExecutable)?;
    Ok(chunk)
}

fn validate_objects(units: &[ChunkObject], program: &ChunkObject) -> Result<(), LinkError> {
    if program.code.is_empty() {
        return Err(LinkError::MissingProgram);
    }
    for object in units.iter().chain(std::iter::once(program)) {
        object
            .validate()
            .map_err(|error| invalid_object(object, error))?;
    }
    for object in units {
        validate_retained_function_entries(object)?;
    }
    Ok(())
}

fn invalid_object(object: &ChunkObject, error: ChunkObjectError) -> LinkError {
    LinkError::InvalidObject {
        owner: object.owner.clone(),
        detail: error.to_string(),
    }
}

/// Link dependency-first unit objects and one root object into a verified register executable.
///
/// IDs are assigned by dependency order and canonical symbol order, except that the root entry is
/// always function zero. Local registers are copied unchanged and are never relocated.
///
/// # Errors
///
/// Returns [`RegisterLinkError`] before executable publication for invalid objects, symbols,
/// visibility, ABI/layout mismatches, relocations, overflows, or final verifier rejection.
pub fn link_register_objects(
    units: &[fpas_unit::object::RelocatableObject],
    program: &fpas_unit::object::RelocatableObject,
) -> Result<fpas_bytecode::VerifiedExecutable, RegisterLinkError> {
    use std::collections::BTreeMap;

    use fpas_bytecode::{
        CodeRange, EnumLayout, EnumVariant, Executable, FunctionFlags, FunctionInfo, GlobalInfo,
        Instruction, InstructionAddress, NO_REGISTER, Opcode, RecordField, RecordLayout,
        ReturnConvention,
    };
    use fpas_unit::object::ObjectReturn;

    let objects = units
        .iter()
        .chain(std::iter::once(program))
        .collect::<Vec<_>>();
    for unit in units {
        unit.validate()
            .map_err(|error| RegisterLinkError::InvalidObject {
                owner: unit.owner.clone(),
                detail: error.to_string(),
            })?;
        if unit.entry.is_some() {
            return Err(RegisterLinkError::UnitEntry(unit.owner.clone()));
        }
    }
    program
        .validate()
        .map_err(|error| RegisterLinkError::InvalidObject {
            owner: program.owner.clone(),
            detail: error.to_string(),
        })?;
    let entry = program
        .entry
        .and_then(|index| usize::try_from(index).ok())
        .ok_or(RegisterLinkError::MissingProgramEntry)?;
    validate_register_initializers(units)?;
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
                .ok_or(RegisterLinkError::Overflow("unit initializer function ID"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let initializer_count = u32::try_from(initializer_targets.len())
        .map_err(|_| RegisterLinkError::Overflow("unit initializer call count"))?;
    let mut strings = strings::StringInterner::default();

    let linked_globals = ids
        .globals
        .order
        .iter()
        .map(|(object, local)| {
            let global = &objects[*object].globals[*local];
            Ok(GlobalInfo {
                name: strings.intern(&global.name)?,
                mutable: global.mutable,
            })
        })
        .collect::<Result<Vec<_>, RegisterLinkError>>()?;
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
                    .map(|field| strings.intern(field).map(|name| RecordField { name }))
                    .collect::<Result<Vec<_>, _>>()?,
            })
        })
        .collect::<Result<Vec<_>, RegisterLinkError>>()?;
    let mut linked_enums = Vec::with_capacity(ids.layouts.enum_order.len());
    let mut linked_variants = Vec::new();
    for (object, local) in &ids.layouts.enum_order {
        let enumeration = &objects[*object].enums[*local];
        linked_enums.push(EnumLayout {
            name: strings.intern(&enumeration.name)?,
        });
        let owner =
            ids.layouts.enums[*object][*local].ok_or(RegisterLinkError::Overflow("enum owner"))?;
        for variant in &enumeration.variants {
            linked_variants.push(EnumVariant {
                owner,
                name: strings.intern(&variant.name)?,
                fields: variant
                    .fields
                    .iter()
                    .map(|field| strings.intern(field))
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
                .ok_or(RegisterLinkError::Overflow("instruction addresses"))?,
        );
        code_length = code_length
            .checked_add(prefix)
            .ok_or(RegisterLinkError::Overflow("instruction addresses"))?
            .checked_add(
                u32::try_from(objects[*object].functions[*function].code.len())
                    .map_err(|_| RegisterLinkError::Overflow("function code length"))?,
            )
            .ok_or(RegisterLinkError::Overflow("instruction addresses"))?;
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
                        .map_err(|error| RegisterLinkError::Instruction(error.to_string()))?,
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
            let instruction =
                if let Some(relocation) =
                    relocation_map.get(&u32::try_from(instruction_index).map_err(|_| {
                        RegisterLinkError::Overflow("function-local instruction index")
                    })?)
                {
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
            .ok_or(RegisterLinkError::Overflow("function code range"))?
            .checked_add(
                u32::try_from(function.code.len())
                    .map_err(|_| RegisterLinkError::Overflow("function code range"))?,
            )
            .ok_or(RegisterLinkError::Overflow("function code range"))?;
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
        });
    }
    let source_map = source_map::merge(
        &objects,
        &ids.functions.order,
        &code_starts,
        &code_bases,
        &mut strings,
    )?;
    let executable = Executable {
        code,
        functions: linked_functions,
        constants: constants.values,
        strings: strings.finish(),
        globals: linked_globals,
        records: linked_records,
        enums: linked_enums,
        enum_variants: linked_variants,
        source_map,
        entry: fpas_bytecode::FunctionId::new(0),
    };
    executable
        .verify()
        .map_err(RegisterLinkError::InvalidExecutable)
}

fn validate_register_initializers(
    units: &[fpas_unit::object::RelocatableObject],
) -> Result<(), RegisterLinkError> {
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
            return Err(RegisterLinkError::InvalidInitializer {
                owner: object.owner.clone(),
                detail,
            });
        }
    }
    Ok(())
}
