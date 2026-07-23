//! Incremental source-adjacent compiled-unit build pipeline.

mod engine;
mod events;
mod options;

pub use engine::{BuildError, BuiltProgram, BuiltUnits, build_library_units, build_program};
pub use events::{BuildCounters, BuildEvent, BuildEventKind};
pub use options::BuildOptions;
