//! Standard hierarchy handlers delegate all source identity and edge discovery to the core.

use super::{Backend, errors};
use tower_lsp_server::{
    jsonrpc::Result,
    ls_types::{
        CallHierarchyIncomingCall, CallHierarchyIncomingCallsParams, CallHierarchyItem,
        CallHierarchyOutgoingCall, CallHierarchyOutgoingCallsParams, CallHierarchyPrepareParams,
    },
};

impl Backend {
    pub(super) async fn prepare_call_hierarchy_request(
        &self,
        params: CallHierarchyPrepareParams,
    ) -> Result<Option<Vec<CallHierarchyItem>>> {
        self.documents
            .prepare_call_hierarchy(params)
            .await
            .map(Some)
            .map_err(errors::request)
    }
    pub(super) async fn incoming_calls_request(
        &self,
        params: CallHierarchyIncomingCallsParams,
    ) -> Result<Option<Vec<CallHierarchyIncomingCall>>> {
        self.documents
            .incoming_calls(params.item)
            .await
            .map(Some)
            .map_err(errors::request)
    }
    pub(super) async fn outgoing_calls_request(
        &self,
        params: CallHierarchyOutgoingCallsParams,
    ) -> Result<Option<Vec<CallHierarchyOutgoingCall>>> {
        self.documents
            .outgoing_calls(params.item)
            .await
            .map(Some)
            .map_err(errors::request)
    }
}
