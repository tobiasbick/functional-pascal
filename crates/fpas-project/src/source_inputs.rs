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

mod inventory;
#[cfg(test)]
mod tests;

pub use inventory::{InputAlias, InputFile, InputGlob, InputInventory};

/// Maximum canonical input files in a source request.
pub const MAX_FILES: usize = 4096;
/// Maximum bytes in one source or manifest.
pub const MAX_FILE_BYTES: usize = 8 * 1024 * 1024;
/// Maximum bytes across sources and manifests.
pub const MAX_TOTAL_BYTES: usize = 64 * 1024 * 1024;
/// Maximum retained alias and discovery metadata, separate from source bytes.
pub const MAX_METADATA_BYTES: usize = 8 * 1024 * 1024;

thread_local! { static INPUTS: RefCell<Option<Inputs>> = const { RefCell::new(None) }; }

struct Inputs {
    files: BTreeMap<PathBuf, Arc<[u8]>>,
    overlays: BTreeMap<PathBuf, Arc<[u8]>>,
    selected: BTreeSet<PathBuf>,
    aliases: BTreeMap<PathBuf, PathBuf>,
    globs: BTreeMap<(PathBuf, String, Vec<String>), Vec<PathBuf>>,
    bytes: usize,
    metadata_bytes: usize,
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
        Self::begin_with_overlays(std::iter::empty::<(PathBuf, Arc<[u8]>)>(), cancelled)
    }

    /// Freezes supplied editor bytes at the same loader boundary as saved sources.
    /// Manifest discovery and semantic analysis therefore see the same overlays.
    pub fn begin_with_overlays<S: AsRef<[u8]> + Into<Arc<[u8]>>>(
        overlays: impl IntoIterator<Item = (PathBuf, S)>,
        cancelled: impl Fn() -> bool + 'static,
    ) -> io::Result<Self> {
        let mut frozen = BTreeMap::<PathBuf, Arc<[u8]>>::new();
        let mut bytes = 0usize;
        for (path, source) in overlays {
            let path = match fs::canonicalize(path) {
                Ok(path) => path,
                // Open buffers may outlive an unrelated deleted project. They cannot add
                // physical members to a checked scope; selecting a missing input still fails.
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(error) => return Err(error),
            };
            if let Some(previous) = frozen.get(&path) {
                if previous.as_ref() != source.as_ref() {
                    return Err(io::Error::other(
                        "Conflicting editor overlays for one physical source.",
                    ));
                }
                continue;
            }
            bytes = bytes.saturating_add(source.as_ref().len());
            if frozen.len() >= MAX_FILES
                || source.as_ref().len() > MAX_FILE_BYTES
                || bytes > MAX_TOTAL_BYTES
            {
                return Err(io::Error::new(
                    io::ErrorKind::FileTooLarge,
                    "Editor input limit reached.",
                ));
            }
            // Borrowed editor text is copied only after its size was admitted; owned bytes
            // can transfer their Arc without a second allocation.
            frozen.insert(path, source.into());
        }
        INPUTS.with_borrow_mut(|slot| {
            if slot.is_some() {
                return Err(io::Error::other("A source snapshot is already active."));
            }
            *slot = Some(Inputs {
                files: BTreeMap::new(),
                overlays: frozen,
                selected: BTreeSet::new(),
                aliases: BTreeMap::new(),
                globs: BTreeMap::new(),
                bytes: 0,
                metadata_bytes: 0,
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

    /// Reads every admitted file, making the exported scope independent of lazy analysis reads.
    pub fn freeze_selected(&self) -> io::Result<()> {
        let selected = INPUTS.with_borrow(|slot| {
            slot.as_ref()
                .map(|i| i.selected.clone())
                .unwrap_or_default()
        });
        for path in selected {
            if fs::metadata(&path)?.is_file() {
                read(&path)?;
            }
        }
        Ok(())
    }

    /// Exports a complete owned read set for validation after the guard or worker ends.
    /// Call `freeze_selected` first; an incomplete or cancelled scope cannot be exported.
    pub fn inventory(&self) -> io::Result<InputInventory> {
        inventory::capture()
    }

    /// Checks current bytes and glob membership without replacing the frozen inputs.
    pub fn verify(&self) -> io::Result<bool> {
        inventory::capture_read_set()?.verify()
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
        let bytes = if let Some(overlay) = inputs.overlays.get(&path) {
            if overlay.len() > available {
                inputs.reason = Some("source_limit");
                return Err(io::Error::new(
                    io::ErrorKind::FileTooLarge,
                    "Query input byte limit reached.",
                ));
            }
            overlay.to_vec()
        } else {
            bounded_read(&path, available).inspect_err(|error| {
                if error.kind() == io::ErrorKind::FileTooLarge {
                    inputs.reason = Some("source_limit");
                }
            })?
        };
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

pub(crate) fn retain_glob(
    root: &Path,
    pattern: &str,
    exclude: &[String],
    paths: &[PathBuf],
) -> io::Result<()> {
    INPUTS.with_borrow_mut(|slot| {
        if let Some(inputs) = slot {
            let bytes = root
                .as_os_str()
                .len()
                .saturating_add(pattern.len())
                .saturating_add(
                    exclude
                        .iter()
                        .map(|p| p.len().saturating_add(24))
                        .sum::<usize>(),
                )
                .saturating_add(
                    paths
                        .iter()
                        .map(|p| p.as_os_str().len().saturating_add(24))
                        .sum::<usize>(),
                );
            inputs.retain_metadata(bytes)?;
            inputs
                .globs
                .insert((root.into(), pattern.into(), exclude.into()), paths.into());
        }
        Ok(())
    })
}

/// Reserves discovered input identities before accumulating source lists.
pub fn select(path: &Path) -> io::Result<()> {
    let path = std::path::absolute(path)?;
    INPUTS.with_borrow_mut(|slot| match slot {
        Some(inputs) => {
            inputs.check()?;
            if inputs.verifying {
                return Ok(());
            }
            let identity = fs::canonicalize(&path)?;
            if inputs
                .aliases
                .get(&path)
                .is_some_and(|first| first != &identity)
            {
                inputs.reason = Some("source_changed");
                return Err(io::Error::other(
                    "Source alias changed identity during the query.",
                ));
            }
            if !inputs.aliases.contains_key(&path) {
                inputs.retain_metadata(
                    path.as_os_str()
                        .len()
                        .saturating_add(identity.as_os_str().len())
                        .saturating_add(64),
                )?;
            }
            inputs
                .aliases
                .entry(path)
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
    fn retain_metadata(&mut self, bytes: usize) -> io::Result<()> {
        let total = self.metadata_bytes.saturating_add(bytes);
        if total > MAX_METADATA_BYTES {
            self.reason = Some("source_limit");
            return Err(io::Error::new(
                io::ErrorKind::FileTooLarge,
                "Query discovery metadata limit reached.",
            ));
        }
        self.metadata_bytes = total;
        Ok(())
    }
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
