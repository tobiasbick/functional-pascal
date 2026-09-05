//! Deterministic relocatable register-bytecode unit objects.

mod codec;
mod conversion;
mod debug_types;
mod error;
mod from_executable;
mod function;
mod metadata;
mod relocation;
mod symbol;
mod validation;

use std::collections::BTreeMap;

use fpas_bytecode::Instruction;

pub use codec::{decode_object, encode_object};
pub use error::ObjectError;
pub use function::{ObjectFunction, ObjectReturn};
pub use metadata::{
    ObjectCaptureKind, ObjectCaptureSource, ObjectConstant, ObjectDebugBinding,
    ObjectDebugBindingKind, ObjectDebugLocation, ObjectDebugScope, ObjectDebugType,
    ObjectEnumLayout, ObjectEnumVariant, ObjectFunctionDebugInfo, ObjectGlobal, ObjectInitializer,
    ObjectRecordLayout, ObjectRecordMethod, ObjectRecordProperty, ObjectSequencePoint,
    ObjectSourceRun,
};
pub use relocation::{Relocation, RelocationKind};
pub use symbol::{
    DefinitionTarget, ImportShape, ObjectDefinition, ObjectImport, SymbolKind, SymbolReference,
};

use validation::{
    relocation_category, validate_import_shape, validate_name, validate_name_order,
    validate_source_runs, validate_unique_names,
};

/// Schema version embedded in every encoded register object payload.
pub const OBJECT_VERSION: u16 = 7;

/// Independently compiled register-bytecode object with symbolic external references.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RelocatableObject {
    /// Object schema version.
    pub version: u16,
    /// Canonical owner unit, or root program name.
    pub owner: String,
    /// Root entry function for a program object; absent for unit objects.
    pub entry: Option<u32>,
    /// Unit initialization function; absent for root program objects.
    pub initializer: Option<u32>,
    /// Independently encoded functions.
    pub functions: Vec<ObjectFunction>,
    /// Persistent constants in object-local order.
    pub constants: Vec<ObjectConstant>,
    /// Dense object-local globals.
    pub globals: Vec<ObjectGlobal>,
    /// Object-local record layouts.
    pub records: Vec<ObjectRecordLayout>,
    /// Object-local enum layouts.
    pub enums: Vec<ObjectEnumLayout>,
    /// Object-local portable debugger type graph.
    pub debug_types: Vec<ObjectDebugType>,
    /// Object-local source paths.
    pub sources: Vec<String>,
    /// Ordered definitions supplied by this object.
    pub definitions: Vec<ObjectDefinition>,
    /// Ordered definitions required from dependencies.
    pub imports: Vec<ObjectImport>,
    /// Complete instruction relocation table.
    pub relocations: Vec<Relocation>,
}

impl RelocatableObject {
    /// Add private definitions for every named local table entry.
    ///
    /// # Errors
    ///
    /// Returns [`ObjectError`] when a table index is not representable.
    pub fn define_all_private(&mut self) -> Result<(), ObjectError> {
        self.definitions.clear();
        for (index, function) in self.functions.iter().enumerate() {
            self.definitions.push(ObjectDefinition {
                name: canonical(&function.name),
                target: DefinitionTarget::Function(
                    u32::try_from(index).map_err(|_| ObjectError::Overflow("function index"))?,
                ),
                public: false,
            });
        }
        for (index, global) in self.globals.iter().enumerate() {
            self.definitions.push(ObjectDefinition {
                name: canonical(&global.name),
                target: DefinitionTarget::Global(
                    u32::try_from(index).map_err(|_| ObjectError::Overflow("global index"))?,
                ),
                public: false,
            });
        }
        for (index, record) in self.records.iter().enumerate() {
            self.definitions.push(ObjectDefinition {
                name: canonical(&record.name),
                target: DefinitionTarget::Record(
                    u32::try_from(index).map_err(|_| ObjectError::Overflow("record index"))?,
                ),
                public: false,
            });
        }
        for (index, enumeration) in self.enums.iter().enumerate() {
            self.definitions.push(ObjectDefinition {
                name: canonical(&enumeration.name),
                target: DefinitionTarget::Enum(
                    u32::try_from(index).map_err(|_| ObjectError::Overflow("enum index"))?,
                ),
                public: false,
            });
        }
        self.definitions
            .sort_by(|left, right| left.name.cmp(&right.name));
        Ok(())
    }

    /// Validate object-local tables, symbols, code, source runs, and relocation coverage.
    ///
    /// # Errors
    ///
    /// Returns a structured [`ObjectError`] for the first deterministic violation.
    pub fn validate(&self) -> Result<(), ObjectError> {
        if self.version != OBJECT_VERSION {
            return Err(ObjectError::Version {
                actual: self.version,
                expected: OBJECT_VERSION,
            });
        }
        if self.owner.is_empty() || self.owner != canonical(&self.owner) {
            return Err(ObjectError::NonCanonicalName(self.owner.clone()));
        }
        if self
            .entry
            .is_some_and(|entry| entry as usize >= self.functions.len())
        {
            return Err(ObjectError::InvalidTableReference("entry function"));
        }
        if self
            .initializer
            .is_some_and(|initializer| initializer as usize >= self.functions.len())
        {
            return Err(ObjectError::InvalidTableReference("initializer function"));
        }
        if self.entry.is_some() && self.initializer.is_some() {
            return Err(ObjectError::InvalidTableReference(
                "entry and initializer are mutually exclusive",
            ));
        }
        for global in &self.globals {
            if let Some(initializer) = global.initializer {
                let instruction = self
                    .functions
                    .get(initializer.function as usize)
                    .and_then(|function| function.code.get(initializer.instruction_start as usize))
                    .copied()
                    .map(Instruction::from_word)
                    .ok_or(ObjectError::InvalidTableReference(
                        "global source initializer instruction",
                    ))?;
                if instruction.opcode().ok() != Some(fpas_bytecode::Opcode::StoreGlobal) {
                    return Err(ObjectError::InvalidTableReference(
                        "global source initializer store",
                    ));
                }
            }
        }
        debug_types::validate_debug_types(
            &self.debug_types,
            &self.globals,
            &self.records,
            &self.enums,
        )?;
        validate_unique_names(self.definitions.iter().map(|definition| &definition.name))?;
        validate_unique_names(self.imports.iter().map(|import| &import.name))?;
        validate_name_order(
            self.definitions.iter().map(|definition| &definition.name),
            "definitions",
        )?;
        validate_name_order(self.imports.iter().map(|import| &import.name), "imports")?;
        for definition in &self.definitions {
            validate_name(&definition.name)?;
            self.validate_target(definition.target)?;
        }
        for import in &self.imports {
            validate_name(&import.name)?;
            validate_import_shape(&import.shape)?;
        }
        for constant in &self.constants {
            if let ObjectConstant::Function { function, .. } = constant {
                self.validate_reference(*function, SymbolKind::Function)?;
            }
        }
        let mut actual = BTreeMap::new();
        let mut previous_relocation = None;
        for relocation in &self.relocations {
            let key = (relocation.function, relocation.instruction);
            if previous_relocation.is_some_and(|previous| previous >= key) {
                return Err(ObjectError::NonDeterministicOrder("relocations"));
            }
            previous_relocation = Some(key);
            if actual.insert(key, relocation).is_some() {
                return Err(ObjectError::DuplicateRelocation {
                    function: relocation.function,
                    instruction: relocation.instruction,
                });
            }
            self.validate_relocation(relocation)?;
        }
        for (function_index, function) in self.functions.iter().enumerate() {
            if function.code.is_empty() {
                return Err(ObjectError::EmptyFunction {
                    function: function.name.clone(),
                });
            }
            validate_source_runs(function, self.sources.len())?;
            validation::validate_debug_info(
                function,
                &self.functions,
                self.sources.len(),
                self.debug_types.len(),
            )?;
            for (instruction_index, word) in function.code.iter().copied().enumerate() {
                let instruction = Instruction::from_word(word);
                let expected = relocation_category(instruction)?;
                let key = (
                    u32::try_from(function_index)
                        .map_err(|_| ObjectError::Overflow("function index"))?,
                    u32::try_from(instruction_index)
                        .map_err(|_| ObjectError::Overflow("instruction index"))?,
                );
                match (expected, actual.get(&key)) {
                    (Some(category), Some(relocation)) if category.matches(&relocation.kind) => {}
                    (None, None) => {}
                    _ => {
                        return Err(ObjectError::RelocationCoverage {
                            function: key.0,
                            instruction: key.1,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_target(&self, target: DefinitionTarget) -> Result<(), ObjectError> {
        let valid = match target {
            DefinitionTarget::Function(index) => (index as usize) < self.functions.len(),
            DefinitionTarget::Global(index) => (index as usize) < self.globals.len(),
            DefinitionTarget::Record(index) => (index as usize) < self.records.len(),
            DefinitionTarget::Enum(index) => (index as usize) < self.enums.len(),
        };
        valid
            .then_some(())
            .ok_or(ObjectError::InvalidDefinitionTarget(target))
    }

    fn validate_reference(
        &self,
        reference: SymbolReference,
        expected: SymbolKind,
    ) -> Result<(), ObjectError> {
        match reference {
            SymbolReference::Local(index) => {
                let valid = match expected {
                    SymbolKind::Function => (index as usize) < self.functions.len(),
                    SymbolKind::Global => (index as usize) < self.globals.len(),
                    SymbolKind::Record => (index as usize) < self.records.len(),
                    SymbolKind::Enum => (index as usize) < self.enums.len(),
                };
                valid
                    .then_some(())
                    .ok_or(ObjectError::InvalidLocalReference {
                        kind: expected,
                        index,
                    })
            }
            SymbolReference::Import(index) => {
                let import = self
                    .imports
                    .get(index as usize)
                    .ok_or(ObjectError::InvalidTableReference("import"))?;
                if import.shape.kind() == expected {
                    Ok(())
                } else {
                    Err(ObjectError::ReferenceKind {
                        expected,
                        actual: import.shape.kind(),
                    })
                }
            }
        }
    }

    fn validate_relocation(&self, relocation: &Relocation) -> Result<(), ObjectError> {
        let function = self
            .functions
            .get(relocation.function as usize)
            .ok_or(ObjectError::InvalidTableReference("relocation function"))?;
        if relocation.instruction as usize >= function.code.len() {
            return Err(ObjectError::InvalidTableReference("relocation instruction"));
        }
        match &relocation.kind {
            RelocationKind::Constant(index) if (*index as usize) < self.constants.len() => Ok(()),
            RelocationKind::Function(reference) => {
                self.validate_reference(*reference, SymbolKind::Function)
            }
            RelocationKind::Global(reference) => {
                self.validate_reference(*reference, SymbolKind::Global)
            }
            RelocationKind::Record(reference) => {
                self.validate_reference(*reference, SymbolKind::Record)
            }
            RelocationKind::RecordField(_) | RelocationKind::EnumField(_) => Ok(()),
            RelocationKind::EnumVariant {
                enumeration,
                variant,
            } => {
                validate_name(variant)?;
                self.validate_reference(*enumeration, SymbolKind::Enum)
            }
            RelocationKind::CodeAddress(target) if (*target as usize) < function.code.len() => {
                Ok(())
            }
            RelocationKind::Constant(_) | RelocationKind::CodeAddress(_) => {
                Err(ObjectError::InvalidRelocationTarget {
                    function: relocation.function,
                    instruction: relocation.instruction,
                })
            }
        }
    }
}

pub(super) fn canonical(name: &str) -> String {
    name.to_ascii_lowercase()
}
