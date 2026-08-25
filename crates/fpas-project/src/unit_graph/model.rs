//! Unit graph data shared by source linking and compiled-unit builds.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use fpas_parser::{QualifiedId, Unit};
use fpas_unit::Digest;

use crate::model::{ProjectLinkMeta, SourceOrigin};
use crate::source::qualified_id_to_string;

/// One source unit and its project ownership metadata, parsed only when required.
#[derive(Debug, Clone)]
pub struct UnitNode {
    canonical_name: String,
    display_name: String,
    path: PathBuf,
    origin: SourceOrigin,
    source_id: u32,
    source_hash: Option<Digest>,
    direct_uses: Vec<QualifiedId>,
    unit: Arc<Unit>,
}

impl UnitNode {
    pub(super) fn new(
        path: PathBuf,
        origin: SourceOrigin,
        unit: Unit,
        source_hash: Option<Digest>,
    ) -> Self {
        let display_name = qualified_id_to_string(&unit.name);
        Self {
            canonical_name: display_name.to_ascii_lowercase(),
            display_name,
            path,
            origin,
            source_id: unit.span.source_id,
            source_hash,
            direct_uses: unit.uses.clone(),
            unit: Arc::new(unit),
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

    /// Hash of the exact source bytes used to construct this filesystem graph node.
    ///
    /// Parsed overlay graphs return `None` because their caller owns the source snapshot.
    #[must_use]
    pub fn source_hash(&self) -> Option<Digest> {
        self.source_hash
    }

    /// Direct unit dependencies from this unit's `uses` clause.
    #[must_use]
    pub fn direct_uses(&self) -> &[QualifiedId] {
        &self.direct_uses
    }

    /// Parsed source unit retained by this authoritative graph snapshot.
    #[must_use]
    pub fn parsed_unit(&self) -> &Unit {
        &self.unit
    }

    /// Parse an owned unit AST from the supplied source snapshot.
    ///
    /// The returned AST always originates from exactly `source`; callers can therefore couple
    /// compilation with the digest of the same bytes instead of a separately cached AST.
    ///
    /// # Errors
    ///
    /// Returns an error when the bytes are invalid source, no longer declare a unit, or declare a
    /// different unit name than the graph node.
    pub fn parse_source_snapshot(&self, source: &[u8]) -> Result<Unit, String> {
        let (parsed, _) =
            crate::source::parse_compilation_unit_source(&self.path, source, self.source_id)?;
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
                "Source file `{}` declares unit `{}`, but the build graph names `{}`.",
                self.path.display(),
                unit.name.parts.join("."),
                self.display_name
            ));
        }
        Ok(unit)
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

    pub(super) fn with_program_source_path(&self, main_path: &Path) -> Self {
        let mut graph = self.clone();
        graph.source_paths[0] = main_path.to_path_buf();
        graph
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
