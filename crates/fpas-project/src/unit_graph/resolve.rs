//! Reachability and project export rules for source units.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md`

use std::collections::HashSet;
use std::path::Path;

use fpas_diagnostics::codes::PROJECT_UNIT_NOT_EXPORTED;

use crate::ProjectError;
use crate::model::{LibraryExportPolicy, SourceOrigin};
use crate::paths::same_file;

use super::model::UnitGraph;
use super::{canonical_unit_key, display_unit_key, is_intrinsic_std_unit, unknown_unit_error};

#[derive(Debug, Clone)]
pub(crate) struct ImportPolicy<'a> {
    graph: &'a UnitGraph,
}

impl<'a> ImportPolicy<'a> {
    pub(crate) fn new(graph: &'a UnitGraph) -> Self {
        Self { graph }
    }

    /// Checks root export rules while preserving the imported name and optional root path.
    pub(crate) fn validate_root_uses(
        &self,
        uses: &[fpas_parser::Import],
        root_path: Option<&Path>,
    ) -> Result<(), ProjectError> {
        if !self.graph.link_meta().enforces_export_rules() {
            return Ok(());
        }
        for used in uses {
            let target_key = canonical_unit_key(&used.unit);
            if !self.can_import(&SourceOrigin::Own, &target_key) {
                return Err(self
                    .not_exported_error(&target_key)
                    .at_source(root_path, used.unit.span));
            }
        }
        Ok(())
    }

    pub(crate) fn can_import_for_unit(&self, requester_key: &str, target_key: &str) -> bool {
        if !self.graph.link_meta().enforces_export_rules() {
            return true;
        }
        let requester = self
            .graph
            .get(requester_key)
            .map_or(SourceOrigin::Own, |node| node.origin().clone());
        self.can_import(&requester, target_key)
    }

    fn can_import(&self, requester: &SourceOrigin, target_key: &str) -> bool {
        let Some(target) = self.graph.get(target_key) else {
            return true;
        };
        self.allow_cross_origin(requester, target.origin(), target_key)
    }

    fn allow_cross_origin(
        &self,
        requester: &SourceOrigin,
        target: &SourceOrigin,
        target_key: &str,
    ) -> bool {
        match (requester, target) {
            (SourceOrigin::Own, SourceOrigin::Own) => true,
            (SourceOrigin::Library(requester_lib), SourceOrigin::Library(target_lib))
                if same_file(requester_lib, target_lib) =>
            {
                true
            }
            (SourceOrigin::Own, SourceOrigin::Library(library_project))
            | (SourceOrigin::Library(_), SourceOrigin::Library(library_project)) => {
                self.is_unit_exported(library_project, target_key)
            }
            (SourceOrigin::Library(_), SourceOrigin::Own) => false,
        }
    }

    fn is_unit_exported(&self, library_project: &Path, target_key: &str) -> bool {
        match self
            .graph
            .link_meta()
            .export_policy_for_library(library_project)
        {
            LibraryExportPolicy::AllUnits => true,
            LibraryExportPolicy::ListedUnits(listed) => listed.contains(target_key),
        }
    }

    pub(crate) fn not_exported_error(&self, target_key: &str) -> ProjectError {
        let display = display_unit_key(target_key);
        let Some(target) = self.graph.get(target_key) else {
            return ProjectError::new(
                PROJECT_UNIT_NOT_EXPORTED,
                format!("Unit `{display}` is not exported from its library project."),
            )
            .with_help(format!(
                "Add `{display}` to `[exports].units` in the library `.fpasprj`, or import a public unit that re-exports its API."
            ));
        };
        let SourceOrigin::Library(library_project) = target.origin() else {
            return ProjectError::new(
                PROJECT_UNIT_NOT_EXPORTED,
                format!("Unit `{display}` cannot be imported here."),
            )
            .with_help("Use a unit exported by the library project.");
        };
        let policy_hint = match self
            .graph
            .link_meta()
            .export_policy_for_library(library_project)
        {
            LibraryExportPolicy::AllUnits => String::new(),
            LibraryExportPolicy::ListedUnits(listed) => {
                let mut names = listed
                    .iter()
                    .map(|key| display_unit_key(key))
                    .collect::<Vec<_>>();
                names.sort();
                format!(
                    " Exported units from `{}`: {}.",
                    library_project.display(),
                    names.join(", ")
                )
            }
        };
        ProjectError::new(
            PROJECT_UNIT_NOT_EXPORTED,
            format!(
                "Unit `{display}` is not exported from library project `{}`.{policy_hint}",
                library_project.display()
            ),
        )
        .with_help(format!(
            "Add `{display}` to `[exports].units` in that `.fpasprj`, or depend on a public unit instead."
        ))
    }
}

/// Resolves program reachability without discarding root import provenance.
pub(super) fn resolve_reachable(
    root_uses: &[fpas_parser::Import],
    graph: &UnitGraph,
    policy: &ImportPolicy<'_>,
    root_path: Option<&Path>,
) -> Result<HashSet<String>, ProjectError> {
    policy.validate_root_uses(root_uses, root_path)?;
    let mut queue = Vec::<&fpas_parser::Import>::new();
    let mut reachable = HashSet::<String>::new();

    for used in root_uses {
        if !is_intrinsic_std_unit(used, graph) {
            queue.push(used);
        }
    }

    while let Some(used) = queue.pop() {
        let next = canonical_unit_key(&used.unit);
        if !reachable.insert(next.clone()) {
            continue;
        }
        let Some(node) = graph.get(&next) else {
            return Err(
                unknown_unit_error(&next, graph, "program").at_source(root_path, used.unit.span)
            );
        };
        for used in node.direct_uses() {
            if is_intrinsic_std_unit(used, graph) {
                continue;
            }
            let dependency_key = canonical_unit_key(&used.unit);
            if !graph.contains(&dependency_key) {
                return Err(unknown_unit_error(
                    &dependency_key,
                    graph,
                    &format!("unit `{}`", node.display_name()),
                )
                .at_source(Some(node.path()), used.span));
            }
            if !policy.can_import_for_unit(&next, &dependency_key) {
                return Err(policy
                    .not_exported_error(&dependency_key)
                    .at_source(Some(node.path()), used.span));
            }
            queue.push(used);
        }
    }

    Ok(reachable)
}

pub(super) fn all_library_units(graph: &UnitGraph) -> Result<HashSet<String>, ProjectError> {
    let policy = ImportPolicy::new(graph);
    let reachable = graph
        .iter()
        .map(|(key, _)| key.to_string())
        .collect::<HashSet<_>>();
    for (key, node) in graph.iter() {
        for used in node.direct_uses() {
            if is_intrinsic_std_unit(used, graph) {
                continue;
            }
            let dependency_key = canonical_unit_key(&used.unit);
            if !reachable.contains(&dependency_key) {
                return Err(unknown_unit_error(
                    &dependency_key,
                    graph,
                    &format!("unit `{}`", node.display_name()),
                )
                .at_source(Some(node.path()), used.span));
            }
            if !policy.can_import_for_unit(key, &dependency_key) {
                return Err(policy
                    .not_exported_error(&dependency_key)
                    .at_source(Some(node.path()), used.span));
            }
        }
    }
    Ok(reachable)
}
