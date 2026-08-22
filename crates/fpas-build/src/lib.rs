#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::panic,
        clippy::unwrap_used,
        reason = "tests use explicit failures to keep fixture assertions focused"
    )
)]

//! Incremental source-adjacent compiled-unit build pipeline.

mod distribution;
mod engine;
mod events;
mod options;
mod program_artifact;
mod source_snapshot;

pub use distribution::{DistributionError, stage_standard_library};
pub use engine::{
    BuildError, BuiltProgram, BuiltUnits, build_library_units, build_program, check_library_units,
    check_program,
};
pub use events::{BuildCounters, BuildEvent, BuildEventKind};
pub use options::BuildOptions;
pub use program_artifact::{ProgramArtifactTarget, build_program_artifact};
