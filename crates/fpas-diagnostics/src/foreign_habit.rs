//! Shared shape of diagnostics for spellings from other languages (Delphi, Free Pascal, Ada,
//! C family).
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings)

use crate::Diagnostic;

/// A recognized foreign spelling and the FPAS text that replaces it.
///
/// Every stage builds its habit diagnostics through this type, so the help text keeps one shape
/// and the replacement stays available as data in the diagnostic's `expected` and `found`
/// fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForeignHabit<'a> {
    /// The foreign text as written, covered by the diagnostic span; stages without source text
    /// abbreviate parts with `…`.
    pub found: &'a str,
    /// The FPAS text that replaces `found`.
    pub replacement: &'a str,
    /// One sentence on why the foreign form does not apply in FPAS.
    pub explanation: &'a str,
}

impl ForeignHabit<'_> {
    /// Returns the help text: the explanation followed by the FPAS form.
    #[must_use]
    pub fn help(&self) -> String {
        format!("{} FPAS writes `{}`.", self.explanation, self.replacement)
    }

    /// Sets the help text and the `expected`/`found` pair of `diagnostic`.
    #[must_use]
    pub fn attach(&self, mut diagnostic: Diagnostic) -> Diagnostic {
        diagnostic.help = Some(self.help());
        diagnostic.with_expected_found(self.replacement, self.found)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codes::PARSE_EXPECTED_TOKEN;

    #[test]
    fn attach_sets_help_and_replacement_data() {
        let habit = ForeignHabit {
            found: "Result := 1",
            replacement: "return 1",
            explanation: "FPAS has no implicit `Result` variable.",
        };
        let diagnostic = habit.attach(Diagnostic::error_without_source(
            PARSE_EXPECTED_TOKEN,
            "message",
            None,
        ));
        assert_eq!(
            diagnostic.help.as_deref(),
            Some("FPAS has no implicit `Result` variable. FPAS writes `return 1`.")
        );
        assert_eq!(diagnostic.expected.as_deref(), Some("return 1"));
        assert_eq!(diagnostic.found.as_deref(), Some("Result := 1"));
    }
}
