//! Incremental source-adjacent compiled-unit build pipeline.

mod distribution;
mod engine;
mod events;
mod options;
mod program_artifact;

pub use distribution::{DistributionError, stage_standard_library};
pub use engine::{BuildError, BuiltProgram, BuiltUnits, build_library_units, build_program};
pub use events::{BuildCounters, BuildEvent, BuildEventKind};
pub use options::BuildOptions;
pub use program_artifact::{ProgramArtifactTarget, build_program_artifact};
