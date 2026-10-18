//! Bounded, immutable disk inputs for one synchronous source-tool request.
//!
//! The guard is thread confined: project loading and document loading on its thread
//! share the same bytes. Ordinary compiler/editor callers retain live disk reads.

use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{self, Read},
    marker::PhantomData,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

#[cfg(test)]
mod tests;

/// Maximum canonical input files in a source request.
pub const MAX_FILES: usize = 4096;
/// Maximum bytes in one source or manifest.
pub const MAX_FILE_BYTES: usize = 8 * 1024 * 1024;
/// Maximum bytes across sources and manifests.
pub const MAX_TOTAL_BYTES: usize = 64 * 1024 * 1024;

thread_local! { static INPUTS: RefCell<Option<Inputs>> = const { RefCell::new(None) }; }

struct Inputs {
    files: BTreeMap<PathBuf, Arc<[u8]>>,
    selected: BTreeSet<PathBuf>,
    aliases: BTreeMap<PathBuf, PathBuf>,
    globs: BTreeMap<(PathBuf, String, Vec<String>), Vec<PathBuf>>,
    bytes: usize,
    reason: Option<&'static str>,
    verifying: bool,
    cancelled: Box<dyn Fn() -> bool>,
}

/// Owns one thread's checked source snapshot, including manifest glob selections.
pub struct SourceInputs {
    _thread: PhantomData<Rc<()>>,
}

impl SourceInputs {
    /// Starts an isolated snapshot; nesting is rejected rather than mixing requests.
    pub fn begin(cancelled: impl Fn() -> bool + 'static) -> io::Result<Self> {
        INPUTS.with_borrow_mut(|slot| {
            if slot.is_some() {
                return Err(io::Error::other("A source snapshot is already active."));
            }
            *slot = Some(Inputs {
                files: BTreeMap::new(),
                selected: BTreeSet::new(),
                aliases: BTreeMap::new(),
                globs: BTreeMap::new(),
                bytes: 0,
                reason: None,
                verifying: false,
                cancelled: Box::new(cancelled),
            });
            Ok(Self {
                _thread: PhantomData,
            })
        })
    }

    /// Returns exact deduplicated bytes in canonical path order.
    pub fn files(&self) -> Vec<(PathBuf, Arc<[u8]>)> {
        INPUTS.with_borrow(|slot| {
            slot.as_ref()
                .map(|i| {
                    i.files
                        .iter()
                        .map(|(p, b)| (p.clone(), Arc::clone(b)))
                        .collect()
                })
                .unwrap_or_default()
        })
    }

    /// Returns a resource/cancellation omission recorded by a lower-level loader.
    pub fn reason(&self) -> Option<&'static str> {
        INPUTS.with_borrow(|slot| slot.as_ref().and_then(|i| i.reason))
    }

    /// Checks current bytes and glob membership without replacing the frozen inputs.
    pub fn verify(&self) -> io::Result<bool> {
        let aliases =
            INPUTS.with_borrow(|slot| slot.as_ref().map(|i| i.aliases.clone()).unwrap_or_default());
        for (path, identity) in aliases {
            if fs::canonicalize(path)? != identity {
                return Ok(false);
            }
        }
        for (path, bytes) in self.files() {
            if bounded_read(&path, MAX_FILE_BYTES)?.as_slice() != bytes.as_ref() {
                return Ok(false);
            }
        }
        let globs =
            INPUTS.with_borrow(|slot| slot.as_ref().map(|i| i.globs.clone()).unwrap_or_default());
        INPUTS.with_borrow_mut(|slot| {
            if let Some(i) = slot {
                i.verifying = true;
            }
        });
        let verified = (|| {
            for ((root, pattern, exclude), expected) in globs {
                let current = crate::path_glob::expand_disk_selected(&root, &pattern, &exclude)
                    .map_err(io::Error::other)?;
                if current != expected {
                    return Ok(false);
                }
            }
            Ok(true)
        })();
        INPUTS.with_borrow_mut(|slot| {
            if let Some(i) = slot {
                i.verifying = false;
            }
        });
        verified
    }
}

impl Drop for SourceInputs {
    fn drop(&mut self) {
        INPUTS.with_borrow_mut(|slot| *slot = None);
    }
}

/// Whether this thread is performing a bounded, recoverable source query.
pub fn active() -> bool {
    INPUTS.with_borrow(Option::is_some)
}

/// Reads exact frozen bytes when a source snapshot is active, otherwise the live file.
pub fn read(path: &Path) -> io::Result<Vec<u8>> {
    if !active() {
        return fs::read(path);
    }
    select(path)?;
    let path = fs::canonicalize(path)?;
    INPUTS.with_borrow_mut(|slot| {
        let Some(inputs) = slot.as_mut() else {
            return Err(io::Error::other("Missing source snapshot."));
        };
        inputs.check()?;
        if let Some(bytes) = inputs.files.get(&path) {
            return Ok(bytes.to_vec());
        }
        inputs.select(&path)?;
        let available = MAX_TOTAL_BYTES
            .saturating_sub(inputs.bytes)
            .min(MAX_FILE_BYTES);
        let bytes = bounded_read(&path, available).inspect_err(|error| {
            if error.kind() == io::ErrorKind::FileTooLarge {
                inputs.reason = Some("source_limit");
            }
        })?;
        inputs.bytes += bytes.len();
        inputs.files.insert(path, Arc::from(bytes.clone()));
        Ok(bytes)
    })
}

/// Reads UTF-8 using the same byte snapshot as manifest validation and analysis.
pub fn read_to_string(path: &Path) -> io::Result<String> {
    String::from_utf8(read(path)?)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

pub(crate) fn glob(root: &Path, pattern: &str, exclude: &[String]) -> Option<Vec<PathBuf>> {
    INPUTS.with_borrow(|slot| {
        slot.as_ref()?
            .globs
            .get(&(root.into(), pattern.into(), exclude.into()))
            .cloned()
    })
}

pub(crate) fn retain_glob(root: &Path, pattern: &str, exclude: &[String], paths: &[PathBuf]) {
    INPUTS.with_borrow_mut(|slot| {
        if let Some(inputs) = slot {
            inputs
                .globs
                .insert((root.into(), pattern.into(), exclude.into()), paths.into());
        }
    });
}

/// Reserves discovered input identities before accumulating source lists.
pub fn select(path: &Path) -> io::Result<()> {
    INPUTS.with_borrow_mut(|slot| match slot {
        Some(inputs) => {
            inputs.check()?;
            if inputs.verifying {
                return Ok(());
            }
            let identity = fs::canonicalize(path)?;
            if inputs
                .aliases
                .get(path)
                .is_some_and(|first| first != &identity)
            {
                inputs.reason = Some("source_changed");
                return Err(io::Error::other(
                    "Source alias changed identity during the query.",
                ));
            }
            inputs
                .aliases
                .entry(path.into())
                .or_insert_with(|| identity.clone());
            inputs.select(&identity)
        }
        None => Ok(()),
    })
}

pub(crate) fn limit_error() -> io::Error {
    INPUTS.with_borrow_mut(|slot| {
        if let Some(inputs) = slot {
            inputs.reason = Some("source_limit");
        }
    });
    io::Error::new(
        io::ErrorKind::FileTooLarge,
        "Query discovery limit reached.",
    )
}

impl Inputs {
    fn check(&mut self) -> io::Result<()> {
        if (self.cancelled)() {
            self.reason = Some("query_cancelled");
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "Query cancelled.",
            ));
        }
        Ok(())
    }
    fn select(&mut self, path: &Path) -> io::Result<()> {
        if !self.selected.contains(path) && self.selected.len() >= MAX_FILES {
            self.reason = Some("source_limit");
            return Err(io::Error::new(
                io::ErrorKind::FileTooLarge,
                "Query input file limit reached.",
            ));
        }
        self.selected.insert(path.into());
        Ok(())
    }
}

fn bounded_read(path: &Path, maximum: usize) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(maximum as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > maximum {
        return Err(io::Error::new(
            io::ErrorKind::FileTooLarge,
            "Query input byte limit reached.",
        ));
    }
    Ok(bytes)
}
