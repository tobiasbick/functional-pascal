//! Registered rich source-tool extension, with ordinary cancellation and document ordering.

use super::{Backend, errors};
use crate::source_tools_requests::{SourceQueryParams, SourceQueryResponse};
use tower_lsp_server::jsonrpc::Result;

impl Backend {
    pub(crate) async fn source_query_request(
        &self,
        params: SourceQueryParams,
    ) -> Result<SourceQueryResponse> {
        self.documents
            .source_query(params)
            .await
            .map_err(errors::request)
    }
}
