//! Recording envelope, capture log, and program identity.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

mod capture;
mod envelope;

pub use capture::{DebugRecordingEvent, DebugRecordingLog, MAX_RECORDING_EVENTS};
pub use envelope::{DebugRecordingEnvelope, RECORDING_ENVELOPE_VERSION};
