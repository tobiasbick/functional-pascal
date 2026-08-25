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
    batch
        .candidates
        .drain(..)
        .filter_map(|candidate| {
            let built = batch
                .link
                .as_ref()
                .map_or_else(
                    || {
                        crate::project_build::build_test_program(
                            &candidate.path,
                            &[],
                            &fpas_project::ProjectLinkMeta::default(),
                            None,
                        )
                    },
                    |link| {
                        crate::project_build::build_test_program_with_graph(
                            &candidate.path,
                            &link.program_graph,
                        )
                    },
                )
                .ok()?;
            Some(ImageAssignment {
                prepared_index: candidate.prepared_index,
                compiled: CompiledTestProgram {
                    image: Arc::new(built.executable),
                    source_paths: Arc::new(built.source_paths),
                },
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_batch_collection_produces_no_assignments() {
        assert!(compile_image_batches(Vec::new()).is_empty());
    }

    #[test]
    fn one_candidate_is_precompiled() {
        let root = std::env::temp_dir().join(format!("fpas-image-single-{}", std::process::id()));
        std::fs::create_dir_all(&root).expect("fixture directory");
        let path = root.join("one_test.fpas");
        std::fs::write(&path, "program One; begin end.").expect("fixture source");
        let batch = ImageBatch::new(
            vec![ImageCandidate {
                prepared_index: 0,
                path,
            }],
            None,
        );
        assert_eq!(compile_image_batches(vec![batch]).len(), 1);
        std::fs::remove_dir_all(root).ok();
    }
}
