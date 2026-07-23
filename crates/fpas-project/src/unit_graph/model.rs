//! Unit graph data shared by source linking and compiled-unit builds.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use fpas_parser::{QualifiedId, Unit};

use crate::common::qualified_id_to_string;
use crate::model::{ProjectLinkMeta, SourceOrigin};

/// One source unit and its project ownership metadata, parsed only when required.
#[derive(Debug, Clone)]
pub struct UnitNode {
    canonical_name: String,
    display_name: String,
    path: PathBuf,
    origin: SourceOrigin,
    source_id: u32,
    direct_uses: Vec<QualifiedId>,
    unit: Arc<OnceLock<Result<Unit, String>>>,
}

impl UnitNode {
    pub(super) fn new(path: PathBuf, origin: SourceOrigin, unit: Unit) -> Self {
        let display_name = qualified_id_to_string(&unit.name);
        Self {
            canonical_name: display_name.to_ascii_lowercase(),
            display_name,
            path,
            origin,
            source_id: unit.span.source_id,
            direct_uses: unit.uses.clone(),
            unit: Arc::new(OnceLock::from(Ok(unit))),
        }
    }

    pub(super) fn from_sidecar(
        path: PathBuf,
        origin: SourceOrigin,
        display_name: String,
        direct_uses: Vec<QualifiedId>,
        source_id: u32,
    ) -> Self {
        Self {
            canonical_name: display_name.to_ascii_lowercase(),
            display_name,
            path,
            origin,
            source_id,
            direct_uses,
            unit: Arc::new(OnceLock::new()),
        }
    }

    /// Case-normalized unit name used as the graph key.
    #[must_use]
    pub fn canonical_name(&self) -> &str {
        &self.canonical_name
    }

    /// Unit name with the spelling used by the source declaration.
    #[must_use]
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    /// Source file that declares this unit.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Project that contributed this unit.
    #[must_use]
    pub fn origin(&self) -> &SourceOrigin {
        &self.origin
    }

    /// Owning library manifest, or `None` for a unit owned by the root project.
    #[must_use]
    pub fn owning_library_project(&self) -> Option<&Path> {
        match &self.origin {
            SourceOrigin::Own => None,
            SourceOrigin::Library(path) => Some(path),
        }
    }

    /// Stable diagnostic source ID assigned while this graph was built.
    #[must_use]
    pub fn source_id(&self) -> u32 {
        self.source_id
    }

    /// Direct unit dependencies from this unit's `uses` clause.
    #[must_use]
    pub fn direct_uses(&self) -> &[QualifiedId] {
        &self.direct_uses
    }

    /// Parsed unit retained for semantic analysis or the transitional source linker.
    pub fn parsed_unit(&self) -> Result<&Unit, String> {
        self.unit
            .get_or_init(|| {
                let (parsed, _) =
                    crate::common::parse_compilation_unit_file(&self.path, self.source_id)?;
                let fpas_parser::CompilationUnit::Unit(unit) = parsed else {
                    return Err(format!(
                        "Source file `{}` no longer declares a unit.",
                        self.path.display()
                    ));
                };
                if !unit
                    .name
                    .parts
                    .join(".")
                    .eq_ignore_ascii_case(&self.display_name)
                {
                    return Err(format!(
                        "Source file `{}` declares unit `{}`, but its compiled metadata names `{}`.",
                        self.path.display(),
                        unit.name.parts.join("."),
                        self.display_name
                    ));
                }
                Ok(unit)
            })
            .as_ref()
            .map_err(Clone::clone)
    }

    /// Returns whether this node's source AST has been materialized in this graph.
    #[must_use]
    pub fn has_parsed_source(&self) -> bool {
        self.unit.get().is_some()
    }
}

/// Parsed project units plus metadata needed to resolve imports and reachability.
#[derive(Debug, Clone)]
pub struct UnitGraph {
    nodes: HashMap<String, UnitNode>,
    link_meta: ProjectLinkMeta,
    source_paths: Vec<PathBuf>,
}

impl UnitGraph {
    pub(super) fn new(
        nodes: HashMap<String, UnitNode>,
        link_meta: ProjectLinkMeta,
        source_paths: Vec<PathBuf>,
    ) -> Self {
        Self {
            nodes,
            link_meta,
            source_paths,
        }
    }

    /// Returns a unit by canonical, case-insensitive name.
    #[must_use]
    pub fn get(&self, canonical_name: &str) -> Option<&UnitNode> {
        self.nodes.get(canonical_name)
    }

    /// Returns whether the graph contains a canonical unit name.
    #[must_use]
    pub fn contains(&self, canonical_name: &str) -> bool {
        self.nodes.contains_key(canonical_name)
    }

    /// Iterates over all nodes. Callers needing deterministic order should sort by key.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &UnitNode)> {
        self.nodes.iter().map(|(key, node)| (key.as_str(), node))
    }

    /// Number of parsed source units.
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns whether no source units were loaded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Source paths indexed by the diagnostic source IDs assigned to graph units.
    #[must_use]
    pub fn source_paths(&self) -> &[PathBuf] {
        &self.source_paths
    }

    pub(crate) fn link_meta(&self) -> &ProjectLinkMeta {
        &self.link_meta
    }
}

/// A deterministic dependency-first selection of units from a [`UnitGraph`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedUnitGraph {
    order: Vec<String>,
}

impl ResolvedUnitGraph {
    pub(super) fn new(order: Vec<String>) -> Self {
        Self { order }
    }

    /// Canonical unit names in dependency-first order.
    #[must_use]
    pub fn order(&self) -> &[String] {
        &self.order
    }

    /// Returns whether no source units are selected.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    /// Number of selected source units.
    #[must_use]
    pub fn len(&self) -> usize {
        self.order.len()
    }
}
