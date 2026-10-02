//! Internal source-entry identities, separate from declared routines.
//!
//! **Documentation:** `docs/pascal/program-structure/units.md`

const PROGRAM_PREFIX: &str = "$entry.";
const UNIT_PREFIX: &str = "$unit.";

/// Return a canonical entry name that cannot be a source identifier.
///
/// The source name remains available through [`entry_source_name`].
#[must_use]
pub fn program_entry_name(source_name: &str) -> String {
    format!("{PROGRAM_PREFIX}{}", source_name.to_ascii_lowercase())
}

/// Return a canonical unit-initializer name distinct from program entries.
///
/// **Documentation:** `docs/pascal/program-structure/units.md`
#[must_use]
pub fn unit_initializer_name(source_name: &str) -> String {
    format!("{UNIT_PREFIX}{}", source_name.to_ascii_lowercase())
}

/// Recover the canonical source name from a program entry or unit initializer.
///
/// Ordinary routine names return `None`. Debugger frames and recording
/// envelopes display this source name rather than the internal identity.
///
/// **Documentation:** `docs/pascal/tools/debugger.md`
#[must_use]
pub fn entry_source_name(name: &str) -> Option<&str> {
    name.strip_prefix(PROGRAM_PREFIX)
        .or_else(|| name.strip_prefix(UNIT_PREFIX))
}
