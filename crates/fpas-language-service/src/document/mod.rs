//! Immutable source snapshots and full-text document storage.

mod line_index;
mod snapshot;
mod store;

use std::path::{Component, Path, PathBuf};

pub use line_index::{LineIndex, TextPosition, TextRange};
pub use snapshot::{DocumentSnapshot, SourceVersion};
pub use store::DocumentStore;

pub(crate) fn normalized_path(path: &Path) -> PathBuf {
    if let Ok(canonical) = std::fs::canonicalize(path) {
        return platform_normalize(canonical);
    }

    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    };
    platform_normalize(lexical_normalize(&absolute))
}

fn lexical_normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

#[cfg(windows)]
fn platform_normalize(path: PathBuf) -> PathBuf {
    let display = path.to_string_lossy();
    if let Some(rest) = display.strip_prefix(r"\\?\UNC\") {
        return PathBuf::from(format!(r"\\{rest}"));
    }
    if let Some(rest) = display.strip_prefix(r"\\?\") {
        return PathBuf::from(rest);
    }
    path
}

#[cfg(not(windows))]
fn platform_normalize(path: PathBuf) -> PathBuf {
    path
}
