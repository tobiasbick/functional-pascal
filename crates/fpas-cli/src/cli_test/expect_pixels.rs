//! Optional golden pixel spot-check files (`*.expect.pixels`) for headless graph tests.
//!
//! **Documentation:** [`docs/pascal/std/testing/test.md`](../../../docs/pascal/std/testing/test.md)

use std::fs;
use std::path::{Path, PathBuf};

use fpas_std::UploadedFrame;

/// One expected pixel color at a graph surface coordinate.
#[derive(Debug, Clone, PartialEq, Eq)]
struct PixelExpectation {
    x: i64,
    y: i64,
    color: u32,
}

/// Returns the default golden pixel path for a test file (`*_test.fpas` → `*.expect.pixels`).
pub(super) fn expect_pixels_path_for_test(test_path: &Path) -> PathBuf {
    test_path.with_extension("expect.pixels")
}

/// Loads expected pixel spot checks when a sidecar file exists.
fn load_expect_pixels(path: &Path) -> Result<Option<Vec<PixelExpectation>>, String> {
    if !path.is_file() {
        return Ok(None);
    }
    let text = fs::read_to_string(path).map_err(|error| {
        format!(
            "Error reading expected pixels `{}`: {error}\n  help: Golden pixel files use `<test>.expect.pixels`.",
            path.display()
        )
    })?;
    Ok(Some(parse_expect_pixels(&text, path)?))
}

/// Compares the last headless graph frame against optional spot checks beside `test_path`.
pub(super) fn compare_pixels(test_path: &Path, frame: &UploadedFrame) -> Result<(), String> {
    let expect_path = expect_pixels_path_for_test(test_path);
    let Some(expectations) = load_expect_pixels(&expect_path)? else {
        return Ok(());
    };
    if expectations.is_empty() {
        return Ok(());
    }

    let mut mismatches = Vec::new();
    for expectation in expectations {
        match pixel_at(frame, expectation.x, expectation.y) {
            Ok(actual) if actual == expectation.color => {}
            Ok(actual) => mismatches.push(format!(
                "({},{}) expected 0x{expected:08X}, got 0x{actual:08X}",
                expectation.x,
                expectation.y,
                expected = expectation.color,
            )),
            Err(message) => mismatches.push(message),
        }
    }

    if mismatches.is_empty() {
        return Ok(());
    }

    Err(format!(
        "pixel mismatch in `{}` ({} spot check(s) failed).\n  help: Update the golden file if the rendered frame is correct.\n        {}",
        expect_path.display(),
        mismatches.len(),
        mismatches.join("\n        "),
    ))
}

fn pixel_at(frame: &UploadedFrame, x: i64, y: i64) -> Result<u32, String> {
    let width = frame.width();
    let height = frame.height();
    if x < 0 || y < 0 || x >= width || y >= height {
        return Err(format!(
            "pixel ({x},{y}) is outside the {width}x{height} frame bounds"
        ));
    }
    let row = usize::try_from(y).unwrap_or(usize::MAX);
    let col = usize::try_from(x).unwrap_or(usize::MAX);
    let width_usize = usize::try_from(width).unwrap_or(usize::MAX);
    let index = row.saturating_mul(width_usize).saturating_add(col);
    frame
        .pixels()
        .get(index)
        .copied()
        .ok_or_else(|| format!("pixel ({x},{y}) is outside the presented frame buffer"))
}

fn parse_expect_pixels(text: &str, path: &Path) -> Result<Vec<PixelExpectation>, String> {
    let mut expectations = Vec::new();
    for (line_no, raw_line) in text.lines().enumerate() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.to_ascii_lowercase().starts_with("size ") {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 3 {
            return Err(format!(
                "Invalid pixel expectation at `{}` line {}: `{raw_line}`\n  help: Use `x y 0xRRGGBB` per line (optional `# size W H` header).",
                path.display(),
                line_no + 1,
            ));
        }

        let x: i64 = parts[0].parse().map_err(|_| {
            format!(
                "Invalid x coordinate at `{}` line {}: `{}`",
                path.display(),
                line_no + 1,
                parts[0],
            )
        })?;
        let y: i64 = parts[1].parse().map_err(|_| {
            format!(
                "Invalid y coordinate at `{}` line {}: `{}`",
                path.display(),
                line_no + 1,
                parts[1],
            )
        })?;
        let color = parse_hex_color(parts[2]).map_err(|message| {
            format!("{message} at `{}` line {}.", path.display(), line_no + 1,)
        })?;
        expectations.push(PixelExpectation { x, y, color });
    }
    Ok(expectations)
}

fn parse_hex_color(token: &str) -> Result<u32, String> {
    let token = token.trim().trim_start_matches('$');
    let hex = token
        .strip_prefix("0x")
        .or_else(|| token.strip_prefix("0X"))
        .unwrap_or(token);
    let value = u32::from_str_radix(hex, 16).map_err(|_| {
        format!(
            "expected a `$00RRGGBB` or `0xRRGGBB` color literal, got `{token}`\n  help: Example: `0 0 0x00020408`"
        )
    })?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_expect_pixels_reads_spot_checks() {
        let parsed = parse_expect_pixels(
            "# size 32 24\n0 0 0x00020408\n2 2 0x00FFFFFF\n",
            Path::new("demo.expect.pixels"),
        )
        .expect("parse");
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].color, 0x00020408);
        assert_eq!(parsed[1].color, 0x00FFFFFF);
    }

    #[test]
    fn parse_hex_color_accepts_dollar_and_hex_prefix() {
        assert_eq!(parse_hex_color("$00ABCDEF").expect("hex"), 0x00ABCDEF);
        assert_eq!(parse_hex_color("0x00020408").expect("hex"), 0x00020408);
    }
}
