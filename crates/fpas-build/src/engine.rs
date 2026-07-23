//! Dependency-first incremental build and final object linking.

use std::collections::HashMap;
use std::fmt;
use std::fs;

use fpas_bytecode::Chunk;
use fpas_parser::{Program, QualifiedId};
use fpas_project::{ResolvedUnitGraph, UnitGraph};
use fpas_unit::interface::{UnitInterface, decode_interface, encode_interface};
use fpas_unit::object::{RelocatableObject, decode_object, encode_object};
use fpas_unit::{
    CompiledUnit, DependencyIdentity, Digest, ExpectedUnitIdentity, SidecarLoad, UnitIdentity,
    load_sidecar, write_sidecar,
};

use crate::{BuildCounters, BuildEvent, BuildEventKind, BuildOptions};

/// Incremental build failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildError {
    detail: String,
}

impl BuildError {
    fn new(detail: impl Into<String>) -> Self {
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
}

impl BuiltUnits {
    /// Aggregate activity counts.
    #[must_use]
    pub fn counters(&self) -> BuildCounters {
        BuildCounters::from_events(&self.events)
    }
}

/// Linked executable program and incremental build activity.
pub struct BuiltProgram {
    /// Executable VM image.
    pub chunk: Chunk,
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
    let mut interfaces = HashMap::<String, UnitInterface>::new();
    let mut objects = Vec::with_capacity(selection.len());
    let mut events = Vec::new();

    for unit_name in selection.order() {
        let node = graph.get(unit_name).ok_or_else(|| {
            BuildError::new(format!(
                "internal build graph error: selected unit `{unit_name}` is missing"
            ))
        })?;
        let dependencies = direct_dependency_identities(node.direct_uses(), &interfaces)?;
        let direct_interfaces = direct_interfaces(node.direct_uses(), &interfaces);
        let source = fs::read(node.path()).map_err(|error| {
            BuildError::new(format!(
                "cannot read unit source `{}`: {error}",
                node.path().display()
            ))
        })?;
        let expected = ExpectedUnitIdentity {
            unit_name: unit_name.clone(),
            source_hash: Digest::of(&source),
            compiler_version: options.compiler_version.clone(),
            bytecode_version: options.bytecode_version,
            options_hash: options.options_hash,
            dependencies: dependencies.clone(),
        };

        let reusable = match load_sidecar(node.path(), &expected)
            .map_err(|error| BuildError::new(error.to_string()))?
        {
            SidecarLoad::Reusable(compiled) => decode_payloads(&compiled).ok(),
            SidecarLoad::Missing
            | SidecarLoad::Stale(_)
            | SidecarLoad::Incompatible(_)
            | SidecarLoad::Corrupt(_) => None,
        };
        let (interface, mut object) = if let Some(payloads) = reusable {
            events.push(event(unit_name, BuildEventKind::SidecarReused));
            payloads
        } else {
            events.push(event(unit_name, BuildEventKind::Parsed));
            let supporting_interfaces: Vec<_> = interfaces.values().cloned().collect();
            let compiled = fpas_compiler::compile_unit_object_with_support(
                node.parsed_unit().map_err(BuildError::new)?,
                &direct_interfaces,
                &supporting_interfaces,
            )
            .map_err(|diagnostics| {
                BuildError::new(format_diagnostics(node.path(), &diagnostics))
            })?;
            events.push(event(unit_name, BuildEventKind::InterfaceAnalyzed));
            events.push(event(unit_name, BuildEventKind::ImplementationAnalyzed));
            events.push(event(unit_name, BuildEventKind::Compiled));
            let interface_bytes = encode_interface(&compiled.interface)
                .map_err(|error| BuildError::new(error.to_string()))?;
            let object_bytes = encode_object(&compiled.object)
                .map_err(|error| BuildError::new(error.to_string()))?;
            let sidecar = CompiledUnit {
                identity: UnitIdentity {
                    unit_name: unit_name.clone(),
                    source_hash: expected.source_hash,
                    interface_hash: Digest::of(&interface_bytes),
                    object_hash: Digest::of(&object_bytes),
                    compiler_version: options.compiler_version.clone(),
                    bytecode_version: options.bytecode_version,
                    options_hash: options.options_hash,
                    dependencies,
                },
                interface: interface_bytes,
                object: object_bytes,
            };
            write_sidecar(node.path(), &sidecar).map_err(|error| {
                BuildError::new(format!(
                    "cannot publish compiled unit beside `{}`: {error}",
                    node.path().display()
                ))
            })?;
            (compiled.interface, compiled.object)
        };
        for location in &mut object.locations {
            location.source_id = node.source_id();
        }
        interfaces.insert(unit_name.clone(), interface);
        objects.push(object);
    }

    Ok(BuiltUnits {
        objects,
        interfaces,
        events,
    })
}

/// Build reachable units, compile the root program from interfaces, and link one [`Chunk`].
pub fn build_program(
    graph: &UnitGraph,
    selection: &ResolvedUnitGraph,
    program: &Program,
    options: &BuildOptions,
) -> Result<BuiltProgram, BuildError> {
    let mut units = build_library_units(graph, selection, options)?;
    let root_interfaces = direct_interfaces(&program.uses, &units.interfaces);
    let supporting_interfaces: Vec<_> = units.interfaces.values().cloned().collect();
    let program_object = fpas_compiler::compile_program_object_with_support(
        program,
        &root_interfaces,
        &supporting_interfaces,
    )
    .map_err(|diagnostics| BuildError::new(format_diagnostics_text(&diagnostics)))?;
    let chunk = fpas_linker::link_objects(&units.objects, &program_object)
        .map_err(|error| BuildError::new(error.to_string()))?;
    units
        .events
        .push(event(&program.name, BuildEventKind::Relinked));
    Ok(BuiltProgram {
        chunk,
        events: units.events,
    })
}

fn decode_payloads(compiled: &CompiledUnit) -> Result<(UnitInterface, RelocatableObject), ()> {
    let interface = decode_interface(&compiled.interface).map_err(|_| ())?;
    let object = decode_object(&compiled.object).map_err(|_| ())?;
    Ok((interface, object))
}

fn direct_dependency_identities(
    uses: &[QualifiedId],
    interfaces: &HashMap<String, UnitInterface>,
) -> Result<Vec<DependencyIdentity>, BuildError> {
    let mut dependencies = Vec::new();
    for used in uses {
        let name = used.parts.join(".").to_ascii_lowercase();
        let Some(interface) = interfaces.get(&name) else {
            continue;
        };
        dependencies.push(DependencyIdentity {
            unit_name: name,
            interface_hash: interface
                .digest()
                .map_err(|error| BuildError::new(error.to_string()))?,
        });
    }
    Ok(dependencies)
}

fn direct_interfaces(
    uses: &[QualifiedId],
    interfaces: &HashMap<String, UnitInterface>,
) -> Vec<UnitInterface> {
    uses.iter()
        .filter_map(|used| {
            interfaces
                .get(&used.parts.join(".").to_ascii_lowercase())
                .cloned()
        })
        .collect()
}

fn event(owner: &str, kind: BuildEventKind) -> BuildEvent {
    BuildEvent {
        owner: owner.to_string(),
        kind,
    }
}

fn format_diagnostics(
    path: &std::path::Path,
    diagnostics: &[fpas_compiler::CompileError],
) -> String {
    diagnostics
        .iter()
        .map(|diagnostic| {
            let mut text = format!(
                "{}:{}:{}: error[{}]: {}",
                path.display(),
                diagnostic.span.line,
                diagnostic.span.column,
                diagnostic.code,
                diagnostic.message
            );
            if let Some(help) = &diagnostic.help {
                text.push_str("\n  help: ");
                text.push_str(help);
            }
            text
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_diagnostics_text(diagnostics: &[fpas_compiler::CompileError]) -> String {
    diagnostics
        .iter()
        .map(|diagnostic| {
            let mut text = format!(
                "{}:{}: error[{}]: {}",
                diagnostic.span.line, diagnostic.span.column, diagnostic.code, diagnostic.message
            );
            if let Some(help) = &diagnostic.help {
                text.push_str("\n  help: ");
                text.push_str(help);
            }
            text
        })
        .collect::<Vec<_>>()
        .join("\n")
}
