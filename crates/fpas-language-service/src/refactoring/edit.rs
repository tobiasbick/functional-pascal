//! Exact edits shared by navigation, refactoring recipes and both adapters.

use crate::DocumentSnapshot;
use fpas_diagnostics::SourceSpan;
use std::{path::PathBuf, sync::Arc};

/// One exact UTF-8 replacement against an immutable source snapshot.
#[derive(Debug, Clone)]
pub struct SourceEdit {
    /// Physical source path.
    pub path: PathBuf,
    /// Original source range; empty ranges insert text.
    pub range: SourceSpan,
    /// Complete replacement bytes represented as UTF-8, never clipped for publication.
    pub new_text: String,
    /// Source used to construct and validate this range.
    pub snapshot: Arc<DocumentSnapshot>,
}

impl PartialEq for SourceEdit {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
            && self.range == other.range
            && self.new_text == other.new_text
            && self.snapshot.source() == other.snapshot.source()
    }
}
impl Eq for SourceEdit {}

/// A fully assembled replacement, ready for adapter-owned staging or editor projection.
#[derive(Debug, Clone)]
pub struct SourceChange {
    /// Physical source path.
    pub path: PathBuf,
    /// Exact before bytes.
    pub original: Arc<DocumentSnapshot>,
    /// Exact after bytes.
    pub replacement: String,
}

/// Semantically validated edits; the adapter must still revalidate inputs before applying.
#[derive(Debug, Clone)]
pub struct CheckedChanges {
    /// Complete participating source snapshots, including unedited dependencies.
    pub inputs: Vec<Arc<DocumentSnapshot>>,
    /// Deterministic source edits against the original snapshots.
    pub edits: Vec<SourceEdit>,
    /// Whole-file replacements assembled from those exact edits.
    pub changes: Vec<SourceChange>,
}
