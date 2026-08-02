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

    if let Some(chunk) = reusable_chunk(target.path, &expected, target.source_paths)? {
        let mut events = units.events;
        events.push(BuildEvent {
            owner: program.name.clone(),
            kind: BuildEventKind::ProgramImageReused,
        });
        return Ok(BuiltProgram { chunk, events });
    }

    let built = link_program(units, &program)?;
    let BuiltProgram { chunk, events } = built;
    let image = ProgramImage::new(expected, target.source_paths.to_vec(), chunk)
        .map_err(|error| BuildError::new(error.to_string()))?;
    let bytes = fpas_program::encode(&image).map_err(|error| BuildError::new(error.to_string()))?;
    atomic::replace(target.path, &bytes).map_err(|error| {
        BuildError::new(format!(
            "cannot publish compiled program `{}`: {error}",
            target.path.display()
        ))
    })?;
    Ok(BuiltProgram {
        chunk: image.into_chunk(),
        events,
    })
}

fn reusable_chunk(
    path: &Path,
    expected: &ProgramIdentity,
    source_paths: &[String],
) -> Result<Option<fpas_bytecode::Chunk>, BuildError> {
    let Some(bytes) = atomic::read(path).map_err(BuildError::new)? else {
        return Ok(None);
    };
    let image = match fpas_program::decode(&bytes) {
        Ok(image) => image,
        Err(_) => return Ok(None),
    };
    if image.identity() != expected || image.source_paths() != source_paths {
        return Ok(None);
    }
    Ok(Some(image.into_chunk()))
}
