//! Reuse and publication of linked `.fpascp` program images.

mod atomic;
mod identity;
mod source;
#[cfg(test)]
mod tests;

use std::path::Path;

use fpas_program::{Digest, ProgramIdentity, ProgramImage};
use fpas_project::UnitGraph;

use crate::engine::link_program;
use crate::{
    BuildError, BuildEvent, BuildEventKind, BuildOptions, BuiltProgram, build_library_units,
};

/// Filesystem target and source metadata for one compiled program artifact.
pub struct ProgramArtifactTarget<'a> {
    /// Destination `.fpascp` path.
    pub path: &'a Path,
    /// Authoritative bytes of the main program source.
    pub source: &'a [u8],
    /// Relative source paths indexed by bytecode source identifiers.
    pub source_paths: &'a [String],
}

/// Reuse or rebuild one linked `.fpascp` program image.
///
/// Missing, stale, incompatible, and corrupt artifacts are rebuilt. Publication
/// happens only after compilation, linking, encoding, and temporary-file
/// validation have succeeded.
///
/// The main program is parsed and its reachable units are resolved internally
/// from [`ProgramArtifactTarget::source`], so an unrelated AST cannot be cached
/// under that source identity.
///
/// # Errors
///
/// Returns an error when the source snapshot is invalid, unit resolution or
/// compilation fails, or the validated image cannot be published atomically.
pub fn build_program_artifact(
    graph: &UnitGraph,
    target: ProgramArtifactTarget<'_>,
    options: &BuildOptions,
) -> Result<BuiltProgram, BuildError> {
    build_program_artifact_before_publish(graph, target, options, || {})
}

fn build_program_artifact_before_publish(
    graph: &UnitGraph,
    target: ProgramArtifactTarget<'_>,
    options: &BuildOptions,
    before_publish: impl FnOnce(),
) -> Result<BuiltProgram, BuildError> {
    let program = source::parse(target.source, target.source_paths)?;
    let selection =
        fpas_project::resolve_program_units(graph, &program.uses).map_err(BuildError::new)?;
    let units = build_library_units(graph, &selection, options)?;
    let expected = identity::expected(target.source, &units, options);
    let source_hashes = source_hashes(graph, target.source, target.source_paths.len())?;

    {
        let publication = publication_lock(target.path)?;
        if let Some(executable) =
            reusable_executable(&publication, &expected, target.source_paths, &source_hashes)?
        {
            source::ensure_current(graph, Digest::of(target.source))?;
            let mut events = units.events;
            events.push(BuildEvent {
                owner: program.name.clone(),
                kind: BuildEventKind::ProgramImageReused,
            });
            return Ok(BuiltProgram { executable, events });
        }
    }

    let built = link_program(units, &program)?;
    let BuiltProgram { executable, events } = built;
    let image = ProgramImage::new(
        expected,
        target.source_paths.to_vec(),
        source_hashes,
        executable,
    )
    .map_err(|error| BuildError::new(error.to_string()))?;
    let bytes = fpas_program::encode(&image).map_err(|error| BuildError::new(error.to_string()))?;
    before_publish();
    let publication = publication_lock(target.path)?;
    let replacement = publication
        .prepare(&bytes)
        .map_err(|error| publication_error(target.path, error))?;
    source::ensure_current(graph, Digest::of(target.source))?;
    replacement
        .commit()
        .map_err(|error| publication_error(target.path, error))?;
    Ok(BuiltProgram {
        executable: image.into_executable(),
        events,
    })
}

fn source_hashes(
    graph: &UnitGraph,
    program_source: &[u8],
    source_count: usize,
) -> Result<Vec<fpas_program::Digest>, BuildError> {
    let mut hashes = vec![None; source_count];
    let Some(main) = hashes.first_mut() else {
        return Err(BuildError::new(
            "cannot publish a program image without its main source identity",
        ));
    };
    *main = Some(fpas_program::Digest::of(program_source));
    for (_, node) in graph.iter() {
        let index = node.source_id() as usize;
        let slot = hashes.get_mut(index).ok_or_else(|| {
            BuildError::new(format!(
                "source identity {} is outside the program source table",
                node.source_id()
            ))
        })?;
        *slot = node
            .source_hash()
            .map(|hash| fpas_program::Digest::from_bytes(*hash.as_bytes()));
    }
    hashes
        .into_iter()
        .enumerate()
        .map(|(index, hash)| {
            hash.ok_or_else(|| {
                BuildError::new(format!(
                    "program source identity {index} is unavailable from the build snapshot"
                ))
            })
        })
        .collect()
}

fn reusable_executable(
    publication: &atomic::PublicationLock,
    expected: &ProgramIdentity,
    source_paths: &[String],
    source_hashes: &[Digest],
) -> Result<Option<fpas_bytecode::VerifiedExecutable>, BuildError> {
    let Some(bytes) = publication.read().map_err(BuildError::new)? else {
        return Ok(None);
    };
    let image = match fpas_program::decode(&bytes) {
        Ok(image) => image,
        Err(_) => return Ok(None),
    };
    if image.identity() != expected || !source_table_matches(&image, source_paths, source_hashes) {
        return Ok(None);
    }
    Ok(Some(image.into_executable()))
}

fn source_table_matches(
    image: &ProgramImage,
    source_paths: &[String],
    source_hashes: &[Digest],
) -> bool {
    image
        .source_paths()
        .iter()
        .zip(image.source_hashes())
        .all(|(path, hash)| {
            source_paths
                .iter()
                .position(|expected| expected == path)
                .and_then(|index| source_hashes.get(index))
                == Some(hash)
        })
}

fn publication_lock(path: &Path) -> Result<atomic::PublicationLock, BuildError> {
    atomic::PublicationLock::acquire(path).map_err(|error| publication_error(path, error))
}

fn publication_error(path: &Path, error: String) -> BuildError {
    BuildError::new(format!(
        "cannot publish compiled program `{}`: {error}",
        path.display()
    ))
}
