//! Register-object compilation and symbolic import extraction.

mod imports;

use std::collections::{BTreeMap, BTreeSet};

use fpas_parser::{Program, Unit};
use fpas_unit::interface::{SymbolKind as InterfaceSymbolKind, UnitInterface};
use fpas_unit::object::{
    DefinitionTarget, ObjectConstant, RelocatableObject, RelocationKind, SymbolReference,
};

use crate::error::{CompileError, internal_compiler_error};
use crate::lowering::ImportPlan;

use self::imports::{
    imported_index, imported_layouts, planned_layouts, prune_unreferenced_layouts, retained_map,
};

/// Public interface and relocatable register implementation for one source unit.
pub struct CompiledRegisterUnitObject {
    /// Stable public semantic interface.
    pub interface: UnitInterface,
    /// Relocatable register implementation and initializer.
    pub object: RelocatableObject,
}

/// Compile one source unit to a relocatable register object.
///
/// # Errors
///
/// Returns semantic, lowering, bytecode, or object-construction diagnostics.
pub fn compile_register_unit_object(
    unit: &Unit,
    interfaces: &[UnitInterface],
) -> Result<CompiledRegisterUnitObject, Vec<CompileError>> {
    compile_register_unit_object_with_support(unit, interfaces, interfaces)
}

/// Compile one source unit with direct imports and transitive type support.
///
/// # Errors
///
/// Returns semantic, lowering, bytecode, or object-construction diagnostics.
pub fn compile_register_unit_object_with_support(
    unit: &Unit,
    interfaces: &[UnitInterface],
    supporting_interfaces: &[UnitInterface],
) -> Result<CompiledRegisterUnitObject, Vec<CompileError>> {
    let analysis =
        fpas_sema::analyze_unit_with_interface_support(unit, interfaces, supporting_interfaces)
            .map_err(|error| {
                vec![internal_compiler_error(
                    error.to_string(),
                    "Rebuild the dependency sidecar; its semantic interface is invalid.",
                    unit.span.line,
                    unit.span.column,
                )]
            })?;
    if !analysis.metadata.errors.is_empty() {
        return Err(analysis.metadata.errors);
    }
    let Some(interface) = analysis.interface else {
        return Err(vec![internal_compiler_error(
            "Semantic analysis succeeded without producing a unit interface.",
            "This is an internal compiler error. Re-run compilation and report the source unit.",
            unit.span.line,
            unit.span.column,
        )]);
    };
    let lowered =
        crate::lowering::lower_register_unit_subset(unit, interfaces, supporting_interfaces)?;
    let executable =
        crate::bytecode::compile_program(&lowered.program).map_err(|error| vec![error])?;
    let owner = unit.name.parts.join(".").to_ascii_lowercase();
    let mut object = RelocatableObject::from_executable(&owner, executable)
        .map_err(|error| object_error(unit.span, error))?;
    object.entry = None;
    object.initializer = Some(0);
    apply_imports(&mut object, lowered.imports).map_err(|error| object_error(unit.span, error))?;
    let owned_layouts = owned_layouts(unit, &owner);
    let retained_layouts = owned_layouts
        .iter()
        .flat_map(|(short, qualified)| [short.clone(), qualified.clone()])
        .collect();
    prune_unreferenced_layouts(&mut object, &retained_layouts)
        .map_err(|error| object_error(unit.span, error))?;
    qualify_unit_definitions(&mut object, &owner, &interface, &owned_layouts)
        .map_err(|error| object_error(unit.span, error))?;
    object
        .validate()
        .map_err(|error| object_error(unit.span, error))?;
    Ok(CompiledRegisterUnitObject { interface, object })
}

/// Compile a root program against dependency interfaces into a register object.
///
/// # Errors
///
/// Returns semantic, lowering, bytecode, or object-construction diagnostics.
pub fn compile_register_program_object_with_support(
    program: &Program,
    interfaces: &[UnitInterface],
    supporting_interfaces: &[UnitInterface],
) -> Result<RelocatableObject, Vec<CompileError>> {
    let lowered = crate::lowering::lower_register_program_with_support(
        program,
        interfaces,
        supporting_interfaces,
    )?;
    let executable =
        crate::bytecode::compile_program(&lowered.program).map_err(|error| vec![error])?;
    let mut object = RelocatableObject::from_executable(&program.name, executable)
        .map_err(|error| object_error(program.span, error))?;
    apply_imports(&mut object, lowered.imports)
        .map_err(|error| object_error(program.span, error))?;
    prune_unreferenced_layouts(&mut object, &BTreeSet::new())
        .map_err(|error| object_error(program.span, error))?;
    object
        .validate()
        .map_err(|error| object_error(program.span, error))?;
    Ok(object)
}

fn apply_imports(
    object: &mut RelocatableObject,
    plan: ImportPlan,
) -> Result<(), fpas_unit::object::ObjectError> {
    let planned_functions = plan
        .functions
        .iter()
        .map(|(id, _)| id.get())
        .collect::<BTreeSet<_>>();
    let planned_globals = plan
        .globals
        .iter()
        .map(|(id, _)| id.get())
        .collect::<BTreeSet<_>>();
    let planned_records = planned_layouts(
        object.records.iter().map(|record| &record.name),
        &plan.layouts,
        |shape| matches!(shape, fpas_unit::object::ImportShape::Record { .. }),
    )?;
    let planned_enums = planned_layouts(
        object.enums.iter().map(|enumeration| &enumeration.name),
        &plan.layouts,
        |shape| matches!(shape, fpas_unit::object::ImportShape::Enum { .. }),
    )?;
    let referenced_records = object
        .relocations
        .iter()
        .filter_map(|relocation| match relocation.kind {
            RelocationKind::Record(SymbolReference::Local(index)) => Some(index),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let referenced_enums = object
        .relocations
        .iter()
        .filter_map(|relocation| match relocation.kind {
            RelocationKind::EnumVariant {
                enumeration: SymbolReference::Local(index),
                ..
            } => Some(index),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let referenced_functions = object
        .relocations
        .iter()
        .filter_map(|relocation| match relocation.kind {
            RelocationKind::Function(SymbolReference::Local(index)) => Some(index),
            _ => None,
        })
        .chain(
            object
                .constants
                .iter()
                .filter_map(|constant| match constant {
                    ObjectConstant::Function {
                        function: SymbolReference::Local(index),
                        ..
                    } => Some(*index),
                    _ => None,
                }),
        )
        .collect::<BTreeSet<_>>();
    let referenced_globals = object
        .relocations
        .iter()
        .filter_map(|relocation| match relocation.kind {
            RelocationKind::Global(SymbolReference::Local(index)) => Some(index),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let mut required_import_names = plan
        .functions
        .iter()
        .filter(|(id, _)| referenced_functions.contains(&id.get()))
        .map(|(_, import)| import.name.clone())
        .chain(
            plan.globals
                .iter()
                .filter(|(id, _)| referenced_globals.contains(&id.get()))
                .map(|(_, import)| import.name.clone()),
        )
        .collect::<BTreeSet<_>>();
    for (index, record) in object.records.iter().enumerate() {
        let index = u32::try_from(index)
            .map_err(|_| fpas_unit::object::ObjectError::Overflow("local record index"))?;
        if planned_records.contains(&index) && referenced_records.contains(&index) {
            required_import_names.insert(record.name.clone());
        }
    }
    for (index, enumeration) in object.enums.iter().enumerate() {
        let index = u32::try_from(index)
            .map_err(|_| fpas_unit::object::ObjectError::Overflow("local enum index"))?;
        if planned_enums.contains(&index) && referenced_enums.contains(&index) {
            required_import_names.insert(enumeration.name.clone());
        }
    }
    let mut imports = plan
        .functions
        .iter()
        .map(|(_, import)| import)
        .chain(plan.globals.iter().map(|(_, import)| import))
        .chain(plan.layouts.iter())
        .filter(|import| required_import_names.contains(&import.name))
        .cloned()
        .collect::<Vec<_>>();
    imports.sort_by(|left, right| left.name.cmp(&right.name));
    imports.dedup_by(|left, right| left.name == right.name);
    let import_indices = imports
        .iter()
        .enumerate()
        .map(|(index, import)| (import.name.clone(), u32::try_from(index)))
        .map(|(name, index)| {
            index
                .map(|index| (name, index))
                .map_err(|_| fpas_unit::object::ObjectError::Overflow("import index"))
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let imported_functions = plan
        .functions
        .iter()
        .filter(|(id, _)| referenced_functions.contains(&id.get()))
        .map(|(id, import)| imported_index(id.get(), import, &import_indices))
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let imported_globals = plan
        .globals
        .iter()
        .filter(|(id, _)| referenced_globals.contains(&id.get()))
        .map(|(id, import)| imported_index(id.get(), import, &import_indices))
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let imported_records = imported_layouts(
        &planned_records,
        &referenced_records,
        object.records.iter().map(|record| &record.name),
        &import_indices,
    )?;
    let imported_enums = imported_layouts(
        &planned_enums,
        &referenced_enums,
        object.enums.iter().map(|enumeration| &enumeration.name),
        &import_indices,
    )?;
    let function_map = retained_map(object.functions.len(), planned_functions.iter().copied())?;
    let global_map = retained_map(object.globals.len(), planned_globals.iter().copied())?;
    let record_map = retained_map(object.records.len(), planned_records.iter().copied())?;
    let enum_map = retained_map(object.enums.len(), planned_enums.iter().copied())?;

    object.functions = object
        .functions
        .iter()
        .enumerate()
        .filter(|(index, _)| {
            !planned_functions.contains(&u32::try_from(*index).unwrap_or(u32::MAX))
        })
        .map(|(_, function)| function.clone())
        .collect();
    object.globals = object
        .globals
        .iter()
        .enumerate()
        .filter(|(index, _)| !planned_globals.contains(&u32::try_from(*index).unwrap_or(u32::MAX)))
        .map(|(_, global)| global.clone())
        .collect();
    object.records = object
        .records
        .iter()
        .enumerate()
        .filter(|(index, _)| !planned_records.contains(&u32::try_from(*index).unwrap_or(u32::MAX)))
        .map(|(_, record)| record.clone())
        .collect();
    object.enums = object
        .enums
        .iter()
        .enumerate()
        .filter(|(index, _)| !planned_enums.contains(&u32::try_from(*index).unwrap_or(u32::MAX)))
        .map(|(_, enumeration)| enumeration.clone())
        .collect();
    object
        .definitions
        .retain_mut(|definition| match definition.target {
            DefinitionTarget::Function(index) => function_map[index as usize]
                .map(|mapped| definition.target = DefinitionTarget::Function(mapped))
                .is_some(),
            DefinitionTarget::Global(index) => global_map[index as usize]
                .map(|mapped| definition.target = DefinitionTarget::Global(mapped))
                .is_some(),
            DefinitionTarget::Record(index) => record_map[index as usize]
                .map(|mapped| definition.target = DefinitionTarget::Record(mapped))
                .is_some(),
            DefinitionTarget::Enum(index) => enum_map[index as usize]
                .map(|mapped| definition.target = DefinitionTarget::Enum(mapped))
                .is_some(),
        });
    object.relocations.retain_mut(|relocation| {
        let Some(mapped_owner) = function_map[relocation.function as usize] else {
            return false;
        };
        relocation.function = mapped_owner;
        match &mut relocation.kind {
            RelocationKind::Function(SymbolReference::Local(index)) => {
                if let Some(import) = imported_functions.get(index) {
                    relocation.kind = RelocationKind::Function(SymbolReference::Import(*import));
                } else if let Some(mapped) = function_map[*index as usize] {
                    *index = mapped;
                }
            }
            RelocationKind::Global(SymbolReference::Local(index)) => {
                if let Some(import) = imported_globals.get(index) {
                    relocation.kind = RelocationKind::Global(SymbolReference::Import(*import));
                } else if let Some(mapped) = global_map[*index as usize] {
                    *index = mapped;
                }
            }
            RelocationKind::Record(SymbolReference::Local(index)) => {
                if let Some(import) = imported_records.get(index) {
                    relocation.kind = RelocationKind::Record(SymbolReference::Import(*import));
                } else if let Some(mapped) = record_map[*index as usize] {
                    *index = mapped;
                }
            }
            RelocationKind::EnumVariant { enumeration, .. } => {
                if let SymbolReference::Local(index) = enumeration {
                    if let Some(import) = imported_enums.get(index) {
                        *enumeration = SymbolReference::Import(*import);
                    } else if let Some(mapped) = enum_map[*index as usize] {
                        *index = mapped;
                    }
                }
            }
            RelocationKind::Constant(_)
            | RelocationKind::Function(SymbolReference::Import(_))
            | RelocationKind::Global(SymbolReference::Import(_))
            | RelocationKind::Record(SymbolReference::Import(_))
            | RelocationKind::RecordField(_)
            | RelocationKind::EnumField(_)
            | RelocationKind::CodeAddress(_) => {}
        }
        true
    });
    for constant in &mut object.constants {
        if let ObjectConstant::Function {
            function: SymbolReference::Local(index),
            ..
        } = constant
        {
            if let Some(import) = imported_functions.get(index) {
                let task_bound = match constant {
                    ObjectConstant::Function { task_bound, .. } => *task_bound,
                    _ => {
                        return Err(fpas_unit::object::ObjectError::InvalidTableReference(
                            "imported function constant",
                        ));
                    }
                };
                *constant = ObjectConstant::Function {
                    function: SymbolReference::Import(*import),
                    task_bound,
                };
            } else if let Some(mapped) = function_map[*index as usize] {
                *index = mapped;
            }
        }
    }
    object.entry = object.entry.and_then(|index| function_map[index as usize]);
    object.initializer = object
        .initializer
        .and_then(|index| function_map[index as usize]);
    object.imports = imports;
    object
        .definitions
        .sort_by(|left, right| left.name.cmp(&right.name));
    object
        .relocations
        .sort_by_key(|relocation| (relocation.function, relocation.instruction));
    Ok(())
}

fn qualify_unit_definitions(
    object: &mut RelocatableObject,
    owner: &str,
    interface: &UnitInterface,
    owned_layouts: &BTreeMap<String, String>,
) -> Result<(), fpas_unit::object::ObjectError> {
    for (index, function) in object.functions.iter_mut().enumerate() {
        if object.initializer == u32::try_from(index).ok() {
            function.name = owner.to_string();
        } else if !function.name.starts_with(&format!("{owner}.")) {
            function.name = format!("{owner}.{}", function.name);
        }
    }
    for global in &mut object.globals {
        if !global.name.starts_with(&format!("{owner}.")) {
            global.name = format!("{owner}.{}", global.name);
        }
    }
    for record in &mut object.records {
        if let Some(qualified) = owned_layouts.get(&record.name.to_ascii_lowercase()) {
            record.name = qualified.clone();
        } else if !record.name.contains('.') {
            record.name = format!("{owner}.{}", record.name);
        }
    }
    for enumeration in &mut object.enums {
        if let Some(qualified) = owned_layouts.get(&enumeration.name.to_ascii_lowercase()) {
            enumeration.name = qualified.clone();
        } else if !enumeration.name.contains('.') {
            enumeration.name = format!("{owner}.{}", enumeration.name);
        }
    }
    object.define_all_private()?;
    let mut public = BTreeSet::new();
    for symbol in &interface.symbols {
        if matches!(
            symbol.kind,
            InterfaceSymbolKind::Function
                | InterfaceSymbolKind::Procedure
                | InterfaceSymbolKind::Variable
                | InterfaceSymbolKind::MutableVariable
                | InterfaceSymbolKind::Type
        ) {
            public.insert(symbol.qualified_name.to_ascii_lowercase());
        }
        let fpas_unit::interface::InterfaceType::Record(record) = &symbol.ty else {
            continue;
        };
        for method in record.methods.iter().chain(&record.static_routines) {
            if record
                .private_members
                .iter()
                .any(|private| private.eq_ignore_ascii_case(&method.name))
            {
                continue;
            }
            public.insert(format!("{}.{}", record.name, method.name).to_ascii_lowercase());
        }
    }
    for definition in &mut object.definitions {
        definition.public = public.contains(&definition.name);
    }
    Ok(())
}

fn owned_layouts(unit: &Unit, owner: &str) -> BTreeMap<String, String> {
    unit.declarations
        .iter()
        .filter_map(|declaration| match declaration {
            fpas_parser::Decl::TypeDef(definition)
                if matches!(
                    definition.body,
                    fpas_parser::TypeBody::Record(_) | fpas_parser::TypeBody::Enum(_)
                ) =>
            {
                Some((
                    definition.name.to_ascii_lowercase(),
                    format!("{owner}.{}", definition.name).to_ascii_lowercase(),
                ))
            }
            _ => None,
        })
        .collect()
}

fn object_error(
    span: fpas_lexer::Span,
    error: fpas_unit::object::ObjectError,
) -> Vec<CompileError> {
    vec![internal_compiler_error(
        format!("Register object construction failed: {error}."),
        "This is an internal compiler error. Re-run compilation and report the source unit.",
        span.line,
        span.column,
    )]
}
