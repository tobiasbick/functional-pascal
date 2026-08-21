//! Relocatable-object compilation and symbolic import extraction.

mod imports;

use std::collections::{BTreeMap, BTreeSet};

use fpas_parser::{Program, Unit};
use fpas_unit::interface::{SymbolKind as InterfaceSymbolKind, UnitInterface};
use fpas_unit::object::RelocatableObject;

use crate::error::{CompileError, internal_compiler_error};

use self::imports::{apply_imports, prune_unreferenced_layouts};

/// Public interface and relocatable implementation for one source unit.
pub struct CompiledUnitObject {
    /// Stable public semantic interface.
    pub interface: UnitInterface,
    /// Relocatable implementation and initializer.
    pub object: RelocatableObject,
}

/// Compile one source unit to a relocatable object.
///
/// # Errors
///
/// Returns semantic, lowering, bytecode, or object-construction diagnostics.
pub fn compile_unit_object(
    unit: &Unit,
    interfaces: &[UnitInterface],
) -> Result<CompiledUnitObject, Vec<CompileError>> {
    compile_unit_object_with_support(unit, interfaces, interfaces)
}

/// Compile one source unit with direct imports and transitive type support.
///
/// # Errors
///
/// Returns semantic, lowering, bytecode, or object-construction diagnostics.
pub fn compile_unit_object_with_support(
    unit: &Unit,
    interfaces: &[UnitInterface],
    supporting_interfaces: &[UnitInterface],
) -> Result<CompiledUnitObject, Vec<CompileError>> {
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
    let lowered = crate::lowering::lower_unit(unit, interfaces, supporting_interfaces)?;
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
        .chain(runtime_aggregate_types())
        .collect();
    prune_unreferenced_layouts(&mut object, &retained_layouts)
        .map_err(|error| object_error(unit.span, error))?;
    qualify_unit_definitions(&mut object, &owner, &interface, &owned_layouts)
        .map_err(|error| object_error(unit.span, error))?;
    object
        .validate()
        .map_err(|error| object_error(unit.span, error))?;
    Ok(CompiledUnitObject { interface, object })
}

/// Compile a root program against dependency interfaces into a relocatable object.
///
/// # Errors
///
/// Returns semantic, lowering, bytecode, or object-construction diagnostics.
pub fn compile_program_object_with_support(
    program: &Program,
    interfaces: &[UnitInterface],
    supporting_interfaces: &[UnitInterface],
) -> Result<RelocatableObject, Vec<CompileError>> {
    let lowered =
        crate::lowering::lower_program_with_support(program, interfaces, supporting_interfaces)?;
    let executable =
        crate::bytecode::compile_program(&lowered.program).map_err(|error| vec![error])?;
    let mut object = RelocatableObject::from_executable(&program.name, executable)
        .map_err(|error| object_error(program.span, error))?;
    apply_imports(&mut object, lowered.imports)
        .map_err(|error| object_error(program.span, error))?;
    prune_unreferenced_layouts(&mut object, &runtime_aggregate_types().collect())
        .map_err(|error| object_error(program.span, error))?;
    object
        .validate()
        .map_err(|error| object_error(program.span, error))?;
    Ok(object)
}

fn runtime_aggregate_types() -> impl Iterator<Item = String> {
    fpas_std::RUNTIME_AGGREGATE_TYPES
        .iter()
        .map(|name| name.to_ascii_lowercase())
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
