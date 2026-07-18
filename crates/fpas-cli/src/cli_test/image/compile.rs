//! Compilation and fallback splitting for in-memory test images.

use std::path::PathBuf;
use std::sync::Arc;

use super::super::run::{CompiledTestProgram, LinkContext};

/// One test that belongs to an image-build batch.
pub(super) struct ImageCandidate {
    /// Index in the runner's prepared-test collection.
    pub prepared_index: usize,
    /// Source program to link into the image.
    pub path: PathBuf,
}

/// Compatible tests and their shared project-link context.
pub(super) struct ImageBatch {
    candidates: Vec<ImageCandidate>,
    link: Option<LinkContext>,
}

impl ImageBatch {
    /// Creates one independently compilable image batch.
    pub(super) fn new(candidates: Vec<ImageCandidate>, link: Option<LinkContext>) -> Self {
        Self { candidates, link }
    }
}

/// One successfully compiled image entry ready to attach to a prepared test.
pub(super) struct ImageAssignment {
    /// Index in the runner's prepared-test collection.
    pub prepared_index: usize,
    /// Shared image and this test's entry offset.
    pub compiled: CompiledTestProgram,
}

/// Compiles independent image batches without changing test order.
pub(super) fn compile_image_batches(batches: Vec<ImageBatch>) -> Vec<ImageAssignment> {
    batches.into_iter().flat_map(compile_image_batch).collect()
}

fn compile_image_batch(mut batch: ImageBatch) -> Vec<ImageAssignment> {
    if batch.candidates.len() < 2 {
        return Vec::new();
    }

    let paths = batch
        .candidates
        .iter()
        .map(|candidate| candidate.path.clone())
        .collect::<Vec<_>>();
    let default_link_meta = fpas_project::ProjectLinkMeta::default();
    let (source_files, link_meta) = batch.link.as_ref().map_or_else(
        || (&[][..], &default_link_meta),
        |link| (link.source_files.as_slice(), &link.link_meta),
    );
    let bundle = if let Some(standard_library) = batch
        .link
        .as_ref()
        .and_then(|link| link.standard_library.as_deref())
    {
        fpas_project::build_test_bundle_from_paths_with_standard_library(
            &paths,
            source_files,
            link_meta,
            standard_library,
        )
    } else {
        fpas_project::build_test_bundle_from_paths(&paths, source_files, link_meta)
    };
    let Ok(bundle) = bundle else {
        return retry_smaller_batches(batch);
    };
    let Ok(chunk) = fpas_compiler::compile_all(&bundle.program) else {
        return retry_smaller_batches(batch);
    };
    let entry_offsets = bundle
        .entry_names
        .iter()
        .map(|name| {
            chunk
                .functions()
                .get(&name.to_ascii_lowercase())
                .map(|(offset, _)| *offset)
        })
        .collect::<Option<Vec<_>>>();
    let Some(entry_offsets) = entry_offsets else {
        return retry_smaller_batches(batch);
    };

    let image = Arc::new(chunk);
    let source_paths = Arc::new(bundle.source_paths);
    batch
        .candidates
        .drain(..)
        .zip(entry_offsets)
        .map(|(candidate, entry_ip)| ImageAssignment {
            prepared_index: candidate.prepared_index,
            compiled: CompiledTestProgram {
                image: Arc::clone(&image),
                entry_ip,
                source_paths: Arc::clone(&source_paths),
            },
        })
        .collect()
}

fn retry_smaller_batches(mut batch: ImageBatch) -> Vec<ImageAssignment> {
    if batch.candidates.len() <= 2 {
        return Vec::new();
    }
    let right = batch.candidates.split_off(batch.candidates.len() / 2);
    let left = std::mem::take(&mut batch.candidates);
    let mut assignments = compile_image_batch(ImageBatch::new(left, batch.link.clone()));
    assignments.extend(compile_image_batch(ImageBatch::new(right, batch.link)));
    assignments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_batch_collection_produces_no_assignments() {
        assert!(compile_image_batches(Vec::new()).is_empty());
    }

    #[test]
    fn one_candidate_is_left_for_individual_compilation() {
        let batch = ImageBatch::new(
            vec![ImageCandidate {
                prepared_index: 0,
                path: PathBuf::from("one_test.fpas"),
            }],
            None,
        );
        assert!(compile_image_batches(vec![batch]).is_empty());
    }
}
