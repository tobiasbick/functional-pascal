//! Typed conversion from service failures to JSON-RPC error categories.

use fpas_language_service::{LanguageServiceError, RenameError};
use tower_lsp_server::jsonrpc::Error;

use crate::documents::{DocumentRequestError, DocumentSyncError};

pub(super) fn request(error: DocumentRequestError) -> Error {
    match error {
        DocumentRequestError::Service(error) => service(error),
        DocumentRequestError::Rename(RenameError::Service(error)) => service(error),
        DocumentRequestError::Rename(RenameError::Validation(error)) => {
            request(DocumentRequestError::SourceTool(error))
        }
        DocumentRequestError::Rename(error) => Error::invalid_params(error.to_string()),
        DocumentRequestError::SourceTool(error) => {
            if error.code == "query_cancelled" {
                return Error::request_cancelled();
            }
            let mut result = Error::invalid_params(error.message);
            let diagnostics = error
                .diagnostics
                .iter()
                .filter_map(|record| {
                    let path = record.path.as_ref().map(|p| p.to_string_lossy());
                    fpas_diagnostics::render_json(
                        path.as_deref(),
                        record.source.as_deref(),
                        &record.diagnostic,
                    )
                    .ok()
                    .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                })
                .collect::<Vec<_>>();
            result.data = Some(serde_json::json!({"code": error.code, "diagnostics": diagnostics}));
            result
        }
        DocumentRequestError::Position(error) => Error::invalid_params(error.to_string()),
        DocumentRequestError::DocumentNotOpen { path } => Error::invalid_params(format!(
            "Cannot query `{}` because the document is not open.",
            path.display()
        )),
        DocumentRequestError::Task(message) => internal(message),
    }
}

pub(super) fn synchronization(error: DocumentSyncError) -> Error {
    match error {
        DocumentSyncError::Service(error) => service(error),
        error => Error::invalid_params(error.to_string()),
    }
}

pub(super) fn service(error: LanguageServiceError) -> Error {
    match error {
        LanguageServiceError::Cancelled => Error::request_cancelled(),
        error => internal(error.to_string()),
    }
}

fn internal(message: String) -> Error {
    let mut error = Error::internal_error();
    error.message = message.into();
    error
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use fpas_language_service::{LanguageServiceError, RenameError};
    use tower_lsp_server::jsonrpc::ErrorCode;

    use super::request;
    use crate::convert::PositionConversionError;
    use crate::documents::DocumentRequestError;

    #[test]
    fn parameter_failures_map_to_invalid_params() {
        let errors = [
            DocumentRequestError::DocumentNotOpen {
                path: PathBuf::from("closed.fpas"),
            },
            DocumentRequestError::Position(PositionConversionError::LineOutOfRange { line: 2 }),
            DocumentRequestError::Rename(RenameError::InvalidIdentifier {
                name: "begin".to_owned(),
            }),
        ];
        assert!(
            errors
                .into_iter()
                .all(|error| request(error).code == ErrorCode::InvalidParams)
        );
    }

    #[test]
    fn service_and_task_failures_map_to_internal_error() {
        let errors = [
            DocumentRequestError::Service(LanguageServiceError::Analysis {
                path: PathBuf::from("source.fpas"),
                message: "analysis failed".to_owned(),
            }),
            DocumentRequestError::Task("worker panicked".to_owned()),
        ];
        assert!(
            errors
                .into_iter()
                .all(|error| request(error).code == ErrorCode::InternalError)
        );
    }

    #[test]
    fn direct_and_nested_cancellation_map_to_request_cancelled() {
        let errors = [
            DocumentRequestError::Service(LanguageServiceError::Cancelled),
            DocumentRequestError::Rename(RenameError::Service(LanguageServiceError::Cancelled)),
        ];
        assert!(
            errors
                .into_iter()
                .all(|error| request(error).code == ErrorCode::RequestCancelled)
        );
    }
}
