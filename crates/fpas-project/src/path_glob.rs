//! Filesystem glob expansion that preserves non-UTF-8 root paths.

use glob::{MatchOptions, Pattern, glob};
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Failure while parsing or walking a filesystem glob.
#[derive(Debug)]
pub enum PathGlobError {
    /// The textual glob pattern is invalid.
    InvalidPattern(glob::PatternError),
    /// A directory or entry required for expansion could not be read.
    ReadPath {
        /// Path whose metadata or entries could not be read.
        path: PathBuf,
        /// Underlying filesystem error.
        source: io::Error,
    },
}

impl fmt::Display for PathGlobError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPattern(error) => error.fmt(formatter),
            Self::ReadPath { path, source } => {
                write!(formatter, "cannot read `{}`: {source}", path.display())
            }
        }
    }
}

impl Error for PathGlobError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidPattern(error) => Some(error),
            Self::ReadPath { source, .. } => Some(source),
        }
    }
}

/// Expands `pattern` relative to `root` without converting `root` to UTF-8.
pub fn expand_path_glob(root: &Path, pattern: &str) -> Result<Vec<PathBuf>, PathGlobError> {
    let resolved = root.join(pattern);
    if let Some(pattern_text) = resolved.to_str() {
        return glob(pattern_text)
            .map_err(PathGlobError::InvalidPattern)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| PathGlobError::ReadPath {
                path: error.path().to_path_buf(),
                source: error.into(),
            });
    }

    expand_below_non_utf8_root(root, pattern)
}

fn expand_below_non_utf8_root(root: &Path, pattern: &str) -> Result<Vec<PathBuf>, PathGlobError> {
    let pattern_path = Path::new(pattern);
    let (walk_root, relative_pattern) = split_literal_prefix(root, pattern_path);
    let relative_pattern = relative_pattern
        .to_str()
        .ok_or_else(|| PathGlobError::ReadPath {
            path: relative_pattern.clone(),
            source: io::Error::new(
                io::ErrorKind::InvalidInput,
                "glob pattern was not valid UTF-8",
            ),
        })?;
    let matcher = Pattern::new(relative_pattern).map_err(PathGlobError::InvalidPattern)?;
    let options = MatchOptions {
        case_sensitive: true,
        require_literal_separator: true,
        require_literal_leading_dot: false,
    };
    let mut pending = vec![walk_root.clone()];
    let mut matches = Vec::new();

    while let Some(directory) = pending.pop() {
        let entries = fs::read_dir(&directory).map_err(|source| PathGlobError::ReadPath {
            path: directory.clone(),
            source,
        })?;
        for entry in entries {
            let entry = entry.map_err(|source| PathGlobError::ReadPath {
                path: directory.clone(),
                source,
            })?;
            let path = entry.path();
            let relative =
                path.strip_prefix(&walk_root)
                    .map_err(|error| PathGlobError::ReadPath {
                        path: path.clone(),
                        source: io::Error::other(error),
                    })?;
            if matcher.matches_path_with(relative, options) {
                matches.push(path.clone());
            }
            let file_type = entry
                .file_type()
                .map_err(|source| PathGlobError::ReadPath {
                    path: path.clone(),
                    source,
                })?;
            if file_type.is_dir() {
                pending.push(path);
            }
        }
    }

    matches.sort();
    Ok(matches)
}

fn split_literal_prefix(root: &Path, pattern: &Path) -> (PathBuf, PathBuf) {
    let mut walk_root = root.to_path_buf();
    let mut relative_pattern = PathBuf::new();
    let mut found_metacharacter = false;

    for component in pattern.components() {
        let text = component.as_os_str().to_string_lossy();
        found_metacharacter |= text
            .chars()
            .any(|character| matches!(character, '*' | '?' | '[' | ']'));
        if found_metacharacter {
            relative_pattern.push(component);
        } else {
            walk_root.push(component);
        }
    }

    (walk_root, relative_pattern)
}
