//! Deterministic relocatable register-bytecode unit objects.

mod codec;
mod conversion;
mod error;
mod function;
mod metadata;
mod relocation;
mod symbol;
mod validation;

use std::collections::BTreeMap;

use fpas_bytecode::{Instruction, VerifiedExecutable};

pub use codec::{decode_object, encode_object};
pub use error::ObjectError;
pub use function::{ObjectFunction, ObjectReturn};
pub use metadata::{
    ObjectConstant, ObjectDebugBinding, ObjectDebugBindingKind, ObjectDebugLocation,
    ObjectDebugScope, ObjectDebugType, ObjectEnumLayout, ObjectEnumVariant,
    ObjectFunctionDebugInfo, ObjectGlobal, ObjectRecordLayout, ObjectRecordProperty,
    ObjectSequencePoint, ObjectSourceRun,
};
pub use relocation::{Relocation, RelocationKind};
pub use symbol::{
    DefinitionTarget, ImportShape, ObjectDefinition, ObjectImport, SymbolKind, SymbolReference,
};

use conversion::{localize_branch, object_debug_type, relocation_for_instruction};
use validation::{
    relocation_category, validate_import_shape, validate_name, validate_name_order,
    validate_source_runs, validate_unique_names,
};

/// Schema version embedded in every encoded register object payload.
pub const OBJECT_VERSION: u16 = 4;

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
    /// Convert one verified executable into a self-contained relocatable program object.
    ///
    /// Numeric table references become object-local relocations and branch targets become
    /// function-local. Callers may subsequently replace local references with imports before
    /// validation and encoding.
    ///
    /// # Errors
    ///
    /// Returns [`ObjectError`] when verified metadata cannot be represented by the object schema.
    pub fn from_executable(
        owner: impl Into<String>,
        verified: VerifiedExecutable,
    ) -> Result<Self, ObjectError> {
        let executable = verified.into_unverified();
        let strings = |id: fpas_bytecode::StringId| {
            executable
                .strings
                .get(id)
                .map(str::to_owned)
                .ok_or(ObjectError::InvalidTableReference("string"))
        };
        let sources = executable
            .source_map
            .sources
            .iter()
            .map(|id| strings(*id))
            .collect::<Result<Vec<_>, _>>()?;
        let constants = executable
            .constants
            .iter()
            .map(|constant| match constant {
                fpas_bytecode::Constant::Integer(value) => Ok(ObjectConstant::Integer(*value)),
                fpas_bytecode::Constant::Real(bits) => Ok(ObjectConstant::Real(*bits)),
                fpas_bytecode::Constant::Boolean(value) => Ok(ObjectConstant::Boolean(*value)),
                fpas_bytecode::Constant::String(id) => strings(*id).map(ObjectConstant::String),
                fpas_bytecode::Constant::Unit => Ok(ObjectConstant::Unit),
                fpas_bytecode::Constant::Function {
                    function,
                    task_bound,
                } => Ok(ObjectConstant::Function {
                    function: SymbolReference::Local(u32::from(function.get())),
                    task_bound: *task_bound,
                }),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let globals = executable
            .globals
            .iter()
            .map(|global| {
                Ok(ObjectGlobal {
                    name: strings(global.name)?,
                    ty: global.ty.get(),
                    mutable: global.mutable,
                })
            })
            .collect::<Result<Vec<_>, ObjectError>>()?;
        let records = executable
            .records
            .iter()
            .map(|record| {
                Ok(ObjectRecordLayout {
                    name: strings(record.name)?,
                    fields: record
                        .fields
                        .iter()
                        .map(|field| strings(field.name))
                        .collect::<Result<Vec<_>, _>>()?,
                    field_types: record.fields.iter().map(|field| field.ty.get()).collect(),
                    properties: record
                        .properties
                        .iter()
                        .map(|property| {
                            Ok(ObjectRecordProperty {
                                name: strings(property.name)?,
                                getter: strings(property.getter)?,
                            })
                        })
                        .collect::<Result<Vec<_>, ObjectError>>()?,
                })
            })
            .collect::<Result<Vec<_>, ObjectError>>()?;
        let mut variants_by_owner = vec![Vec::new(); executable.enums.len()];
        for variant in &executable.enum_variants {
            let fields = variant
                .fields
                .iter()
                .map(|field| strings(*field))
                .collect::<Result<Vec<_>, _>>()?;
            let Some(owner) = variants_by_owner.get_mut(usize::from(variant.owner.get())) else {
                return Err(ObjectError::InvalidTableReference("enum owner"));
            };
            owner.push(ObjectEnumVariant {
                name: strings(variant.name)?,
                fields,
                field_types: variant.field_types.iter().map(|ty| ty.get()).collect(),
            });
        }
        let enums = executable
            .enums
            .iter()
            .zip(variants_by_owner)
            .map(|(layout, variants)| {
                Ok(ObjectEnumLayout {
                    name: strings(layout.name)?,
                    variants,
                })
            })
            .collect::<Result<Vec<_>, ObjectError>>()?;

        let mut functions = Vec::with_capacity(executable.functions.len());
        let mut relocations = Vec::new();
        for (function_index, function) in executable.functions.iter().enumerate() {
            let start = usize::try_from(function.code.start.get())
                .map_err(|_| ObjectError::Overflow("function start"))?;
            let end = usize::try_from(function.code.end.get())
                .map_err(|_| ObjectError::Overflow("function end"))?;
            let code = executable
                .code
                .get(start..end)
                .ok_or(ObjectError::InvalidTableReference("function code"))?;
            let mut local_code = Vec::with_capacity(code.len());
            for (instruction_index, instruction) in code.iter().copied().enumerate() {
                let local = localize_branch(instruction, function.code.start.get())?;
                local_code.push(local.word());
                if let Some(kind) =
                    relocation_for_instruction(local, &executable.enum_variants, &enums)?
                {
                    relocations.push(Relocation {
                        function: u32::try_from(function_index)
                            .map_err(|_| ObjectError::Overflow("function index"))?,
                        instruction: u32::try_from(instruction_index)
                            .map_err(|_| ObjectError::Overflow("instruction index"))?,
                        kind,
                    });
                }
            }
            let source_runs = executable
                .source_map
                .runs
                .iter()
                .filter(|run| function.code.contains(run.instruction_start))
                .map(|run| ObjectSourceRun {
                    instruction_start: run.instruction_start.get() - function.code.start.get(),
                    source: run.source.get(),
                    line: run.line,
                    column: run.column,
                })
                .collect();
            let debug = ObjectFunctionDebugInfo {
                scopes: function
                    .debug
                    .scopes
                    .iter()
                    .map(|scope| ObjectDebugScope {
                        id: scope.id,
                        parent: scope.parent,
                    })
                    .collect(),
                bindings: function
                    .debug
                    .bindings
                    .iter()
                    .map(|binding| {
                        Ok(ObjectDebugBinding {
                            name: strings(binding.name)?,
                            type_name: strings(binding.type_name)?,
                            ty: binding.ty.get(),
                            register: binding.register.get(),
                            kind: match binding.kind {
                                fpas_bytecode::DebugBindingKind::Parameter => {
                                    ObjectDebugBindingKind::Parameter
                                }
                                fpas_bytecode::DebugBindingKind::Local => {
                                    ObjectDebugBindingKind::Local
                                }
                                fpas_bytecode::DebugBindingKind::Capture => {
                                    ObjectDebugBindingKind::Capture
                                }
                            },
                            mutable: binding.mutable,
                            scope: binding.scope,
                            declaration: binding.declaration.map(|location| ObjectDebugLocation {
                                source: location.source.get(),
                                line: location.line,
                                column: location.column,
                            }),
                            hidden: binding.hidden,
                            cell_backed: binding.cell_backed,
                        })
                    })
                    .collect::<Result<Vec<_>, ObjectError>>()?,
                sequence_points: function
                    .debug
                    .sequence_points
                    .iter()
                    .map(|point| ObjectSequencePoint {
                        instruction_start: point.instruction.get() - function.code.start.get(),
                        location: ObjectDebugLocation {
                            source: point.location.source.get(),
                            line: point.location.line,
                            column: point.location.column,
                        },
                        scope: point.scope,
                    })
                    .collect(),
                result_type: function.debug.result_type.map(|ty| ty.get()),
            };
            functions.push(ObjectFunction {
                name: strings(function.name)?,
                code: local_code,
                arity: function.arity,
                capture_count: function.capture_count,
                register_count: function.register_count,
                returns: match function.return_convention {
                    fpas_bytecode::ReturnConvention::Unit => ObjectReturn::Unit,
                    fpas_bytecode::ReturnConvention::Value => ObjectReturn::Value,
                },
                uses_spawn_tasks: function.flags.uses_spawn_tasks,
                source_runs,
                debug,
            });
        }
        let entry = Some(u32::from(executable.entry.get()));
        let debug_types = executable
            .debug_types
            .iter()
            .map(|ty| object_debug_type(ty, &executable))
            .collect::<Result<Vec<_>, ObjectError>>()?;
        let mut object = Self {
            version: OBJECT_VERSION,
            owner: canonical(&owner.into()),
            entry,
            initializer: None,
            functions,
            constants,
            globals,
            records,
            enums,
            debug_types,
            sources,
            definitions: Vec::new(),
            imports: Vec::new(),
            relocations,
        };
        object.define_all_private()?;
        object.validate()?;
        Ok(object)
    }

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
        validation::validate_debug_types(
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
            validation::validate_debug_info(function, self.sources.len(), self.debug_types.len())?;
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

fn canonical(name: &str) -> String {
    name.to_ascii_lowercase()
}
