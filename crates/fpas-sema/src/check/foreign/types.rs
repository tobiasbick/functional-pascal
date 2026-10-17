//! Type names from other languages and their FPAS types.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings),
//! `docs/pascal/language/types/README.md`

use fpas_diagnostics::ForeignHabit;

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
