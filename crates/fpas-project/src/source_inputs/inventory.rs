//! Owned source inputs, independent of the request thread and its lifetime.

use super::{INPUTS, MAX_FILE_BYTES, SourceInputs, active, bounded_read};
use std::{fs, io, path::PathBuf, sync::Arc};

/// Exact bytes of one canonical physical input.
#[derive(Clone, Debug)]
pub struct InputFile {
    /// Canonical physical path.
    pub path: PathBuf,
    /// Exact bytes retained by the source loader.
    pub bytes: Arc<[u8]>,
    /// An editor owns these bytes; its version is checked by the editor adapter.
    pub is_overlay: bool,
}

/// A discovered spelling and the physical identity to which it was bound.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputAlias {
    /// Absolute spelling used during discovery or loading.
    pub lexical: PathBuf,
    /// Filesystem identity observed for that spelling.
    pub physical: PathBuf,
}

/// One manifest selection, including empty selections and exclusions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputGlob {
    /// Directory against which the pattern was resolved.
    pub root: PathBuf,
    /// Inclusion pattern from project discovery.
    pub pattern: String,
    /// Ordered exclusions applied before admitting inputs.
    pub exclude: Vec<String>,
    /// Selected lexical paths in stable order.
    pub members: Vec<PathBuf>,
}

/// Immutable read set. It is validation data, never permission to write its paths.
#[derive(Clone, Debug)]
pub struct InputInventory {
    files: Vec<InputFile>,
    aliases: Vec<InputAlias>,
    selected: Vec<PathBuf>,
    globs: Vec<InputGlob>,
}

impl InputInventory {
    /// Canonical files in path order.
    pub fn files(&self) -> &[InputFile] {
        &self.files
    }
    /// Lexical-to-physical bindings in lexical path order.
    pub fn aliases(&self) -> &[InputAlias] {
        &self.aliases
    }
    /// All admitted physical identities, including discovery-only directories.
    pub fn selected(&self) -> &[PathBuf] {
        &self.selected
    }
    /// Manifest selections and their observed membership.
    pub fn globs(&self) -> &[InputGlob] {
        &self.globs
    }

    /// Rechecks aliases, bytes and membership, including after the source guard is dropped.
    /// An IO error means verification failed, never permission to proceed.
    pub fn verify(&self) -> io::Result<bool> {
        // Discovery must retain its bounded/exclusion-aware behavior without a live request.
        let _guard = if active() {
            None
        } else {
            Some(SourceInputs::begin(|| false)?)
        };
        let _verification = Verification::begin();
        for alias in &self.aliases {
            check_cancelled()?;
            if fs::canonicalize(&alias.lexical)? != alias.physical {
                return Ok(false);
            }
        }
        for file in &self.files {
            check_cancelled()?;
            if file.is_overlay {
                continue;
            }
            if bounded_read(&file.path, MAX_FILE_BYTES)?.as_slice() != file.bytes.as_ref() {
                return Ok(false);
            }
        }
        for glob in &self.globs {
            check_cancelled()?;
            let current =
                crate::path_glob::expand_disk_selected(&glob.root, &glob.pattern, &glob.exclude)
                    .map_err(io::Error::other)?
                    .iter()
                    .map(std::path::absolute)
                    .collect::<io::Result<Vec<_>>>()?;
            if current != glob.members {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

pub(super) fn capture() -> io::Result<InputInventory> {
    INPUTS.with_borrow(|slot| {
        let inputs = slot
            .as_ref()
            .ok_or_else(|| io::Error::other("Missing source snapshot."))?;
        if inputs.reason.is_some() {
            return Err(io::Error::other(
                "Cannot export an incomplete source inventory.",
            ));
        }
        for path in &inputs.selected {
            if fs::metadata(path)?.is_file() && !inputs.files.contains_key(path) {
                return Err(io::Error::other("Cannot export an unread source input."));
            }
        }
        Ok(())
    })?;
    capture_read_set()
}

pub(super) fn capture_read_set() -> io::Result<InputInventory> {
    INPUTS.with_borrow(|slot| {
        let inputs = slot
            .as_ref()
            .ok_or_else(|| io::Error::other("Missing source snapshot."))?;
        Ok(InputInventory {
            files: inputs
                .files
                .iter()
                .map(|(path, bytes)| InputFile {
                    path: path.clone(),
                    bytes: Arc::clone(bytes),
                    is_overlay: inputs.overlays.contains_key(path),
                })
                .collect(),
            aliases: inputs
                .aliases
                .iter()
                .map(|(lexical, physical)| InputAlias {
                    lexical: lexical.clone(),
                    physical: physical.clone(),
                })
                .collect(),
            selected: inputs.selected.iter().cloned().collect(),
            globs: inputs
                .globs
                .iter()
                .map(|((root, pattern, exclude), members)| {
                    Ok(InputGlob {
                        root: std::path::absolute(root)?,
                        pattern: pattern.clone(),
                        exclude: exclude.clone(),
                        members: members
                            .iter()
                            .map(std::path::absolute)
                            .collect::<io::Result<_>>()?,
                    })
                })
                .collect::<io::Result<_>>()?,
        })
    })
}

fn check_cancelled() -> io::Result<()> {
    INPUTS.with_borrow_mut(|slot| match slot {
        Some(inputs) => inputs.check(),
        None => Ok(()),
    })
}

struct Verification(bool);
impl Verification {
    fn begin() -> Self {
        Self(INPUTS.with_borrow_mut(|slot| {
            slot.as_mut()
                .is_some_and(|i| std::mem::replace(&mut i.verifying, true))
        }))
    }
}
impl Drop for Verification {
    fn drop(&mut self) {
        INPUTS.with_borrow_mut(|slot| {
            if let Some(i) = slot {
                i.verifying = self.0;
            }
        });
    }
}
