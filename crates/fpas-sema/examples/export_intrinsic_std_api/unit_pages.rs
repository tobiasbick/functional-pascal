//! Handbook page and summary per intrinsic unit, for the unit's documentation comment.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

/// The handbook page whose title is exactly the unit name.
pub(super) struct UnitPage {
    /// Repository-relative path with forward slashes.
    pub(super) path: String,
    /// First sentence of the page's opening paragraph.
    pub(super) summary: String,
}

/// Maps each unit to the page titled `# Std.Unit` or ``# `Std.Unit` ``.
pub(super) fn unit_pages(
    repository: &Path,
    paths: &[std::path::PathBuf],
    units: &[&str],
) -> io::Result<HashMap<String, UnitPage>> {
    let mut pages = HashMap::new();
    for path in paths {
        let source = fs::read_to_string(path)?;
        let mut lines = source.lines();
        let Some(title) = lines.by_ref().find_map(|line| line.strip_prefix("# ")) else {
            continue;
        };
        let title = title.trim().trim_matches('`');
        let Some(unit) = units.iter().find(|unit| **unit == title) else {
            continue;
        };
        let paragraph = lines
            .skip_while(|line| line.trim().is_empty())
            .take_while(|line| !line.trim().is_empty())
            .map(str::trim)
            .collect::<Vec<_>>()
            .join(" ");
        let relative = path
            .strip_prefix(repository)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        pages.insert(
            (*unit).to_owned(),
            UnitPage {
                path: relative,
                summary: first_sentence(&paragraph),
            },
        );
    }
    Ok(pages)
}

/// Renders the comment placed directly before `unit Std.Name;`.
pub(super) fn unit_comment(page: &UnitPage) -> String {
    format!(
        "// {}\n//\n// Handbook: `{}`.\n",
        page.summary.trim_end_matches('.').to_owned() + ".",
        page.path
    )
}

fn first_sentence(paragraph: &str) -> String {
    paragraph
        .find(". ")
        .map_or(paragraph, |end| &paragraph[..=end])
        .trim()
        .to_owned()
}
