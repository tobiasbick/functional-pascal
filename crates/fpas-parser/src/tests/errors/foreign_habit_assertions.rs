//! Shared assertions for foreign-habit diagnostics: exactly one diagnostic, its replacement
//! data, and recovery up to a later unrelated error.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings)

use super::*;
use fpas_diagnostics::{Diagnostic, DiagnosticCode};

/// A later, unrelated error that recovery must still reach.
pub(super) const LATER_ERROR: &str = "begin\n  Foo(1;\nend.";

/// Parser errors of a program source.
pub(super) fn parser_errors(source: &str) -> Vec<Diagnostic> {
    let (_, diagnostics) = parse_with_errors(source);
    diagnostics
        .iter()
        .filter_map(ParseDiagnostic::as_parser_error)
        .cloned()
        .collect()
}

/// Asserts exactly one habit diagnostic, then that recovery reaches [`LATER_ERROR`].
pub(super) fn assert_one_habit(
    declarations: &str,
    code: DiagnosticCode,
    message: &str,
    found: &str,
    replacement: &str,
) {
    let source = format!("program P;\n\n{declarations}\n\nbegin\nend.");
    let errors = parser_errors(&source);
    assert_eq!(errors.len(), 1, "{source}: {errors:#?}");
    let error = &errors[0];
    assert_eq!(error.code, code, "{source}");
    assert!(error.message.contains(message), "{source}: {error:#?}");
    assert_eq!(error.found.as_deref(), Some(found), "{source}");
    assert_eq!(error.expected.as_deref(), Some(replacement), "{source}");
    let help = error.help.as_deref().unwrap_or_default();
    assert!(
        help.contains(&format!("FPAS writes `{replacement}`")),
        "{source}: {help}"
    );

    let source = format!("program P;\n\n{declarations}\n\n{LATER_ERROR}");
    let errors = parser_errors(&source);
    assert_eq!(errors.len(), 2, "{source}: {errors:#?}");
    let later_line = source.lines().count() - 1;
    assert_eq!(
        errors[1].span.expect("source span").line() as usize,
        later_line,
        "{source}: {errors:#?}"
    );
}

/// Parser errors of a source that may be a unit.
pub(super) fn compilation_unit_errors(source: &str) -> Vec<Diagnostic> {
    let (_, diagnostics) = parse_compilation_unit_with_errors(source);
    diagnostics
        .iter()
        .filter_map(ParseDiagnostic::as_parser_error)
        .cloned()
        .collect()
}
