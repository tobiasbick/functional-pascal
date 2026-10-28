//! Boolean text shared by Std.Text.Format and Std.Text.Parse.
//!
//! **Documentation:** `docs/pascal/std/text/parse.md`.

/// Parses true or false after trimming whitespace, ignoring ASCII case.
pub(crate) fn parse_bool_text(text: &str) -> Option<bool> {
    if text.trim().eq_ignore_ascii_case("true") {
        Some(true)
    } else if text.trim().eq_ignore_ascii_case("false") {
        Some(false)
    } else {
        None
    }
}
