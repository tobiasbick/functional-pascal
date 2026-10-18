//! Type names from other languages and their FPAS types.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings),
//! `docs/pascal/language/types/README.md`

use fpas_diagnostics::{Applicability, Fix, ForeignHabit, SourceSpan, fix_title};
use fpas_lexer::Span;

/// Lowercase foreign type names, their FPAS type and why. One line per group.
const FOREIGN_TYPES: &[(&[&str], &str, &str)] = &[
    (
        &["char", "widechar", "ansichar"],
        "string",
        "FPAS has no character type; one Unicode scalar is a `string` of length 1.",
    ),
    (
        &["double", "single", "extended", "float"],
        "real",
        "FPAS has one floating-point type, `real` (binary64).",
    ),
    (
        &[
            "longint", "int64", "cardinal", "longword", "smallint", "shortint", "byte", "word",
        ],
        "integer",
        "FPAS has one integer type, `integer` (signed 64-bit).",
    ),
    (
        &["ansistring", "widestring", "unicodestring", "shortstring"],
        "string",
        "FPAS has one string type, `string` (immutable Unicode).",
    ),
];

/// Returns the help for an unknown type name that is a known foreign type.
pub(crate) fn foreign_type_hint(name: &str) -> Option<String> {
    let lower = name.to_ascii_lowercase();
    let (_, replacement, explanation) = FOREIGN_TYPES
        .iter()
        .find(|(names, _, _)| names.contains(&lower.as_str()))?;
    Some(
        ForeignHabit {
            found: name,
            replacement,
            explanation,
        }
        .help(),
    )
}

/// Returns the fix that replaces a foreign type name over `span` with its FPAS type.
///
/// A character becomes a one-scalar `string`, which changes the operations on it, so that
/// replacement needs review.
pub(crate) fn foreign_type_fix(name: &str, span: Span) -> Option<Fix> {
    let lower = name.to_ascii_lowercase();
    let (names, replacement, _) = FOREIGN_TYPES
        .iter()
        .find(|(names, _, _)| names.contains(&lower.as_str()))?;
    let applicability = if names.contains(&"char") {
        Applicability::NeedsReview
    } else {
        Applicability::MachineApplicable
    };
    Fix::replace(
        fix_title(replacement),
        applicability,
        SourceSpan::try_from(span).ok()?,
        *replacement,
    )
}
