//! Versioned extension wire types; positions follow the LSP UTF-16 convention.

use fpas_language_service::{
    QueryPolicy, ScopeContext, ScopeSnapshot, SymbolKind, SymbolVisibility, TargetReference,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use tower_lsp_server::ls_types::{Position, Range, Uri};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SourceOperation {
    Inspect,
    Callers,
    Callees,
    Find,
    Refs,
    Definition,
    Type,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SourceQueryParams {
    pub version: u32,
    pub operation: SourceOperation,
    pub context: Option<Uri>,
    pub selector: SourceSelector,
    pub limit: Option<usize>,
    pub policy: Option<QueryPolicy>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum SourceSelector {
    Position { uri: Uri, position: Position },
    Reference { target_ref: Box<TargetReference> },
    Symbol { name: String },
    Search { text: String },
}

#[derive(Serialize)]
pub(crate) struct SourceQueryResponse {
    pub version: u32,
    pub outcome: &'static str,
    pub context: ScopeContext,
    pub snapshot: ScopeSnapshot,
    pub target: Option<SourceTarget>,
    pub candidates: Vec<SourceTarget>,
    pub result: SourcePayload,
    pub diagnostics: Vec<Value>,
    pub coverage: SourceCoverage,
}

#[derive(Serialize)]
pub(crate) struct SourceTarget {
    pub name: String,
    pub qualified_name: String,
    pub kind: Option<SymbolKind>,
    pub declaration: Option<SourceLocation>,
    pub target_ref: Option<TargetReference>,
    pub reference_unavailable: Option<&'static str>,
    pub owner: Option<String>,
    pub contexts: Vec<String>,
    pub visibility: Option<SymbolVisibility>,
    pub origin: String,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub checked: bool,
}

#[derive(Serialize)]
pub(crate) struct SourceLocation {
    pub uri: Uri,
    pub range: Range,
    pub byte_start: usize,
    pub byte_length: usize,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum SourcePayload {
    Callers {
        #[serde(flatten)]
        calls: fpas_language_service::CallRelationsProjection<SourceLocation>,
    },
    Callees {
        #[serde(flatten)]
        calls: fpas_language_service::CallRelationsProjection<SourceLocation>,
    },
    Inspect {
        #[serde(flatten)]
        inspection: fpas_language_service::InspectionProjection<SourceLocation>,
    },
    Find {
        symbols: Vec<SourceTarget>,
    },
    Definition {
        locations: Vec<SourceLocation>,
    },
    Refs {
        locations: Vec<SourceLocation>,
    },
    Type {
        r#type: Option<SourceType>,
        unavailable_reason: Option<&'static str>,
    },
}

#[derive(Serialize)]
pub(crate) struct SourceType {
    pub display: String,
    pub kind: String,
    pub location: Option<SourceLocation>,
    pub declaration: Option<SourceLocation>,
}

#[derive(Serialize, Default)]
pub(crate) struct SourceCoverage {
    pub analysis_complete: bool,
    pub reasons: BTreeSet<String>,
    pub collections: BTreeMap<String, SourceCollection>,
    pub omissions: Vec<String>,
}

#[derive(Serialize)]
pub(crate) struct SourceCollection {
    pub returned: usize,
    pub known_total: Option<usize>,
    pub complete: bool,
}
