//! Versioned recording envelope for program identity without host paths.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use fpas_bytecode::{BYTECODE_VERSION, VerifiedExecutable};

use super::super::types::{DebugErrorKind, DebugSessionError};

/// Envelope schema version for debugger recordings.
pub const RECORDING_ENVELOPE_VERSION: u32 = 1;

/// Versioned source and program identity for one debuggee executable.
///
/// Sources are portable identities from the executable source map. Host
/// filesystem paths are rejected instead of recorded.
///
/// **Documentation:** `docs/pascal/tools/debugger.md`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugRecordingEnvelope {
    /// Envelope schema version.
    pub version: u32,
    /// Bytecode version understood by this runtime.
    pub bytecode_version: u32,
    /// Entry-function diagnostic name.
    pub program: String,
    /// Portable source identities in executable order.
    pub sources: Vec<String>,
}

impl DebugRecordingEnvelope {
    /// Name the current executable without capturing events or host paths.
    ///
    /// **Documentation:** `docs/pascal/tools/debugger.md`
    ///
    /// # Errors
    ///
    /// Returns [`DebugErrorKind::RecordingHostPath`] when a source identity is a
    /// host filesystem path. The error does not echo the path.
    pub fn from_executable(executable: &VerifiedExecutable) -> Result<Self, DebugSessionError> {
        let image = executable.executable();
        let mut sources = Vec::new();
        for source in &image.source_map.sources {
            let Some(path) = image.strings.get(*source) else {
                continue;
            };
            if path.is_empty() {
                continue;
            }
            if source_identity_is_absolute(path) {
                return Err(host_path_error());
            }
            if !sources.iter().any(|existing| existing == path) {
                sources.push(path.to_owned());
            }
        }
        let program = image
            .functions
            .get(usize::from(image.entry.get()))
            .and_then(|function| image.strings.get(function.name))
            .unwrap_or_default()
            .to_owned();
        Ok(Self {
            version: RECORDING_ENVELOPE_VERSION,
            bytecode_version: BYTECODE_VERSION,
            program,
            sources,
        })
    }
}

fn host_path_error() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::RecordingHostPath,
        message: "recording identity cannot include a host filesystem path".to_string(),
        hint: "Rebuild the debuggee with portable source identities. Recordings never store host paths.".to_string(),
    }
}

/// Match the program-image absolute-source rule without depending on `fpas-program`.
fn source_identity_is_absolute(path: &str) -> bool {
    let bytes = path.as_bytes();
    path.starts_with('/')
        || path.starts_with('\\')
        || matches!(
            bytes,
            [drive, b':', separator, ..]
                if drive.is_ascii_alphabetic() && matches!(separator, b'/' | b'\\')
        )
}
