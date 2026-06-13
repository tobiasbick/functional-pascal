//! Cross-project import rules for library `[exports]`.
//!
//! Documentation: `docs/pascal/10-projects.md`

use super::UnitFile;
use super::support::canonical_unit_key;
use crate::common::display_unit_key;
use crate::model::{LibraryExportPolicy, ProjectLinkMeta, SourceOrigin};
use crate::paths::same_file;

use std::collections::HashMap;
use std::path::Path;

/// Enforces which units a compilation unit may reference through `uses`.
#[derive(Debug, Clone)]
pub(super) struct ImportPolicy<'a> {
    meta: &'a ProjectLinkMeta,
    units: &'a HashMap<String, UnitFile>,
}

impl<'a> ImportPolicy<'a> {
    pub(super) fn new(meta: &'a ProjectLinkMeta, units: &'a HashMap<String, UnitFile>) -> Self {
        Self { meta, units }
    }

    /// Root program `uses` entries must target importable units.
    pub(super) fn validate_root_uses(
        &self,
        uses: &[fpas_parser::QualifiedId],
    ) -> Result<(), String> {
        if !self.meta.enforces_export_rules() {
            return Ok(());
        }
        for used in uses {
            if super::support::is_std_unit(used) {
                continue;
            }
            let target_key = canonical_unit_key(used);
            if !self.can_import(&SourceOrigin::Own, &target_key)? {
                return Err(self.not_exported_error(&target_key));
            }
        }
        Ok(())
    }

    pub(super) fn can_import_for_unit(
        &self,
        requester_key: &str,
        target_key: &str,
    ) -> Result<bool, String> {
        if !self.meta.enforces_export_rules() {
            return Ok(true);
        }
        let requester_origin = self.origin_for_unit_key(requester_key)?;
        self.can_import(&requester_origin, target_key)
    }

    fn origin_for_unit_key(&self, unit_key: &str) -> Result<SourceOrigin, String> {
        let Some(unit_file) = self.units.get(unit_key) else {
            return Ok(SourceOrigin::Own);
        };
        Ok(self.meta.origin_for_source(&unit_file.path))
    }

    fn can_import(&self, requester: &SourceOrigin, target_key: &str) -> Result<bool, String> {
        let Some(target_file) = self.units.get(target_key) else {
            return Ok(true);
        };
        let target_origin = self.meta.origin_for_source(&target_file.path);
        Ok(self.allow_cross_origin(requester, &target_origin, target_key))
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
                self.is_unit_exported(library_project.as_path(), target_key)
            }
            (SourceOrigin::Library(_), SourceOrigin::Own) => false,
        }
    }

    fn is_unit_exported(&self, library_project: &Path, target_key: &str) -> bool {
        match self.meta.export_policy_for_library(library_project) {
            LibraryExportPolicy::AllUnits => true,
            LibraryExportPolicy::ListedUnits(listed) => listed.contains(target_key),
        }
    }

    pub(super) fn not_exported_error(&self, target_key: &str) -> String {
        let display = display_unit_key(target_key);
        let Some(target_file) = self.units.get(target_key) else {
            return format!(
                "Unit `{display}` is not exported from its library project.\n  help: Add `{display}` to `[exports].units` in the library `.fpasprj`, or import a public unit that re-exports its API."
            );
        };
        let SourceOrigin::Library(library_project) = self.meta.origin_for_source(&target_file.path)
        else {
            return format!(
                "Unit `{display}` cannot be imported here.\n  help: Use a unit exported by the library project."
            );
        };
        let policy_hint = match self.meta.export_policy_for_library(&library_project) {
            LibraryExportPolicy::AllUnits => String::new(),
            LibraryExportPolicy::ListedUnits(listed) => {
                let mut names = listed
                    .iter()
                    .map(|key| display_unit_key(key))
                    .collect::<Vec<_>>();
                names.sort();
                format!(
                    " Exported units from `{}`: {}.",
                    library_project.to_string_lossy(),
                    names.join(", ")
                )
            }
        };
        format!(
            "Unit `{display}` is not exported from library project `{}`.{policy_hint}\n  help: Add `{display}` to `[exports].units` in that `.fpasprj`, or depend on a public unit instead.",
            library_project.to_string_lossy()
        )
    }
}
