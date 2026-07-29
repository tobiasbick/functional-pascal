//! File-URI conversion with explicit scheme validation.

use std::fmt;
use std::path::PathBuf;

use tower_lsp_server::ls_types::Uri;

/// A recoverable failure while converting a protocol URI to a local file path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileUriError {
    uri: String,
    reason: &'static str,
}

impl FileUriError {
    fn new(uri: &Uri, reason: &'static str) -> Self {
        Self {
            uri: uri.as_str().to_owned(),
            reason,
        }
    }

    /// Returns the rejected URI text.
    #[must_use]
    pub fn uri(&self) -> &str {
        &self.uri
    }
}

impl fmt::Display for FileUriError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Cannot use document URI `{}`: {}",
            self.uri, self.reason
        )
    }
}

impl std::error::Error for FileUriError {}

/// Converts a `file:` URI to a local path and rejects every other URI scheme.
pub fn file_uri_to_path(uri: &Uri) -> Result<PathBuf, FileUriError> {
    if !uri.scheme().as_str().eq_ignore_ascii_case("file") {
        return Err(FileUriError::new(
            uri,
            "unsupported URI scheme; open a local `file:` document",
        ));
    }

    uri.to_file_path()
        .map(|path| path.into_owned())
        .ok_or_else(|| FileUriError::new(uri, "the file URI cannot be converted to a local path"))
}
