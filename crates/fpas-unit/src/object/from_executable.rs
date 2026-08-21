//! Convert a verified executable into a relocatable object.

use fpas_bytecode::VerifiedExecutable;

use super::conversion::{localize_branch, object_debug_type, relocation_for_instruction};
use super::{
    OBJECT_VERSION, ObjectConstant, ObjectDebugBinding, ObjectDebugBindingKind,
    ObjectDebugLocation, ObjectDebugScope, ObjectEnumLayout, ObjectEnumVariant, ObjectError,
    ObjectFunction, ObjectFunctionDebugInfo, ObjectGlobal, ObjectInitializer, ObjectRecordLayout,
    ObjectRecordMethod, ObjectRecordProperty, ObjectReturn, ObjectSequencePoint, ObjectSourceRun,
    RelocatableObject, Relocation, SymbolReference, canonical,
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
                    name: strings(global.name)?,
                    ty: global.ty.get(),
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
}
