//! Links relocatable Functional Pascal objects into the VM's executable [`Chunk`].

mod operands;

use std::collections::{HashMap, HashSet};
use std::fmt;

use fpas_bytecode::{Chunk, Op, SourceLocation};
use fpas_unit::object::{
    DefinitionKind, ObjectDefinition, ObjectError, ObjectLocation, RelocatableObject,
};

use operands::relocate_instruction;

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
            Self::Overflow(field) => write!(formatter, "linked {field} exceeds bytecode limits"),
            Self::InvalidRelocation { owner, instruction } => write!(
                formatter,
                "object `{owner}` has invalid relocation at instruction {instruction}"
            ),
            Self::ConstantPool(detail) => {
                write!(formatter, "cannot construct linked constant pool: {detail}")
            }
            Self::MissingProgram => write!(formatter, "cannot link without a program object"),
        }
    }
}

impl std::error::Error for LinkError {}

/// Link dependency-first unit objects followed by one root program object.
pub fn link_objects(
    units: &[RelocatableObject],
    program: &RelocatableObject,
) -> Result<Chunk, LinkError> {
    validate_objects(units, program)?;
    validate_definitions_and_imports(units, program)?;

    let mut chunk = Chunk::new();
    for object in units {
        append_object(&mut chunk, object, false)?;
    }
    append_object(&mut chunk, program, true)?;
    chunk
        .validate_invariants()
        .map_err(|error| LinkError::InvalidObject {
            owner: program.owner.clone(),
            detail: error.to_string(),
        })?;
    Ok(chunk)
}

fn validate_objects(
    units: &[RelocatableObject],
    program: &RelocatableObject,
) -> Result<(), LinkError> {
    if program.code.is_empty() {
        return Err(LinkError::MissingProgram);
    }
    for object in units.iter().chain(std::iter::once(program)) {
        object
            .validate()
            .map_err(|error| invalid_object(object, error))?;
        if object.code[..object.code.len() - 1]
            .iter()
            .any(|op| matches!(op, Op::Halt))
        {
            return Err(LinkError::InvalidObject {
                owner: object.owner.clone(),
                detail: "an internal Halt would stop later startup sections".to_string(),
            });
        }
    }
    Ok(())
}

fn invalid_object(object: &RelocatableObject, error: ObjectError) -> LinkError {
    LinkError::InvalidObject {
        owner: object.owner.clone(),
        detail: error.to_string(),
    }
}

fn validate_definitions_and_imports(
    units: &[RelocatableObject],
    program: &RelocatableObject,
) -> Result<(), LinkError> {
    let mut definitions = HashMap::<String, &ObjectDefinition>::new();
    for definition in units
        .iter()
        .chain(std::iter::once(program))
        .flat_map(|object| &object.definitions)
    {
        let key = definition.name.to_ascii_lowercase();
        if definitions.insert(key, definition).is_some() {
            return Err(LinkError::DuplicateDefinition(definition.name.clone()));
        }
    }
    for object in units.iter().chain(std::iter::once(program)) {
        for import in &object.imports {
            let key = import.name.to_ascii_lowercase();
            let Some(definition) = definitions.get(&key) else {
                return Err(LinkError::UnresolvedImport {
                    owner: object.owner.clone(),
                    name: import.name.clone(),
                    kind: import.kind,
                });
            };
            if !definition.public || definition.kind != import.kind {
                return Err(LinkError::UnresolvedImport {
                    owner: object.owner.clone(),
                    name: import.name.clone(),
                    kind: import.kind,
                });
            }
        }
    }
    Ok(())
}

fn append_object(
    chunk: &mut Chunk,
    object: &RelocatableObject,
    retain_halt: bool,
) -> Result<(), LinkError> {
    let code_base = u32::try_from(chunk.len()).map_err(|_| LinkError::Overflow("code"))?;
    let mut constant_map = Vec::with_capacity(object.constants.len());
    for constant in &object.constants {
        let index = chunk
            .add_constant(constant.to_value())
            .map_err(|error| LinkError::ConstantPool(error.to_string()))?;
        constant_map.push(index);
    }

    let relocation_by_instruction = relocation_map(object);
    let code_length = object.code.len() - usize::from(!retain_halt);
    for (offset, (op, location)) in object
        .code
        .iter()
        .zip(&object.locations)
        .take(code_length)
        .enumerate()
    {
        let mut relocated = *op;
        if let Some(relocations) = relocation_by_instruction.get(&(offset as u32)) {
            for relocation in relocations {
                relocate_instruction(&mut relocated, relocation.kind, &constant_map, code_base)
                    .map_err(|()| LinkError::InvalidRelocation {
                        owner: object.owner.clone(),
                        instruction: offset as u32,
                    })?;
            }
        }
        chunk.emit(relocated, source_location(*location));
    }

    let mut function_names = HashSet::new();
    for (name, function) in &object.functions {
        let key = name.to_ascii_lowercase();
        if !function_names.insert(key.clone()) || chunk.functions().contains_key(&key) {
            return Err(LinkError::DuplicateFunction(name.clone()));
        }
        let start = code_base
            .checked_add(function.code_start)
            .ok_or(LinkError::Overflow("function address"))?;
        chunk.insert_function(key, start as usize, function.arity);
    }
    Ok(())
}

fn relocation_map(object: &RelocatableObject) -> HashMap<u32, Vec<fpas_unit::object::Relocation>> {
    let mut result = HashMap::<u32, Vec<_>>::new();
    for relocation in &object.relocations {
        result
            .entry(relocation.instruction)
            .or_default()
            .push(*relocation);
    }
    result
}

fn source_location(location: ObjectLocation) -> SourceLocation {
    SourceLocation::new_with_source(location.line, location.column, location.source_id)
}
