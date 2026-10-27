//! Convert a verified executable into a relocatable object.
//! Documentation: `docs/pascal/program-structure/compiled-programs.md`.

use fpas_bytecode::VerifiedExecutable;
use std::collections::BTreeSet;

use super::conversion::{localize_branch, object_debug_type, relocation_for_instruction};
use super::{
    OBJECT_VERSION, ObjectConstant, ObjectDebugBinding, ObjectDebugBindingKind,
    ObjectDebugLocation, ObjectDebugScope, ObjectEnumLayout, ObjectEnumVariant, ObjectError,
    ObjectFunction, ObjectFunctionDebugInfo, ObjectGlobal, ObjectInitializer, ObjectRecordLayout,
    ObjectRecordMethod, ObjectReturn, ObjectSequencePoint, ObjectSourceRun, RelocatableObject,
    Relocation, SymbolReference, canonical,
};

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
                    projection: global
                        .projection
                        .as_ref()
                        .map(|node| super::projections::from_executable(node, &strings))
                        .transpose()?,
                    name: strings(global.name)?,
                    ty: global.ty.get(),
                    type_name: global.type_name.map(strings).transpose()?,
                    mutable: global.mutable,
                    initializer: global
                        .initializer
                        .map(|initializer| {
                            let function = executable
                                .functions
                                .get(usize::from(initializer.function.get()))
                                .ok_or(ObjectError::InvalidTableReference(
                                    "global initializer function",
                                ))?;
                            let instruction_start = initializer
                                .instruction
                                .get()
                                .checked_sub(function.code.start.get())
                                .ok_or(ObjectError::InvalidTableReference(
                                    "global initializer instruction",
                                ))?;
                            Ok(ObjectInitializer {
                                function: u32::from(initializer.function.get()),
                                instruction_start,
                            })
                        })
                        .transpose()?,
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
                    construction: record
                        .construction
                        .as_ref()
                        .map(|info| {
                            Ok(super::ObjectRecordConstructionInfo {
                                owner_unit: info.owner_unit.map(strings).transpose()?,
                                requires_owner: info.requires_owner,
                                aliases: info
                                    .aliases
                                    .iter()
                                    .map(|alias| {
                                        Ok(super::ObjectRecordTypeAlias {
                                            source: alias.source.get(),
                                            name: strings(alias.name)?,
                                            unit: strings(alias.unit)?,
                                        })
                                    })
                                    .collect::<Result<Vec<_>, ObjectError>>()?,
                                defaults: info
                                    .defaults
                                    .iter()
                                    .map(|name| name.map(strings).transpose())
                                    .collect::<Result<Vec<_>, _>>()?,
                            })
                        })
                        .transpose()?,
                    field_types: record.fields.iter().map(|field| field.ty.get()).collect(),
                    field_type_names: record
                        .fields
                        .iter()
                        .map(|field| field.type_name.map(strings).transpose())
                        .collect::<Result<Vec<_>, _>>()?,
                    methods: record
                        .methods
                        .iter()
                        .map(|method| {
                            Ok(ObjectRecordMethod {
                                name: strings(method.name)?,
                                routine: strings(method.routine)?,
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
                error: variant
                    .error
                    .as_ref()
                    .map(|error| {
                        Ok(super::ObjectErrorVariant {
                            field_types: error.field_types.clone(),
                            message: error.message.map(strings).transpose()?,
                            cause: error.cause.map(|cause| cause.field),
                        })
                    })
                    .transpose()?,
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
                    error: layout.error.clone(),
                    name: strings(layout.name)?,
                    variants,
                })
            })
            .collect::<Result<Vec<_>, ObjectError>>()?;

        let function_names = executable
            .functions
            .iter()
            .map(|function| strings(function.name))
            .collect::<Result<Vec<_>, _>>()?;
        let mut reserved_names = function_names
            .iter()
            .chain(globals.iter().map(|global| &global.name))
            .chain(records.iter().map(|record| &record.name))
            .chain(enums.iter().map(|enumeration| &enumeration.name))
            .map(|name| canonical(name))
            .collect::<BTreeSet<_>>();
        for key in enums
            .iter()
            .filter_map(|enumeration| enumeration.error.as_ref())
        {
            reserved_names
                .insert(crate::errors::instance_symbol(key).map_err(ObjectError::Encode)?);
        }
        let mut seen_names = BTreeSet::new();
        let mut functions = Vec::with_capacity(executable.functions.len());
        let mut relocations = Vec::new();
        for (function_index, function) in executable.functions.iter().enumerate() {
            let source_name = &function_names[function_index];
            let (name, display_name) = if seen_names.insert(canonical(source_name)) {
                (source_name.clone(), None)
            } else {
                // Concrete factories may share a defining display name. Retain the
                // first symbol for name-based consumers and distinguish later locals.
                let base = format!("{source_name}.$object.{function_index}");
                let mut name = base.clone();
                let mut suffix = 0;
                while !reserved_names.insert(canonical(&name)) {
                    suffix += 1;
                    name = format!("{base}.{suffix}");
                }
                (name, Some(source_name.clone()))
            };
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
                            projection: binding
                                .projection
                                .as_ref()
                                .map(|node| super::projections::from_executable(node, &strings))
                                .transpose()?,
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
                            initializer_start: binding
                                .initializer
                                .map(|initializer| {
                                    initializer
                                        .get()
                                        .checked_sub(function.code.start.get())
                                        .ok_or(ObjectError::InvalidTableReference(
                                            "debug binding initializer instruction",
                                        ))
                                })
                                .transpose()?,
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
                lexical_owner: function
                    .debug
                    .lexical_owner
                    .map(|owner| u32::from(owner.get())),
                capture_sources: function
                    .debug
                    .capture_sources
                    .iter()
                    .map(|source| crate::object::ObjectCaptureSource {
                        binding: source.binding.get(),
                        ty: source.ty.get(),
                        kind: match source.kind {
                            fpas_bytecode::DebugCaptureKind::Value => {
                                crate::object::ObjectCaptureKind::Value
                            }
                            fpas_bytecode::DebugCaptureKind::Cell => {
                                crate::object::ObjectCaptureKind::Cell
                            }
                            fpas_bytecode::DebugCaptureKind::EnclosingCell => {
                                crate::object::ObjectCaptureKind::EnclosingCell
                            }
                        },
                    })
                    .collect(),
            };
            functions.push(ObjectFunction {
                display_name,
                error_constructors: None,
                name,
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
        let distinct_types = executable
            .distinct_types
            .iter()
            .map(|distinct| {
                Ok(super::ObjectDistinctTypeName {
                    source: distinct.source.get(),
                    name: strings(distinct.name)?,
                    underlying: distinct.underlying.get(),
                })
            })
            .collect::<Result<Vec<_>, ObjectError>>()?;
        let mut object = Self {
            equality_declarations: Vec::new(),
            error_schemas: Vec::new(),
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
            distinct_types,
            sources,
            definitions: Vec::new(),
            imports: Vec::new(),
            relocations,
        };
        object.define_all_private()?;
        object.validate()?;
        Ok(object)
    }
}
