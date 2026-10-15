//! Version-one JSON views over detached VM observations.
//!
//! Contract: `docs/pascal/tools/failure-dump.md`.

use super::value::{Policy, Value};
use fpas_bytecode::{DebugBindingKind, DebugSourceLocation};
use fpas_diagnostics::Diagnostic;
use fpas_vm::{FailureBinding, FailureFrame, FailureGlobal, FailureTask};
use serde::{Serialize, Serializer, ser::SerializeSeq};

/// One complete document; optional missing data remains explicit JSON null.
#[derive(Serialize)]
pub(super) struct Report<'a> {
    pub(super) schema_version: u8,
    pub(super) error: Error<'a>,
    pub(super) related_errors: Vec<Error<'a>>,
    pub(super) sources: Vec<Source<'a>>,
    pub(super) capture: Capture,
    pub(super) globals: Globals<'a>,
    pub(super) tasks: Vec<Task<'a>>,
}

/// Selected diagnostic fields; presentation text remains borrowed and bounded.
#[derive(Serialize)]
pub(super) struct Error<'a> {
    pub(super) code: String,
    pub(super) message: &'a str,
    pub(super) hint: Option<&'a str>,
    pub(super) task_id: Option<u64>,
    pub(super) location: Option<Location>,
}

impl<'a> Error<'a> {
    /// Preserve known diagnostic starts; runtime spans need not carry a source byte range.
    pub(super) fn new(error: &'a Diagnostic, task_id: Option<u64>) -> Self {
        Self {
            code: error.code.to_string(),
            message: prefix(&error.message),
            hint: error.help.as_deref().map(prefix),
            task_id,
            location: error.span.map(|span| Location {
                source_id: span.source_id(),
                line: span.line(),
                column: span.column(),
            }),
        }
    }
}

/// Bound fallback diagnostics too, without allocating a complete message copy.
pub(super) fn prefix(text: &str) -> &str {
    text.char_indices()
        .nth(4096)
        .map_or(text, |(end, _)| &text[..end])
}

/// A portable image position; all coordinates are one-based.
#[derive(Clone, Copy, Serialize)]
pub(super) struct Location {
    pub(super) source_id: u32,
    pub(super) line: u32,
    pub(super) column: u32,
}
impl From<DebugSourceLocation> for Location {
    fn from(location: DebugSourceLocation) -> Self {
        Self {
            source_id: location.source.get(),
            line: location.line,
            column: location.column,
        }
    }
}

/// One source used by a delivered error/frame, identified independently of its portable path.
#[derive(Serialize)]
pub(super) struct Source<'a> {
    pub(super) id: u32,
    pub(super) path: Option<&'a str>,
}

/// Completeness includes both capture-time and serialized-document omissions.
#[derive(Serialize)]
pub(super) struct Capture {
    pub(super) consistency: &'static str,
    pub(super) incomplete: bool,
    pub(super) reasons: Vec<&'static str>,
    pub(super) total_tasks: Option<usize>,
    pub(super) omitted_tasks: Option<usize>,
    pub(super) globals_status: &'static str,
    pub(super) limits: Limits,
}

/// Fixed schema-version-one bounds; these are not CLI tuning options.
#[derive(Serialize)]
pub(super) struct Limits {
    tasks: usize,
    frames_per_task: usize,
    depth: usize,
    children_per_value: usize,
    string_chars: usize,
    values: usize,
    retained_bytes: usize,
    bytes: usize,
    peer_collection_ms: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            tasks: 1024,
            frames_per_task: 256,
            depth: 16,
            children_per_value: 256,
            string_chars: 4096,
            values: 16384,
            retained_bytes: 8388608,
            bytes: super::budget::MAX_BYTES,
            peer_collection_ms: 500,
        }
    }
}

/// One selected task; final output is sorted by runtime identity.
#[derive(Serialize)]
pub(super) struct Task<'a> {
    pub(super) id: u64,
    pub(super) name: Option<&'a str>,
    pub(super) state: &'static str,
    pub(super) observation: &'static str,
    pub(super) inspection: &'static str,
    pub(super) unavailable_reason: Option<&'static str>,
    pub(super) total_frames: Option<usize>,
    pub(super) frames: Vec<Frame<'a>>,
}
impl<'a> Task<'a> {
    /// Retain metadata before allocating any optional frame planning entries.
    pub(super) fn new(task: &'a FailureTask) -> Self {
        Self {
            id: task.id,
            name: task.name.as_deref(),
            state: task.state,
            observation: task.observation,
            inspection: task.inspection,
            unavailable_reason: task.reason,
            total_frames: task
                .snapshot
                .as_ref()
                .and_then(|snapshot| snapshot.total_frames),
            frames: Vec::new(),
        }
    }
    /// Make every serialization omission visible on the containing task.
    pub(super) fn limited(&mut self) {
        self.inspection = "partial";
        self.unavailable_reason = Some("byte_limit");
    }
}

/// Frame headers survive omitted bindings, with an explicit truncation marker.
#[derive(Serialize)]
pub(super) struct Frame<'a> {
    pub(super) depth: usize,
    pub(super) function: &'a str,
    pub(super) location: Option<Location>,
    pub(super) bindings: Bindings<'a>,
    pub(super) truncated: bool,
}
impl<'a> Frame<'a> {
    /// Start with no optional bindings; the planner admits a bounded prefix.
    pub(super) fn new(frame: &'a FailureFrame) -> Self {
        Self {
            depth: frame.depth,
            function: &frame.function,
            location: frame.location.map(Into::into),
            bindings: Bindings(&[], None),
            truncated: frame.reason.is_some(),
        }
    }
}

/// Source-visible bindings retain lexical identities even when names repeat.
pub(super) struct Bindings<'a>(pub(super) &'a [FailureBinding], pub(super) Option<Policy>);
impl Serialize for Bindings<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut array = serializer.serialize_seq(Some(self.0.len()))?;
        for (index, binding) in self.0.iter().enumerate() {
            let policy = if index + 1 == self.0.len() {
                self.1.unwrap_or_default()
            } else {
                Policy::default()
            };
            array.serialize_element(&Binding::local(binding).with_policy(policy))?;
        }
        array.end()
    }
}

/// Shared globals appear exactly once instead of being copied into each task.
pub(super) struct Globals<'a>(pub(super) &'a [FailureGlobal], pub(super) Option<Policy>);
impl Serialize for Globals<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut array = serializer.serialize_seq(Some(self.0.len()))?;
        for (index, binding) in self.0.iter().enumerate() {
            let policy = if index + 1 == self.0.len() {
                self.1.unwrap_or_default()
            } else {
                Policy::default()
            };
            array.serialize_element(&Binding::global(binding).with_policy(policy))?;
        }
        array.end()
    }
}

/// A borrowed binding's wire shape, shared by global and frame arrays.
#[derive(Serialize)]
pub(super) struct Binding<'a> {
    name: &'a str,
    kind: &'static str,
    scope_id: Option<u32>,
    value: Value<'a>,
}
impl<'a> Binding<'a> {
    /// Apply a borrowed document-size clipping policy to this binding's value.
    pub(super) fn with_policy(mut self, policy: Policy) -> Self {
        self.value.policy = policy;
        self
    }
    /// Parameters, locals and captures retain their original scope identifiers.
    pub(super) fn local(binding: &'a FailureBinding) -> Self {
        Self {
            name: &binding.name,
            kind: match binding.kind {
                DebugBindingKind::Parameter => "parameter",
                DebugBindingKind::Local => "local",
                DebugBindingKind::Capture => "capture",
            },
            scope_id: Some(binding.scope_id),
            value: Value::new(&binding.value),
        }
    }
    /// Globals have no function-local lexical scope.
    pub(super) fn global(binding: &'a FailureGlobal) -> Self {
        Self {
            name: &binding.name,
            kind: "global",
            scope_id: None,
            value: Value::new(&binding.value),
        }
    }
}
