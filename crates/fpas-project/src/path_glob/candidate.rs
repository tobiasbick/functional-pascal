//! Predict new membership through the same directory walk and lexical exclusion rules.

use super::*;

/// Whether creating a file would change this recorded glob's membership, including exclusions.
/// The destination's parent must exist; no file or directory is created by this check.
pub fn would_select_path(
    root: &Path,
    pattern: &str,
    exclude: &[String],
    destination: &Path,
) -> Result<bool, PathGlobError> {
    let _bounded = if crate::source_inputs::active() {
        None
    } else {
        Some(
            crate::source_inputs::SourceInputs::begin(|| false).map_err(|source| {
                PathGlobError::ReadPath {
                    path: root.into(),
                    source,
                }
            })?,
        )
    };
    let destination =
        std::path::absolute(destination).map_err(|source| PathGlobError::ReadPath {
            path: destination.into(),
            source,
        })?;
    let physical = canonical_candidate(&destination)?;
    let exclusions = exclusions(root, exclude)?;
    let (walk_root, relative_pattern) = split_literal_prefix(root, Path::new(pattern));
    if relative_pattern.as_os_str().is_empty() {
        return Ok(
            canonical_candidate(&walk_root)? == physical && !excluded(&exclusions, &walk_root)
        );
    }
    let matcher =
        Pattern::new(&relative_pattern.to_string_lossy()).map_err(PathGlobError::InvalidPattern)?;
    let name = destination
        .file_name()
        .ok_or_else(|| PathGlobError::ReadPath {
            path: destination.clone(),
            source: io::Error::new(
                io::ErrorKind::InvalidInput,
                "Artifact destination must name a file.",
            ),
        })?;
    let parent = physical.parent();
    let mut selected = false;
    // A symlink may expose the destination through several lexical paths. Exclusions are
    // lexical, so physical containment alone cannot predict any of those memberships.
    walk_directories(
        root,
        pattern,
        |_| false,
        |directory, canonical| {
            if Some(canonical) == parent {
                let candidate = directory.join(name);
                if candidate
                    .strip_prefix(&walk_root)
                    .is_ok_and(|relative| matcher.matches_path_with(relative, match_options()))
                    && !excluded(&exclusions, &candidate)
                {
                    selected = true;
                }
            }
            Ok(())
        },
    )?;
    Ok(selected)
}

fn canonical_candidate(path: &Path) -> Result<PathBuf, PathGlobError> {
    let mut ancestor = path;
    let mut missing = Vec::new();
    loop {
        match fs::canonicalize(ancestor) {
            Ok(mut physical) => {
                for component in missing.into_iter().rev() {
                    physical.push(component);
                }
                return Ok(physical);
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                if let Some((name, parent)) = ancestor.file_name().zip(ancestor.parent()) {
                    missing.push(name);
                    ancestor = parent;
                } else {
                    return Err(PathGlobError::ReadPath {
                        path: path.into(),
                        source: error,
                    });
                }
            }
            Err(error) => {
                return Err(PathGlobError::ReadPath {
                    path: path.into(),
                    source: error,
                });
            }
        }
    }
}
