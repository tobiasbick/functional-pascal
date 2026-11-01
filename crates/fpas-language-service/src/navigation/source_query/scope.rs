//! Operation-independent checked inputs for saved-source and editor requests.

use super::SourceQueryError;
use crate::analysis::identity::encoding::CanonicalHasher;
use crate::{
    ANALYSIS_BUILD_ID, CancellationToken, DocumentSnapshot, InventoryIdentity, LanguageService,
    LanguageServiceError, WorkspaceKind, identity_path,
};
use fpas_project::source_inputs::{InputInventory, SourceInputs};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::Path, sync::Arc};

/// Discovery policy is explicit; catalogue entries cannot prove checked uniqueness.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryPolicy {
    /// Owned sources and eligible imported declarations, with complete checked analysis.
    CheckedContext,
    /// Interactive editor discovery, including loose buffers and unimported standard APIs.
    EditorCatalog,
}

/// Manifest ownership used by a strict source request.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopeKind {
    /// Interactive catalogue discovered below a folder; never a checked mutation context.
    Folder,
    /// A single source without an owning manifest.
    Loose,
    /// One project and its dependencies.
    Project,
    /// Explicit workspace members and their dependencies.
    Workspace,
}

/// Context reconstruction information, independent of either adapter's cwd or URI encoding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScopeContext {
    /// Owning manifest, or loose source when no manifest exists.
    pub input: String,
    /// Scope category.
    pub kind: ScopeKind,
    /// Physical context root; mutations cannot expand beyond it.
    pub root: String,
    /// Selected source standard library, if present.
    pub standard_library: Option<String>,
    /// Sorted participating project manifests.
    pub projects: Vec<String>,
    /// Analysis and declaration-discovery policy.
    pub policy: QueryPolicy,
}

/// Complete content identity with an auditable input inventory.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScopeSnapshot {
    /// SHA-256 of the framed context and inventory.
    pub id: String,
    /// Shared semantic implementation identity.
    pub analysis_identity: String,
    /// All input hashes, aliases and membership records.
    pub inventory: InventoryIdentity,
}

impl ScopeSnapshot {
    pub(crate) fn new(context: &ScopeContext, inventory: InventoryIdentity) -> Self {
        let mut hash = CanonicalHasher::new("fpas-source-snapshot-v1");
        hash.text(ANALYSIS_BUILD_ID);
        hash.text(match context.kind {
            ScopeKind::Folder => "folder",
            ScopeKind::Loose => "loose",
            ScopeKind::Project => "project",
            ScopeKind::Workspace => "workspace",
        });
        hash.text(&context.input);
        hash.text(&context.root);
        hash.count(usize::from(context.standard_library.is_some()));
        if let Some(library) = &context.standard_library {
            hash.text(library);
        }
        hash.text(match context.policy {
            QueryPolicy::CheckedContext => "checked_context",
            QueryPolicy::EditorCatalog => "editor_catalog",
        });
        hash.count(context.projects.len());
        for project in &context.projects {
            hash.text(project);
        }
        hash.text(&inventory.digest());
        Self {
            id: hash.finish(),
            analysis_identity: ANALYSIS_BUILD_ID.into(),
            inventory,
        }
    }
}

/// One synchronous frozen context. Both adapters use this for complete queries and edits.
/// Ordinary interactive navigation keeps the narrower cached `LanguageService` API.
pub struct SourceSession {
    service: LanguageService,
    inputs: SourceInputs,
    context: ScopeContext,
    snapshot: ScopeSnapshot,
    inventory: InputInventory,
}

impl SourceSession {
    /// Loads saved manifests and overlays every participating open source before discovery.
    pub fn load(
        input: &Path,
        standard_library: Option<&Path>,
        overlays: &[Arc<DocumentSnapshot>],
        cancellation: &CancellationToken,
    ) -> Result<Self, SourceQueryError> {
        Self::load_with_policy(
            input,
            standard_library,
            overlays,
            QueryPolicy::CheckedContext,
            cancellation,
        )
    }

    /// Loads an explicit editor catalogue without granting checked-reference authority.
    pub fn load_with_policy(
        input: &Path,
        standard_library: Option<&Path>,
        overlays: &[Arc<DocumentSnapshot>],
        policy: QueryPolicy,
        cancellation: &CancellationToken,
    ) -> Result<Self, SourceQueryError> {
        cancellation.check()?;
        let probe = cancellation.clone();
        let inputs = SourceInputs::begin_with_overlays(
            overlays
                .iter()
                .map(|s| (s.path().to_path_buf(), s.source().as_bytes())),
            move || probe.is_cancelled(),
        )
        .map_err(|e| {
            SourceQueryError::new(
                if e.kind() == std::io::ErrorKind::FileTooLarge {
                    "source_limit"
                } else {
                    "source_read"
                },
                e.to_string(),
            )
        })?;
        enum Preparation {
            Manifest(std::path::PathBuf),
            Scope(Box<(LanguageService, ScopeContext, ScopeSnapshot, InputInventory)>),
        }
        let prepared = (|| -> Result<_, LanguageServiceError> {
            let mut service = match standard_library {
                Some(library) => LanguageService::load_with_standard_library_and_cancellation(
                    input,
                    library,
                    cancellation,
                )?,
                None => LanguageService::load_with_cancellation(input, cancellation)?,
            };
            for overlay in overlays {
                if let crate::SourceVersion::Editor(version) = overlay.version() {
                    service.documents_mut().open_document(
                        overlay.path(),
                        version,
                        overlay.source(),
                    )?;
                }
            }
            if let Some(issue) = service.workspace().issues().first() {
                return Err(issue.clone().into_analysis_error());
            }
            if input
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("fpas"))
                && let Some(manifest) = service.workspace().manifest_path()
            {
                // Ownership discovery chooses the manifest; its search probes are not inputs
                // of that explicit semantic scope. Rebuild once so a recorded reference and
                // an inferred positional request have the same complete inventory.
                return Ok(Preparation::Manifest(manifest.to_path_buf()));
            }
            let kind = match service.workspace().kind() {
                WorkspaceKind::Loose => ScopeKind::Loose,
                WorkspaceKind::Project => ScopeKind::Project,
                WorkspaceKind::Workspace => ScopeKind::Workspace,
                WorkspaceKind::Folder if policy == QueryPolicy::EditorCatalog => ScopeKind::Folder,
                _ => {
                    return Err(LanguageServiceError::analysis(
                        input,
                        "A source query requires a source or explicit manifest context.",
                    ));
                }
            };
            let mut paths = service
                .workspace()
                .projects()
                .iter()
                .flat_map(|p| p.all_source_paths())
                .collect::<BTreeSet<_>>();
            paths.extend(service.standard_library_source_paths());
            paths.extend(service.editor_api_source_paths());
            if policy == QueryPolicy::EditorCatalog {
                paths.extend(service.catalog_source_paths());
            }
            if kind == ScopeKind::Loose {
                paths.insert(input.to_path_buf());
            }
            for path in paths {
                cancellation.check()?;
                // An open snapshot bypasses the disk reader, so admit its physical input
                // explicitly before freezing the inventory, including loose editor files.
                fpas_project::source_inputs::select(&path)
                    .map_err(|e| LanguageServiceError::source_read(&path, e))?;
                service.snapshot(&path)?;
            }
            inputs
                .freeze_selected()
                .map_err(|e| LanguageServiceError::source_read(input, e))?;
            let inventory = inputs
                .inventory()
                .map_err(|e| LanguageServiceError::source_read(input, e))?;
            let path_identity = |path: &Path| {
                std::fs::canonicalize(path)
                    .and_then(|p| identity_path(&p))
                    .map_err(|e| LanguageServiceError::source_read(path, e))
            };
            let mut projects = service
                .workspace()
                .projects()
                .iter()
                .map(|p| path_identity(p.manifest_path()))
                .collect::<Result<Vec<_>, _>>()?;
            projects.sort();
            let context = ScopeContext {
                input: path_identity(service.workspace().manifest_path().unwrap_or(input))?,
                kind,
                root: path_identity(service.workspace().root())?,
                standard_library: standard_library.map(path_identity).transpose()?,
                projects,
                policy,
            };
            let metadata = InventoryIdentity::from_inputs(&inventory)
                .map_err(|e| LanguageServiceError::source_read(input, e))?;
            let snapshot = ScopeSnapshot::new(&context, metadata);
            Ok(Preparation::Scope(Box::new((
                service, context, snapshot, inventory,
            ))))
        })();
        let prepared = prepared.map_err(|error| SourceQueryError {
            code: inputs.reason().unwrap_or("context_error"),
            message: error.to_string(),
            diagnostics: error.diagnostics(),
        })?;
        let (service, context, snapshot, inventory) = match prepared {
            Preparation::Scope(scope) => *scope,
            Preparation::Manifest(manifest) => {
                drop(inputs);
                return Self::load_with_policy(
                    &manifest,
                    standard_library,
                    overlays,
                    policy,
                    cancellation,
                );
            }
        };
        Ok(Self {
            service,
            inputs,
            context,
            snapshot,
            inventory,
        })
    }

    /// Shared semantic engine over these immutable inputs.
    pub fn service(&mut self) -> &mut LanguageService {
        &mut self.service
    }
    /// Context that must be reconstructed when reusing a reference or plan.
    pub fn context(&self) -> &ScopeContext {
        &self.context
    }
    /// Identity before any operation-specific work.
    pub fn snapshot(&self) -> &ScopeSnapshot {
        &self.snapshot
    }
    /// Owned bytes and discovery records for a surviving publication coordinator.
    pub fn inventory(&self) -> &InputInventory {
        &self.inventory
    }

    /// Verifies unchanged inputs and rejects a query that escaped its original read set.
    pub fn verify(&self) -> Result<bool, LanguageServiceError> {
        let input = Path::new(&self.context.input);
        let current = self
            .inputs
            .inventory()
            .and_then(|i| InventoryIdentity::from_inputs(&i))
            .map_err(|e| LanguageServiceError::source_read(input, e))?;
        if current != self.snapshot.inventory {
            return Ok(false);
        }
        self.inventory
            .verify()
            .map_err(|e| LanguageServiceError::source_read(input, e))
    }
}
