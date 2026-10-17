//! Built-in routine names from other languages and their FPAS forms.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings)

use fpas_diagnostics::ForeignHabit;

/// Lowercase foreign built-in name and the habit it stands for. One line per entry.
const FOREIGN_BUILTINS: &[(&str, ForeignHabit<'static>)] = &[(
    "exit",
    ForeignHabit {
        found: "Exit",
        replacement: "return Value;",
        explanation: "Delphi `Exit` does not exist; leave a routine with `return`, without a value in a procedure.",
    },
)];

/// Returns the help for an unresolved unqualified name that is a known foreign built-in.
pub(crate) fn foreign_builtin_hint(name: &str) -> Option<String> {
    FOREIGN_BUILTINS
        .iter()
        .find(|(foreign, _)| name.eq_ignore_ascii_case(foreign))
        .map(|(_, habit)| habit.help())
}
