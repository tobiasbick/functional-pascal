//! In-memory model for portable verified register program images.

mod limits;

use std::collections::{HashMap, HashSet};
use std::fmt;

use fpas_code::{StringId, StringTable, ValidationError, VerifiedExecutable};

use crate::{Digest, ProgramIdentity};

use self::limits::validate_identity_resources;

/// Invalid in-memory program image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageError {
    /// Required identity field is empty.
    EmptyIdentityField(&'static str),
    /// Image bytecode is incompatible with this runtime.
    BytecodeVersion {
        /// Version recorded in the image.
        image: u32,
        /// Version understood by this runtime.
        runtime: u32,
    },
    /// A reachable unit appears more than once.
    DuplicateUnit(String),
    /// The portable source-path table has the wrong length.
    SourcePathCount {
        /// Number of paths supplied by the image.
        paths: usize,
        /// Number of source identifiers used by the executable.
        sources: usize,
    },
    /// The source digest table has the wrong length.
    SourceHashCount {
        /// Number of digests supplied by the image.
        hashes: usize,
        /// Number of source identifiers used by the executable.
        sources: usize,
    },
    /// A source path is empty.
    EmptySourcePath(usize),
    /// A source path appears more than once.
    DuplicateSourcePath(String),
    /// A source path contains machine-specific absolute metadata.
    AbsoluteSourcePath(String),
    /// Register executable verification failed.
    Executable(ValidationError),
    /// A program-image resource exceeds its configured limit.
    ResourceLimit {
        /// Logical resource name.
        field: &'static str,
        /// Observed resource size.
        size: usize,
        /// Largest accepted resource size.
        maximum: usize,
    },
}

impl fmt::Display for ImageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BytecodeVersion { image, runtime } => write!(
                formatter,
                "compiled program uses bytecode version {image}, but this runtime requires version {runtime}"
            ),
            Self::Executable(error) => write!(formatter, "invalid register executable: {error}"),
            other => write!(formatter, "invalid compiled program image: {other:?}"),
        }
    }
}

impl std::error::Error for ImageError {}

/// Complete portable register executable plus its build identity and diagnostic paths.
#[derive(Debug)]
pub struct ProgramImage {
    identity: ProgramIdentity,
    source_paths: Vec<String>,
    source_hashes: Vec<Digest>,
    executable: VerifiedExecutable,
}

impl ProgramImage {
    /// Construct and validate a complete program image.
    ///
    /// Portable source paths replace compiler-local source labels in the executable string table.
    ///
    /// # Errors
    ///
    /// Returns [`ImageError`] when identity, source paths, resources, or executable invariants fail.
    pub fn new(
        mut identity: ProgramIdentity,
        source_paths: Vec<String>,
        source_hashes: Vec<Digest>,
        executable: VerifiedExecutable,
    ) -> Result<Self, ImageError> {
        canonicalize_units(&mut identity);
        validate_identity(&identity)?;
        validate_source_paths(&source_paths)?;
        validate_source_hash_count(source_hashes.len(), source_paths.len())?;
        validate_identity_resources(&identity)?;
        let (executable, selected_sources) = install_source_paths(executable, &source_paths)?;
        let selected_hashes = selected_sources
            .iter()
            .map(|&index| source_hashes[index])
            .collect();
        let selected_paths = selected_sources
            .into_iter()
            .map(|index| source_paths[index].clone())
            .collect();
        Ok(Self {
            identity,
            source_paths: selected_paths,
            source_hashes: selected_hashes,
            executable,
        })
    }

    pub(crate) fn from_decoded(
        mut identity: ProgramIdentity,
        source_hashes: Vec<Digest>,
        executable: VerifiedExecutable,
    ) -> Result<Self, ImageError> {
        canonicalize_units(&mut identity);
        validate_identity(&identity)?;
        validate_identity_resources(&identity)?;
        let source_paths = executable
            .executable()
            .source_map
            .sources
            .iter()
            .map(|id| {
                executable
                    .executable()
                    .strings
                    .get(*id)
                    .unwrap_or_default()
                    .to_string()
            })
            .collect::<Vec<_>>();
        validate_source_paths(&source_paths)?;
        validate_source_hash_count(source_hashes.len(), source_paths.len())?;
        Ok(Self {
            identity,
            source_paths,
            source_hashes,
            executable,
        })
    }

    /// Return the recorded build identity.
    #[must_use]
    pub const fn identity(&self) -> &ProgramIdentity {
        &self.identity
    }

    /// Return portable relative source paths indexed by bytecode source identifiers.
    #[must_use]
    pub fn source_paths(&self) -> &[String] {
        &self.source_paths
    }

    /// Return source-content digests indexed by bytecode source identifiers.
    #[must_use]
    pub fn source_hashes(&self) -> &[Digest] {
        &self.source_hashes
    }

    /// Return the verified register executable.
    #[must_use]
    pub const fn executable(&self) -> &VerifiedExecutable {
        &self.executable
    }

    /// Consume the image and return its verified register executable.
    #[must_use]
    pub fn into_executable(self) -> VerifiedExecutable {
        self.executable
    }
}

fn validate_source_hash_count(hashes: usize, sources: usize) -> Result<(), ImageError> {
    if hashes != sources {
        return Err(ImageError::SourceHashCount { hashes, sources });
    }
    Ok(())
}

fn canonicalize_units(identity: &mut ProgramIdentity) {
    for unit in &mut identity.units {
        unit.unit_name.make_ascii_lowercase();
    }
}

fn validate_identity(identity: &ProgramIdentity) -> Result<(), ImageError> {
    if identity.compiler_version.trim().is_empty() {
        return Err(ImageError::EmptyIdentityField("compiler_version"));
    }
    if identity.bytecode_version != fpas_code::BYTECODE_VERSION {
        return Err(ImageError::BytecodeVersion {
            image: identity.bytecode_version,
            runtime: fpas_code::BYTECODE_VERSION,
        });
    }
    let mut units = HashSet::with_capacity(identity.units.len());
    for unit in &identity.units {
        if unit.unit_name.trim().is_empty() {
            return Err(ImageError::EmptyIdentityField("unit_name"));
        }
        if !units.insert(&unit.unit_name) {
            return Err(ImageError::DuplicateUnit(unit.unit_name.clone()));
        }
    }
    Ok(())
}

fn validate_source_paths(source_paths: &[String]) -> Result<(), ImageError> {
    let mut unique = HashSet::with_capacity(source_paths.len());
    for (index, source_path) in source_paths.iter().enumerate() {
        if source_path.trim().is_empty() {
            return Err(ImageError::EmptySourcePath(index));
        }
        if is_portably_absolute(source_path) {
            return Err(ImageError::AbsoluteSourcePath(source_path.clone()));
        }
        if !unique.insert(source_path) {
            return Err(ImageError::DuplicateSourcePath(source_path.clone()));
        }
    }
    Ok(())
}

fn install_source_paths(
    verified: VerifiedExecutable,
    source_paths: &[String],
) -> Result<(VerifiedExecutable, Vec<usize>), ImageError> {
    let mut executable = verified.into_unverified();
    let mut strings = executable
        .strings
        .iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let selected_sources = executable
        .source_map
        .sources
        .iter()
        .enumerate()
        .map(|(source_index, source)| {
            let label = executable.strings.get(*source).unwrap_or_default();
            source_index_for_label(label, source_index, source_paths.len())
        })
        .collect::<Option<Vec<_>>>()
        .ok_or(ImageError::SourcePathCount {
            paths: source_paths.len(),
            sources: executable.source_map.sources.len(),
        })?;
    let mut string_indices = executable
        .strings
        .iter()
        .enumerate()
        .map(|(index, text)| (text, index))
        .collect::<HashMap<_, _>>();
    let mut unique_sources = HashSet::with_capacity(selected_sources.len());
    executable.source_map.sources.clear();
    for &source in &selected_sources {
        let path = &source_paths[source];
        if !unique_sources.insert(source) {
            return Err(ImageError::DuplicateSourcePath(path.clone()));
        }
        let index = *string_indices.entry(path.as_str()).or_insert_with(|| {
            let index = strings.len();
            strings.push(path.clone());
            index
        });
        let id = StringId::try_from_index(index).map_err(|_| ImageError::ResourceLimit {
            field: "strings",
            size: strings.len(),
            maximum: fpas_code::limits::MAX_STRINGS,
        })?;
        executable.source_map.sources.push(id);
    }
    executable.strings = StringTable::new(strings);
    let executable = executable.verify().map_err(ImageError::Executable)?;
    Ok((executable, selected_sources))
}

fn source_index_for_label(label: &str, source_index: usize, source_count: usize) -> Option<usize> {
    label
        .strip_prefix("source-")
        .and_then(|suffix| suffix.strip_suffix(".fpas"))
        .and_then(|index| index.parse::<usize>().ok())
        .filter(|&index| index < source_count)
        .or_else(|| (source_index < source_count).then_some(source_index))
}

fn is_portably_absolute(path: &str) -> bool {
    let bytes = path.as_bytes();
    path.starts_with('/')
        || path.starts_with('\\')
        || matches!(
            bytes,
            [drive, b':', separator, ..]
                if drive.is_ascii_alphabetic() && matches!(separator, b'/' | b'\\')
        )
}

#[cfg(test)]
mod tests;
