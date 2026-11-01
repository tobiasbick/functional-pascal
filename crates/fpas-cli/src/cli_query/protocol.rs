//! CLI projections of shared source facts. These types contain no semantic resolution.

use fpas_language_service::{ScopeContext, ScopeSnapshot, TargetReference};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Outcome {
    Ok,
    NotFound,
    Incomplete,
    Error,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct Report {
    pub schema_version: u32,
    pub operation: Option<String>,
    pub outcome: Outcome,
    pub exit_code: i32,
    pub request: Request,
    pub context: Option<Context>,
    pub snapshot: Option<ScopeSnapshot>,
    pub target: Option<Target>,
    pub candidates: Vec<Target>,
    pub result: Option<Payload>,
    pub diagnostics: Vec<Value>,
    pub coverage: Coverage,
    pub limits: Limits,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(super) struct Request {
    pub selector: Option<Selector>,
    pub arguments: BTreeMap<String, Value>,
    pub options: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(super) enum Selector {
    Search {
        text: String,
    },
    Symbol {
        name: String,
    },
    Reference {
        path: String,
    },
    Position {
        path: String,
        line: usize,
        column: usize,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct Context {
    #[serde(flatten)]
    pub scope: ScopeContext,
    pub path_base: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct Target {
    pub name: String,
    pub qualified_name: String,
    pub kind: String,
    pub origin: String,
    pub declaration: Option<Location>,
    pub target_ref: Option<TargetReference>,
    pub reference_unavailable: Option<String>,
    pub owner: Option<String>,
    pub contexts: Vec<String>,
    pub visibility: Option<fpas_language_service::SymbolVisibility>,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub checked: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(super) enum Payload {
    Refactor {
        status: fpas_language_service::RefactorStatus,
        plan: Option<Box<fpas_language_service::RefactorPlan>>,
        blockers: Vec<fpas_language_service::RefactorBlocker>,
        limitations: Vec<String>,
        publication: Option<super::refactor::Publication>,
    },
    Callers {
        #[serde(flatten)]
        calls: fpas_language_service::CallRelationsProjection<Location>,
    },
    Callees {
        #[serde(flatten)]
        calls: fpas_language_service::CallRelationsProjection<Location>,
    },
    Inspect {
        #[serde(flatten)]
        inspection: fpas_language_service::InspectionProjection<Location>,
    },
    Find {
        symbols: Vec<Target>,
    },
    Definition {
        locations: Vec<Location>,
    },
    Refs {
        locations: Vec<Location>,
    },
    Type {
        r#type: Option<TypeInfo>,
        unavailable_reason: Option<String>,
    },
}

impl Payload {
    pub fn locations_mut(&mut self) -> Option<&mut Vec<Location>> {
        match self {
            Self::Definition { locations } | Self::Refs { locations } => Some(locations),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct TypeInfo {
    pub display: String,
    pub kind: String,
    pub location: Option<Location>,
    pub declaration: Option<Location>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct Location {
    pub path: String,
    pub range: Range,
    pub byte_span: ByteSpan,
    pub preview: Preview,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct Position {
    pub line: usize,
    pub column: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct Range {
    pub start: Position,
    pub end: Position,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct ByteSpan {
    pub start: usize,
    pub length: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct Preview {
    pub text: String,
    pub column_start: usize,
    pub truncated: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(super) struct Coverage {
    pub analysis_complete: bool,
    pub reasons: BTreeSet<String>,
    pub collections: BTreeMap<String, Collection>,
    pub omissions: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct Collection {
    pub returned: usize,
    pub known_total: Option<usize>,
    pub complete: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct Limits {
    pub timeout_ms: u64,
    pub result_count: Option<usize>,
    pub input_files: usize,
    pub file_bytes: usize,
    pub input_bytes: usize,
    pub json_bytes: usize,
    pub preparation_bytes: usize,
    pub excerpt_bytes: usize,
    pub supporting_types: usize,
    pub text_scalars: usize,
    pub preview_scalars: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            timeout_ms: 30_000,
            result_count: None,
            input_files: 4096,
            file_bytes: 8 * 1024 * 1024,
            input_bytes: 64 * 1024 * 1024,
            json_bytes: super::report::MAX_JSON,
            preparation_bytes: 128 * 1024 * 1024,
            excerpt_bytes: 32 * 1024,
            supporting_types: 12,
            text_scalars: 4096,
            preview_scalars: 160,
        }
    }
}
