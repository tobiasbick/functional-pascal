//! Reachability and project export rules for source units.

use std::collections::HashSet;
use std::path::Path;

use fpas_parser::QualifiedId;

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

    pub(crate) fn validate_root_uses(&self, uses: &[QualifiedId]) -> Result<(), String> {
        if !self.graph.link_meta().enforces_export_rules() {
            return Ok(());
        }
        for used in uses {
            let target_key = canonical_unit_key(used);
            if !self.can_import(&SourceOrigin::Own, &target_key) {
                return Err(self.not_exported_error(&target_key));
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

    pub(crate) fn not_exported_error(&self, target_key: &str) -> String {
        let display = display_unit_key(target_key);
        let Some(target) = self.graph.get(target_key) else {
            return format!(
                "Unit `{display}` is not exported from its library project.\n  help: Add `{display}` to `[exports].units` in the library `.fpasprj`, or import a public unit that re-exports its API."
            );
        };
        let SourceOrigin::Library(library_project) = target.origin() else {
            return format!(
                "Unit `{display}` cannot be imported here.\n  help: Use a unit exported by the library project."
            );
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
        format!(
            "Unit `{display}` is not exported from library project `{}`.{policy_hint}\n  help: Add `{display}` to `[exports].units` in that `.fpasprj`, or depend on a public unit instead.",
            library_project.display()
        )
    }
}

pub(super) fn resolve_reachable(
    root_uses: &[QualifiedId],
    graph: &UnitGraph,
    policy: &ImportPolicy<'_>,
) -> Result<HashSet<String>, String> {
    policy.validate_root_uses(root_uses)?;
    let mut queue = Vec::<String>::new();
    let mut reachable = HashSet::<String>::new();

    for used in root_uses {
        if !is_intrinsic_std_unit(used, graph) {
            queue.push(canonical_unit_key(used));
        }
    }

    while let Some(next) = queue.pop() {
        if !reachable.insert(next.clone()) {
            continue;
        }
        let Some(node) = graph.get(&next) else {
            return Err(unknown_unit_error(&next, graph, "program"));
        };
        for used in node.direct_uses() {
            if is_intrinsic_std_unit(used, graph) {
                continue;
            }
            let dependency_key = canonical_unit_key(used);
            if !graph.contains(&dependency_key) {
                return Err(unknown_unit_error(
                    &dependency_key,
                    graph,
                    &format!("unit `{}`", node.display_name()),
                ));
            }
            if !policy.can_import_for_unit(&next, &dependency_key) {
                return Err(policy.not_exported_error(&dependency_key));
            }
            queue.push(dependency_key);
        }
    }

    Ok(reachable)
}

pub(super) fn all_library_units(graph: &UnitGraph) -> Result<HashSet<String>, String> {
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
            let dependency_key = canonical_unit_key(used);
            if !reachable.contains(&dependency_key) {
                return Err(unknown_unit_error(
                    &dependency_key,
                    graph,
                    &format!("unit `{}`", node.display_name()),
                ));
            }
            if !policy.can_import_for_unit(key, &dependency_key) {
                return Err(policy.not_exported_error(&dependency_key));
            }
        }
    }
    Ok(reachable)
}
