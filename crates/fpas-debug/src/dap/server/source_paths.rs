//! DAP editor-path resolution against portable debugger source identities.

use std::collections::HashSet;

use crate::DebugSourceContent;

#[derive(Debug)]
pub(super) struct SourcePaths {
    portable: Vec<String>,
    aliases: Vec<(String, String)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct AmbiguousSourcePath;

impl SourcePaths {
    pub(super) fn new(portable: &[String], sources: &[DebugSourceContent]) -> Self {
        let mut portable = portable.to_vec();
        for source in sources {
            if !portable.contains(&source.path) {
                portable.push(source.path.clone());
            }
        }
        let aliases = sources
            .iter()
            .filter_map(|source| {
                source
                    .original_path
                    .as_ref()
                    .map(|original| (original.to_string_lossy().into_owned(), source.path.clone()))
            })
            .collect();
        Self { portable, aliases }
    }

    pub(super) fn resolve(&self, requested: &str) -> Result<String, AmbiguousSourcePath> {
        if let Some(resolved) = unique_matches(
            self.portable
                .iter()
                .filter(|candidate| paths_equal(requested, candidate))
                .cloned(),
        )? {
            return Ok(resolved);
        }
        if let Some(resolved) = unique_matches(
            self.aliases
                .iter()
                .filter(|(alias, _)| paths_equal(requested, alias))
                .map(|(_, portable)| portable.clone()),
        )? {
            return Ok(resolved);
        }
        if let Some(resolved) = unique_matches(
            self.portable
                .iter()
                .filter(|candidate| {
                    has_path_suffix(requested, candidate) || has_path_suffix(candidate, requested)
                })
                .cloned(),
        )? {
            return Ok(resolved);
        }
        if let Some(resolved) = unique_matches(
            self.portable
                .iter()
                .filter(|candidate| file_names_equal(requested, candidate))
                .cloned(),
        )? {
            return Ok(resolved);
        }
        Ok(requested.replace('\\', "/"))
    }
}

fn unique_matches(
    candidates: impl IntoIterator<Item = String>,
) -> Result<Option<String>, AmbiguousSourcePath> {
    let candidates = candidates.into_iter().collect::<HashSet<_>>();
    match candidates.len() {
        0 => Ok(None),
        1 => Ok(candidates.into_iter().next()),
        _ => Err(AmbiguousSourcePath),
    }
}

fn paths_equal(left: &str, right: &str) -> bool {
    compare_normalized(left, right, |left, right| left == right)
}

fn has_path_suffix(path: &str, suffix: &str) -> bool {
    compare_normalized(path, suffix, |path, suffix| {
        path == suffix || path.ends_with(&format!("/{suffix}"))
    })
}

fn file_names_equal(left: &str, right: &str) -> bool {
    compare_normalized(left, right, |left, right| {
        left.rsplit('/').next() == right.rsplit('/').next()
    })
}

fn compare_normalized(left: &str, right: &str, compare: impl FnOnce(&str, &str) -> bool) -> bool {
    let left = left.replace('\\', "/");
    let right = right.replace('\\', "/");
    if cfg!(windows) || is_windows_style(&left) || is_windows_style(&right) {
        compare(&left.to_ascii_lowercase(), &right.to_ascii_lowercase())
    } else {
        compare(&left, &right)
    }
}

fn is_windows_style(path: &str) -> bool {
    let bytes = path.as_bytes();
    path.starts_with("//")
        || (bytes.len() >= 3
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && bytes[2] == b'/')
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn source(path: &str, original: Option<&str>) -> DebugSourceContent {
        DebugSourceContent {
            path: path.to_string(),
            original_path: original.map(PathBuf::from),
            content: String::new(),
        }
    }

    #[test]
    fn windows_aliases_ignore_case_and_separator_style() {
        let paths = SourcePaths::new(
            &["src/Main.fpas".to_string()],
            &[source("src/Main.fpas", Some("C:\\Work\\Src\\Main.fpas"))],
        );
        assert_eq!(
            paths.resolve("c:/work/src/main.fpas"),
            Ok("src/Main.fpas".to_string())
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn native_paths_remain_case_sensitive() {
        let paths = SourcePaths::new(
            &["src/Main.fpas".to_string()],
            &[source("src/Main.fpas", Some("/work/src/Main.fpas"))],
        );
        assert_eq!(
            paths.resolve("/work/src/main.fpas"),
            Ok("/work/src/main.fpas".to_string())
        );
    }

    #[test]
    fn ambiguous_suffix_is_rejected() {
        let paths = SourcePaths::new(
            &["left/main.fpas".to_string(), "right/main.fpas".to_string()],
            &[],
        );
        assert_eq!(
            paths.resolve("workspace/main.fpas"),
            Err(AmbiguousSourcePath)
        );
    }
}
