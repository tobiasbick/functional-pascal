//! Reuse and publication of linked `.fpascp` program images.

mod atomic;
mod identity;
mod source;

use std::path::Path;

use fpas_program::{ProgramIdentity, ProgramImage};
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
    let program = source::parse(target.source, target.source_paths)?;
    let selection =
        fpas_project::resolve_program_units(graph, &program.uses).map_err(BuildError::new)?;
    let units = build_library_units(graph, &selection, options)?;
    let expected = identity::expected(target.source, &units, options);

    if let Some(executable) = reusable_executable(target.path, &expected, target.source_paths)? {
        let mut events = units.events;
        events.push(BuildEvent {
            owner: program.name.clone(),
            kind: BuildEventKind::ProgramImageReused,
        });
        return Ok(BuiltProgram { executable, events });
    }

    let built = link_program(units, &program)?;
    let BuiltProgram { executable, events } = built;
    let source_hashes = source_hashes(graph, target.source, target.source_paths.len())?;
    let image = ProgramImage::new(
        expected,
        target.source_paths.to_vec(),
        source_hashes,
        executable,
    )
    .map_err(|error| BuildError::new(error.to_string()))?;
    let bytes = fpas_program::encode(&image).map_err(|error| BuildError::new(error.to_string()))?;
    atomic::replace(target.path, &bytes).map_err(|error| {
        BuildError::new(format!(
            "cannot publish compiled program `{}`: {error}",
            target.path.display()
        ))
    })?;
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
    path: &Path,
    expected: &ProgramIdentity,
    source_paths: &[String],
) -> Result<Option<fpas_bytecode::VerifiedExecutable>, BuildError> {
    let Some(bytes) = atomic::read(path).map_err(BuildError::new)? else {
        return Ok(None);
    };
    let image = match fpas_program::decode(&bytes) {
        Ok(image) => image,
        Err(_) => return Ok(None),
    };
    if image.identity() != expected
        || image
            .source_paths()
            .iter()
            .any(|path| !source_paths.contains(path))
    {
        return Ok(None);
    }
    Ok(Some(image.into_executable()))
}
