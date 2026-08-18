//! Recording envelope, capture log, and program identity.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

mod capture;
mod effects;
mod envelope;

pub use capture::{DebugRecordingEvent, DebugRecordingLog, MAX_RECORDING_EVENTS};
pub(crate) use effects::pending_unsupported_recording_effect;
pub use envelope::{DebugRecordingEnvelope, RECORDING_ENVELOPE_VERSION};
