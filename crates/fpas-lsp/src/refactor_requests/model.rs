//! Editor recipes use URIs and UTF-16 ranges; portable plans remain inside the shared core.

use fpas_language_service::{
    RefactorBlocker, RefactorStatus, ScopeContext, ScopeSnapshot, TargetReference,
};
use serde::{Deserialize, Serialize};
use tower_lsp_server::ls_types::{Position, Range, Uri, WorkspaceEdit};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "phase", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum RefactorParams {
    Prepare {
        version: u32,
        context: Option<Uri>,
        recipe: Box<EditorRecipe>,
    },
    Revalidate {
        version: u32,
        handle: String,
    },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum EditorSelector {
    Position { uri: Uri, position: Position },
    Symbol { name: String },
    Reference { target_ref: Box<TargetReference> },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "action", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum EditorRecipe {
    AddParameter {
        selector: EditorSelector,
        name: String,
        r#type: String,
        mode: fpas_language_service::ParameterMode,
        argument: String,
    },
    Rename {
        selector: EditorSelector,
        name: String,
    },
    ExtractVariable {
        uri: Uri,
        range: Range,
    },
    OrganizeImports {
        uri: Uri,
    },
}

#[derive(Debug, Serialize)]
pub(crate) struct RefactorResponse {
    pub version: u32,
    pub status: RefactorStatus,
    pub context: ScopeContext,
    pub snapshot: ScopeSnapshot,
    pub handle: Option<String>,
    pub edit: Option<WorkspaceEdit>,
    pub blockers: Vec<RefactorBlocker>,
    pub diagnostics: Vec<serde_json::Value>,
    pub limitations: Vec<String>,
    pub coverage: RefactorCoverage,
}

#[derive(Debug, Serialize)]
pub(crate) struct RefactorCoverage {
    pub analysis_complete: bool,
    pub reasons: std::collections::BTreeSet<String>,
}
