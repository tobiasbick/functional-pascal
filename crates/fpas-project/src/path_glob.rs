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
    expand_selected(root, pattern, &[])
}

pub(crate) fn expand_selected(
    root: &Path,
    pattern: &str,
    exclude: &[String],
) -> Result<Vec<PathBuf>, PathGlobError> {
    if let Some(paths) = crate::source_inputs::glob(root, pattern, exclude) {
        return Ok(paths);
    }
    let paths = expand_disk_selected(root, pattern, exclude)?;
    crate::source_inputs::retain_glob(root, pattern, exclude, &paths);
    Ok(paths)
}

pub(crate) fn expand_disk_selected(
    root: &Path,
    pattern: &str,
    exclude: &[String],
) -> Result<Vec<PathBuf>, PathGlobError> {
    if !crate::source_inputs::active() {
        return expand_disk(root, pattern);
    }
    let exclusions = exclude
        .iter()
        .map(|entry| {
            Pattern::new(&if Path::new(entry).is_absolute() {
                entry.replace('\\', "/")
            } else {
                format!(
                    "{}/{}",
                    Pattern::escape(&root.to_string_lossy().replace('\\', "/")),
                    entry.replace('\\', "/")
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(PathGlobError::InvalidPattern)?;
    expand_walk(root, pattern, |path| {
        let text = path.to_string_lossy().replace('\\', "/");
        !exclusions.iter().any(|p| {
            p.matches_with(
                &text,
                MatchOptions {
                    case_sensitive: true,
                    require_literal_separator: true,
                    require_literal_leading_dot: false,
                },
            )
        })
    })
}

pub(crate) fn expand_disk(root: &Path, pattern: &str) -> Result<Vec<PathBuf>, PathGlobError> {
    if crate::source_inputs::active() {
        return expand_below_non_utf8_root(root, pattern);
    }
    let pattern_path = Path::new(pattern);
    let resolved = root.join(pattern_path);
    if let Some(pattern_text) = resolved.to_str() {
        let pattern_text = if pattern_path.is_absolute() {
            pattern_text.to_owned()
        } else {
            let Some(root_text) = root.to_str() else {
                return expand_below_non_utf8_root(root, pattern);
            };
            Path::new(&Pattern::escape(root_text))
                .join(pattern_path)
                .to_string_lossy()
                .into_owned()
        };
        let mut paths = Vec::new();
        for entry in glob(&pattern_text).map_err(PathGlobError::InvalidPattern)? {
            let path = entry.map_err(|error| PathGlobError::ReadPath {
                path: error.path().to_path_buf(),
                source: error.into(),
            })?;
            reserve(&path)?;
            paths.push(path);
        }
        paths.sort();
        return Ok(paths);
    }

    expand_below_non_utf8_root(root, pattern)
}

pub(crate) fn has_match(root: &Path, pattern: &str) -> Result<bool, PathGlobError> {
    let mut any = false;
    expand_walk(root, pattern, |_| {
        any = true;
        false
    })?;
    Ok(any)
}

fn expand_below_non_utf8_root(root: &Path, pattern: &str) -> Result<Vec<PathBuf>, PathGlobError> {
    expand_walk(root, pattern, |_| true)
}

fn expand_walk(
    root: &Path,
    pattern: &str,
    mut retain: impl FnMut(&Path) -> bool,
) -> Result<Vec<PathBuf>, PathGlobError> {
    let pattern_path = Path::new(pattern);
    let (walk_root, relative_pattern) = split_literal_prefix(root, pattern_path);
    if relative_pattern.as_os_str().is_empty() {
        return if walk_root.exists() && retain(&walk_root) {
            reserve(&walk_root)?;
            Ok(vec![walk_root])
        } else {
            Ok(Vec::new())
        };
    }
    if !walk_root.exists() {
        return Ok(Vec::new());
    }
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
    let mut pending = vec![(walk_root.clone(), Vec::new())];
    let mut matches = Vec::new();
    let mut visited_count = 0;
    let recursive = relative_pattern.contains("**");
    let depth = Path::new(relative_pattern).components().count();

    while let Some((directory, mut ancestors)) = pending.pop() {
        let canonical = fs::canonicalize(&directory).map_err(|source| PathGlobError::ReadPath {
            path: directory.clone(),
            source,
        })?;
        if ancestors.contains(&canonical) {
            continue;
        }
        ancestors.push(canonical);
        visited_count += 1;
        if crate::source_inputs::active()
            && (visited_count > crate::source_inputs::MAX_FILES || ancestors.len() > 256)
        {
            return Err(PathGlobError::ReadPath {
                path: directory,
                source: crate::source_inputs::limit_error(),
            });
        }
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
            if matcher.matches_path_with(relative, options) && retain(&path) {
                if crate::source_inputs::active()
                    && matches.len() >= crate::source_inputs::MAX_FILES
                {
                    return Err(PathGlobError::ReadPath {
                        path,
                        source: crate::source_inputs::limit_error(),
                    });
                }
                reserve(&path)?;
                matches.push(path.clone());
            }
            let descend = recursive || relative.components().count() < depth;
            if descend && fs::metadata(&path).is_ok_and(|m| m.is_dir()) {
                if crate::source_inputs::active()
                    && pending.len() >= crate::source_inputs::MAX_FILES
                {
                    return Err(PathGlobError::ReadPath {
                        path,
                        source: crate::source_inputs::limit_error(),
                    });
                }
                pending.push((path, ancestors.clone()));
            }
        }
    }

    matches.sort();
    Ok(matches)
}

fn reserve(path: &Path) -> Result<(), PathGlobError> {
    crate::source_inputs::select(path).map_err(|source| PathGlobError::ReadPath {
        path: path.into(),
        source,
    })
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
