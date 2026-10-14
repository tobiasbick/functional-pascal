//! Versioned public-signature requests for read-only editor documents.

use fpas_language_service::SourceVersion;
use serde::Serialize;
use tower_lsp_server::{
    jsonrpc::{Error, Result},
    ls_types::TextDocumentIdentifier,
};

use crate::{
    convert::file_uri_to_path,
    documents::{require_open, tasks},
    server::{Backend, errors},
};

/// Public-signature text and the exact editor version it describes.
#[derive(Serialize)]
pub(crate) struct PublicSignatures {
    text: String,
    version: i64,
}

impl Backend {
    /// Projects the requested open unit without writing any source or artifact.
    pub(crate) async fn public_signatures_request(
        &self,
        document: TextDocumentIdentifier,
    ) -> Result<PublicSignatures> {
        let path = file_uri_to_path(&document.uri)
            .map_err(|error| Error::invalid_params(error.to_string()))?;
        let result = tasks::run(&self.documents.service, move |service, cancellation| {
            cancellation.check()?;
            require_open(service, &path)?;
            let result = service.public_signatures(&path)?;
            cancellation.check()?;
            Ok::<_, crate::documents::DocumentRequestError>(result)
        })
        .await
        .map_err(errors::request)?;
        let SourceVersion::Editor(version) = result.snapshot.version() else {
            return Err(Error::invalid_params(
                "Open the unit in the editor before requesting public signatures.",
            ));
        };
        Ok(PublicSignatures {
            text: result.value,
            version,
        })
    }
}
