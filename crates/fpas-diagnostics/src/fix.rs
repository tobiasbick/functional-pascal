//! Source edits that correct a diagnostic, usable by `fpas check --fix` and editor quick fixes.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Fixes)

use crate::SourceSpan;

/// Whether a fix may be applied without the author reviewing it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Applicability {
    /// The edits are correct as written; `fpas check --fix` applies them.
    MachineApplicable,
    /// The edits may change meaning or contain placeholders; they are offered, never applied
    /// automatically.
    NeedsReview,
}

impl Applicability {
    /// Returns the stable JSON spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MachineApplicable => "machine-applicable",
            Self::NeedsReview => "needs-review",
        }
    }
}

/// One replacement of a source range; a zero-length span is an insertion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixEdit {
    /// Replaced byte range in the diagnostic's source.
    pub span: SourceSpan,
    /// Text written in place of the range.
    pub new_text: String,
}

/// Edits that together correct one diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fix {
    /// Short imperative description, such as "Replace `&&` with `and`".
    pub title: String,
    /// Whether the fix may be applied automatically.
    pub applicability: Applicability,
    /// Non-overlapping edits in one source, ordered by offset.
    pub edits: Vec<FixEdit>,
}

impl Fix {
    /// Creates a fix and orders its edits by source offset.
    ///
    /// Returns `None` when the edits are empty, overlap, or belong to different sources, so
    /// producers never publish a fix that cannot be applied as one change.
    #[must_use]
    pub fn new(
        title: impl Into<String>,
        applicability: Applicability,
        mut edits: Vec<FixEdit>,
    ) -> Option<Self> {
        edits.sort_by_key(|edit| (edit.span.offset(), edit.span.end()));
        let first = edits.first()?;
        let source_id = first.span.source_id();
        let consistent = edits
            .iter()
            .all(|edit| edit.span.source_id() == source_id && !edit.span.is_synthetic());
        let disjoint = edits
            .windows(2)
            .all(|pair| pair[0].span.end() <= pair[1].span.offset());
        (consistent && disjoint).then(|| Self {
            title: title.into(),
            applicability,
            edits,
        })
    }

    /// Creates a fix that replaces one range.
    #[must_use]
    pub fn replace(
        title: impl Into<String>,
        applicability: Applicability,
        span: SourceSpan,
        new_text: impl Into<String>,
    ) -> Option<Self> {
        Self::new(
            title,
            applicability,
            vec![FixEdit {
                span,
                new_text: new_text.into(),
            }],
        )
    }

    /// Returns `true` when `fpas check --fix` may apply this fix.
    #[must_use]
    pub fn is_machine_applicable(&self) -> bool {
        self.applicability == Applicability::MachineApplicable
    }

    /// Applies the edits to `source`, which must be the text the spans were produced for.
    ///
    /// Returns `None` when an edit lies outside `source` or splits a UTF-8 character.
    #[must_use]
    pub fn apply(&self, source: &str) -> Option<String> {
        apply_edits(source, self.edits.iter())
    }
}

/// Applies, in order, every machine-applicable fix whose edits neither overlap nor touch an
/// edit already chosen, and returns the new text with the number of fixes applied.
///
/// Touching edits conflict too, so two insertions at one point never depend on their order;
/// a skipped fix is reported again by the next check of the changed text.
#[must_use]
pub fn apply_machine_applicable<'a>(
    source: &str,
    fixes: impl IntoIterator<Item = &'a Fix>,
) -> (String, usize) {
    let mut chosen: Vec<&FixEdit> = Vec::new();
    let mut applied = 0;
    for fix in fixes {
        if !fix.is_machine_applicable() || fix.apply(source).is_none() {
            continue;
        }
        let conflicts = fix.edits.iter().any(|edit| {
            chosen.iter().any(|other| {
                edit.span.offset() <= other.span.end() && other.span.offset() <= edit.span.end()
            })
        });
        if !conflicts {
            chosen.extend(&fix.edits);
            applied += 1;
        }
    }
    chosen.sort_by_key(|edit| edit.span.offset());
    match apply_edits(source, chosen) {
        Some(text) => (text, applied),
        None => (source.to_owned(), 0),
    }
}

/// Titles a fix after its replacement text, or generically when that text is long.
#[must_use]
pub fn fix_title(replacement: &str) -> String {
    const MAX_TITLE_REPLACEMENT: usize = 40;
    if replacement.contains('\n') || replacement.chars().count() > MAX_TITLE_REPLACEMENT {
        "Rewrite in FPAS form".to_owned()
    } else if replacement.is_empty() {
        "Remove the foreign text".to_owned()
    } else {
        format!("Write `{replacement}`")
    }
}

/// Applies disjoint edits, ordered by offset, to `source`.
///
/// Returns `None` when an edit overlaps its predecessor, lies outside `source`, or splits a
/// UTF-8 character.
#[must_use]
pub fn apply_edits<'a>(
    source: &str,
    edits: impl IntoIterator<Item = &'a FixEdit>,
) -> Option<String> {
    let mut output = String::with_capacity(source.len());
    let mut copied = 0;
    for edit in edits {
        let start = edit.span.offset();
        let end = edit.span.end();
        if start < copied {
            return None;
        }
        output.push_str(source.get(copied..start)?);
        source.get(start..end)?;
        output.push_str(&edit.new_text);
        copied = end;
    }
    output.push_str(source.get(copied..)?);
    Some(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edit(offset: usize, length: usize, text: &str) -> FixEdit {
        FixEdit {
            span: SourceSpan::new(offset, length, 1, offset as u32 + 1),
            new_text: text.to_owned(),
        }
    }

    #[test]
    fn new_orders_edits_and_rejects_overlaps() {
        let fix = Fix::new(
            "t",
            Applicability::MachineApplicable,
            vec![edit(4, 1, "b"), edit(0, 1, "a")],
        )
        .unwrap();
        assert_eq!(fix.edits[0].span.offset(), 0);
        assert!(
            Fix::new(
                "t",
                Applicability::NeedsReview,
                vec![edit(0, 3, ""), edit(2, 1, "")]
            )
            .is_none()
        );
        assert!(Fix::new("t", Applicability::NeedsReview, Vec::new()).is_none());
    }

    #[test]
    fn apply_replaces_and_inserts() {
        let fix = Fix::new(
            "t",
            Applicability::MachineApplicable,
            vec![edit(2, 2, "and"), edit(0, 0, "(")],
        )
        .unwrap();
        assert_eq!(fix.apply("a && b").as_deref(), Some("(a and b"));
        assert_eq!(fix.apply("é").as_deref(), None);
    }

    #[test]
    fn machine_applicable_fixes_skip_review_and_touching_edits() {
        let fix = |applicability, edits| Fix::new("t", applicability, edits).unwrap();
        let fixes = [
            fix(Applicability::MachineApplicable, vec![edit(2, 2, "and")]),
            fix(Applicability::MachineApplicable, vec![edit(4, 0, " x")]),
            fix(Applicability::NeedsReview, vec![edit(0, 1, "B")]),
            fix(Applicability::MachineApplicable, vec![edit(5, 1, "C")]),
        ];
        let (text, applied) = apply_machine_applicable("a && b", &fixes);
        assert_eq!((text.as_str(), applied), ("a and C", 2));
    }
}
