//! Dependency-first incremental build and final object linking.

mod backend;
mod interfaces;

use std::collections::HashMap;
use std::fmt;

use fpas_bytecode::VerifiedExecutable;
use fpas_parser::Program;
use fpas_program::LinkedUnitIdentity;
use fpas_project::{ResolvedUnitGraph, UnitGraph};
use fpas_unit::interface::{UnitInterface, encode_interface};
use fpas_unit::object::RelocatableObject;
use fpas_unit::{CompiledUnit, Digest, ExpectedUnitIdentity, UnitIdentity, write_sidecar};

use self::backend::{RegisterBackend, UnitBackend};
use self::interfaces::{InterfaceRegistry, direct_interfaces_from_map};
use crate::source_snapshot::UnitSourceSnapshot;
use crate::{BuildCounters, BuildEvent, BuildEventKind, BuildOptions};

/// Incremental build failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildError {
    detail: String,
}

impl BuildError {
    pub(crate) fn new(detail: impl Into<String>) -> Self {
        Self {
            detail: detail.into(),
        }
    }
}

impl fmt::Display for BuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.detail)
    }
}

impl std::error::Error for BuildError {}

/// Dependency-first compiled units and their build activity.
pub struct BuiltUnits {
    /// Objects in deterministic dependency order.
    pub objects: Vec<RelocatableObject>,
    /// Interfaces indexed by canonical unit name.
    pub interfaces: HashMap<String, UnitInterface>,
    /// Structured activity stream.
    pub events: Vec<BuildEvent>,
    pub(crate) linked_units: Vec<LinkedUnitIdentity>,
    supporting_interfaces: Vec<UnitInterface>,
}

impl BuiltUnits {
    /// Aggregate activity counts.
    #[must_use]
    pub fn counters(&self) -> BuildCounters {
        BuildCounters::from_events(&self.events)
    }

    fn supporting_interfaces(&self) -> &[UnitInterface] {
        &self.supporting_interfaces
    }
}

/// Linked executable program and incremental build activity.
pub struct BuiltProgram {
    /// Fully verified register executable.
    pub executable: VerifiedExecutable,
    /// Structured unit and link activity.
    pub events: Vec<BuildEvent>,
}

impl BuiltProgram {
    /// Aggregate activity counts.
    #[must_use]
    pub fn counters(&self) -> BuildCounters {
        BuildCounters::from_events(&self.events)
    }
}

/// Build or reuse every unit in a library selection.
pub fn build_library_units(
    graph: &UnitGraph,
    selection: &ResolvedUnitGraph,
    options: &BuildOptions,
) -> Result<BuiltUnits, BuildError> {
    compile_library_units(graph, selection, options, SidecarPublication::Enabled)
}

/// Compile every selected unit without publishing source-adjacent sidecars.
///
/// Existing compatible sidecars may still be reused. Newly compiled units remain in memory,
/// making this path suitable for read-only validation commands.
///
/// # Errors
///
/// Returns [`BuildError`] when a selected unit cannot be read, compiled, or decoded.
pub fn check_library_units(
    graph: &UnitGraph,
    selection: &ResolvedUnitGraph,
    options: &BuildOptions,
) -> Result<BuiltUnits, BuildError> {
    compile_library_units(graph, selection, options, SidecarPublication::Disabled)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SidecarPublication {
    Enabled,
    Disabled,
}

fn compile_library_units(
    graph: &UnitGraph,
    selection: &ResolvedUnitGraph,
    options: &BuildOptions,
    sidecar_publication: SidecarPublication,
) -> Result<BuiltUnits, BuildError> {
    let units = compile_units::<RegisterBackend>(graph, selection, options, sidecar_publication)?;
    Ok(BuiltUnits {
        objects: units.objects,
        interfaces: units.interfaces,
        events: units.events,
        linked_units: units.linked_units,
        supporting_interfaces: units.supporting_interfaces,
    })
}

pub(super) struct CompiledUnits<Object> {
    pub(super) objects: Vec<Object>,
    pub(super) interfaces: HashMap<String, UnitInterface>,
    pub(super) events: Vec<BuildEvent>,
    pub(super) linked_units: Vec<LinkedUnitIdentity>,
    pub(super) supporting_interfaces: Vec<UnitInterface>,
}

fn compile_units<Backend: UnitBackend>(
    graph: &UnitGraph,
    selection: &ResolvedUnitGraph,
    options: &BuildOptions,
    sidecar_publication: SidecarPublication,
) -> Result<CompiledUnits<Backend::Object>, BuildError> {
    let mut interfaces = InterfaceRegistry::default();
    let mut objects = Vec::with_capacity(selection.len());
    let mut linked_units = Vec::with_capacity(selection.len());
    let mut events = Vec::new();

    for unit_name in selection.order() {
        let node = graph.get(unit_name).ok_or_else(|| {
            BuildError::new(format!(
                "internal build graph error: selected unit `{unit_name}` is missing"
            ))
        })?;
        let dependencies = interfaces.direct_dependency_identities(node.direct_uses());
        let direct_interfaces = interfaces.direct_interfaces(node.direct_uses());
        let source = UnitSourceSnapshot::read(node)?;
        let expected = ExpectedUnitIdentity {
            unit_name: unit_name.clone(),
            source_hash: source.hash(),
            compiler_version: options.compiler_version.clone(),
            bytecode_version: options.bytecode_version,
            options_hash: options.options_hash,
            dependencies: dependencies.clone(),
        };

        let reusable = Backend::load(node.path(), &expected)?;
        let (interface, mut object, interface_hash, object_hash) = if let Some(reusable) = reusable
        {
            events.push(event(unit_name, BuildEventKind::SidecarReused));
            (
                reusable.interface,
                reusable.object,
                reusable.interface_hash,
                reusable.object_hash,
            )
        } else {
            events.push(event(unit_name, BuildEventKind::Parsed));
            let parsed = node
                .parse_source_snapshot(source.bytes())
                .map_err(BuildError::new)?;
            let (interface, object) =
                Backend::compile(&parsed, &direct_interfaces, interfaces.all()).map_err(
                    |diagnostics| {
                        BuildError::new(format_diagnostics(Some(node.path()), &diagnostics))
                    },
                )?;
            events.push(event(unit_name, BuildEventKind::InterfaceAnalyzed));
            events.push(event(unit_name, BuildEventKind::ImplementationAnalyzed));
            events.push(event(unit_name, BuildEventKind::Compiled));
            let interface_bytes =
                encode_interface(&interface).map_err(|error| BuildError::new(error.to_string()))?;
            let interface_hash = Digest::of(&interface_bytes);
            let object_bytes = Backend::encode(&object)?;
            let object_hash = Digest::of(&object_bytes);
            let sidecar = CompiledUnit {
                identity: UnitIdentity {
                    unit_name: unit_name.clone(),
                    source_hash: expected.source_hash,
                    interface_hash,
                    object_hash,
                    compiler_version: options.compiler_version.clone(),
                    bytecode_version: options.bytecode_version,
                    options_hash: options.options_hash,
                    dependencies,
                },
                interface: interface_bytes,
                object: object_bytes,
            };
            if sidecar_publication == SidecarPublication::Enabled {
                source.ensure_current(node)?;
                write_sidecar(node.path(), &sidecar).map_err(|error| {
                    BuildError::new(format!(
                        "cannot publish compiled unit beside `{}`: {error}",
                        node.path().display()
                    ))
                })?;
            }
            (interface, object, interface_hash, object_hash)
        };
        Backend::normalize(&mut object, node.source_id());
        interfaces.insert(unit_name.clone(), interface, interface_hash);
        linked_units.push(LinkedUnitIdentity {
            unit_name: unit_name.clone(),
            object_hash: fpas_program::Digest::from_bytes(*object_hash.as_bytes()),
        });
        objects.push(object);
    }

    Ok(interfaces.finish(objects, linked_units, events))
}

/// Build reachable units, compile the root program, and link one verified register executable.
pub fn build_program(
    graph: &UnitGraph,
    selection: &ResolvedUnitGraph,
    program: &Program,
    options: &BuildOptions,
) -> Result<BuiltProgram, BuildError> {
    let units = build_library_units(graph, selection, options)?;
    link_program(units, program)
}

/// Compile and link a program without publishing newly compiled unit sidecars.
///
/// This validates the complete program graph while leaving its source tree unchanged.
///
/// # Errors
///
/// Returns [`BuildError`] when a selected unit or the root program cannot be compiled or linked.
pub fn check_program(
    graph: &UnitGraph,
    selection: &ResolvedUnitGraph,
    program: &Program,
    options: &BuildOptions,
) -> Result<BuiltProgram, BuildError> {
    let units = check_library_units(graph, selection, options)?;
    link_program(units, program)
}

pub(crate) fn link_program(
    mut units: BuiltUnits,
    program: &Program,
) -> Result<BuiltProgram, BuildError> {
    let root_interfaces = direct_interfaces_from_map(&program.uses, &units.interfaces);
    let mut program_object = fpas_compiler::compile_register_program_object_with_support(
        program,
        &root_interfaces,
        units.supporting_interfaces(),
    )
    .map_err(|diagnostics| BuildError::new(format_diagnostics(None, &diagnostics)))?;
    normalize_sources(&mut program_object, 0);
    let executable = fpas_linker::link_register_objects(&units.objects, &program_object)
        .map_err(|error| BuildError::new(error.to_string()))?;
    units
        .events
        .push(event(&program.name, BuildEventKind::Relinked));
    Ok(BuiltProgram {
        executable,
        events: units.events,
    })
}

fn normalize_sources(object: &mut RelocatableObject, source_id: u32) {
    let source = format!("source-{source_id}.fpas");
    object.sources.fill(source);
}

fn event(owner: &str, kind: BuildEventKind) -> BuildEvent {
    BuildEvent {
        owner: owner.to_string(),
        kind,
    }
}

fn format_diagnostics(
    path: Option<&std::path::Path>,
    diagnostics: &[fpas_compiler::CompileError],
) -> String {
    diagnostics
        .iter()
        .map(|diagnostic| format_diagnostic(path, diagnostic))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_diagnostic(
    path: Option<&std::path::Path>,
    diagnostic: &fpas_compiler::CompileError,
) -> String {
    if let Some(path) = path {
        fpas_diagnostics::render(path.to_string_lossy().as_ref(), diagnostic)
    } else {
        fpas_diagnostics::render_without_path(diagnostic)
    }
}
