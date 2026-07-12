//! Optional golden stdout files (`*.expect.stdout`) beside test sources.
//!
//! **Documentation:** [`docs/pascal/std/testing/test.md`](../../../docs/pascal/std/testing/test.md)

use std::fs;
use std::path::{Path, PathBuf};

/// Returns the default golden stdout path for a test file (`*_test.fpas` → `*.expect.stdout`).
pub(super) fn expect_stdout_path_for_test(test_path: &Path) -> PathBuf {
    test_path.with_extension("expect.stdout")
}

/// Loads expected stdout lines when a sidecar file exists.
pub(super) fn load_expect_stdout(path: &Path) -> Result<Option<Vec<String>>, String> {
    if !path.is_file() {
        return Ok(None);
    }
    let text = fs::read_to_string(path).map_err(|error| {
        format!(
            "Error reading expected stdout `{}`: {error}\n  help: Golden stdout files use `<test>.expect.stdout`.",
            path.display()
        )
    })?;
    Ok(Some(parse_expect_stdout_lines(&text)))
}

pub(super) fn parse_expect_stdout_lines(text: &str) -> Vec<String> {
    let normalized = text.replace("\r\n", "\n");
    let trimmed = normalized.strip_suffix('\n').unwrap_or(normalized.as_str());
    if trimmed.is_empty() {
        return Vec::new();
    }
    trimmed
        .split('\n')
        .map(|line| line.trim_end_matches('\r').to_string())
        .collect()
}

/// Compares captured VM stdout lines against an optional golden file beside `test_path`.
pub(super) fn compare_stdout(test_path: &Path, actual: &[String]) -> Result<(), String> {
    let expect_path = expect_stdout_path_for_test(test_path);
    let Some(expected) = load_expect_stdout(&expect_path)? else {
        return Ok(());
    };
    if actual == expected.as_slice() {
        return Ok(());
    }
    Err(format_stdout_mismatch(&expect_path, &expected, actual))
}

fn format_stdout_mismatch(expect_path: &Path, expected: &[String], actual: &[String]) -> String {
    let mut message = format!(
        "stdout mismatch (see `{}`).\n  help: Update the golden file if the new output is correct.",
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
    fn parse_expect_stdout_lines_trims_trailing_newline() {
        assert_eq!(
            parse_expect_stdout_lines("a\nb\n"),
            vec!["a".to_string(), "b".to_string()]
        );
        assert_eq!(
            parse_expect_stdout_lines("a\r\nb\r\n"),
            vec!["a".to_string(), "b".to_string()]
        );
    }

    #[test]
    fn compare_stdout_passes_when_file_matches() {
        let cwd = create_temp_dir("expect-stdout-pass");
        let test_path = cwd.join("echo_test.fpas");
        write_text(&test_path, "program T; begin end.");
        write_text(&expect_stdout_path_for_test(&test_path), "Hello\nWorld\n");

        assert!(compare_stdout(&test_path, &["Hello".into(), "World".into()]).is_ok());
    }

    #[test]
    fn compare_stdout_fails_with_line_diff_hint() {
        let cwd = create_temp_dir("expect-stdout-fail");
        let test_path = cwd.join("echo_test.fpas");
        write_text(&test_path, "program T; begin end.");
        write_text(&expect_stdout_path_for_test(&test_path), "Hello\n");

        let err = compare_stdout(&test_path, &["Hi".into()]).expect_err("mismatch must fail");
        assert!(err.contains("stdout mismatch"));
        assert!(err.contains("expected (1 lines)"));
        assert!(err.contains("actual (1 lines)"));
    }
}
