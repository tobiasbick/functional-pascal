//! Optional golden screen files (`*.expect.screen`) beside TUI test sources.
//!
//! **Documentation:** [`docs/future/test-framework/runner.md`](../../../docs/future/test-framework/runner.md)

use std::fs;
use std::path::{Path, PathBuf};

use super::expect_stdout::parse_expect_stdout_lines;

/// Returns the default golden screen path for a test file (`*_test.fpas` → `*.expect.screen`).
pub(super) fn expect_screen_path_for_test(test_path: &Path) -> PathBuf {
    test_path.with_extension("expect.screen")
}

/// Loads expected screen lines when a sidecar file exists.
pub(super) fn load_expect_screen(path: &Path) -> Result<Option<Vec<String>>, String> {
    if !path.is_file() {
        return Ok(None);
    }
    let text = fs::read_to_string(path).map_err(|error| {
        format!(
            "Error reading expected screen `{}`: {error}\n  help: Golden screen files use `<test>.expect.screen`.",
            path.display()
        )
    })?;
    Ok(Some(parse_expect_stdout_lines(&text)))
}

/// Compares compact CRT screen lines against an optional golden file beside `test_path`.
pub(super) fn compare_screen(test_path: &Path, actual: &[String]) -> Result<(), String> {
    let expect_path = expect_screen_path_for_test(test_path);
    let Some(expected) = load_expect_screen(&expect_path)? else {
        return Ok(());
    };
    if actual == expected.as_slice() {
        return Ok(());
    }
    Err(format_screen_mismatch(&expect_path, &expected, actual))
}

fn format_screen_mismatch(expect_path: &Path, expected: &[String], actual: &[String]) -> String {
    let mut message = format!(
        "screen mismatch (see `{}`).\n  help: Update the golden file if the painted layout is correct.",
        expect_path.display()
    );
    message.push_str(&format!("\n        expected ({} lines):", expected.len()));
    for line in expected {
        message.push_str(&format!("\n          {line}"));
    }
    message.push_str(&format!("\n        actual ({} lines):", actual.len()));
    for line in actual {
        message.push_str(&format!("\n          {line}"));
    }
    message
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{create_temp_dir, write_text};

    #[test]
    fn compare_screen_passes_when_file_matches() {
        let cwd = create_temp_dir("expect-screen-pass");
        let test_path = cwd.join("paint_test.fpas");
        write_text(&test_path, "program T; begin end.");
        write_text(&expect_screen_path_for_test(&test_path), "Hello TUI\n");

        assert!(compare_screen(&test_path, &["Hello TUI".into()]).is_ok());
    }
}
