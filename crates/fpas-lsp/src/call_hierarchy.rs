//! Standard Call Hierarchy is a projection of checked direct edges, with snapshot-bound data.

use crate::documents::read_set::Versions;
use crate::documents::{DocumentRequestError, SynchronizedDocuments, tasks};
use fpas_language_service::{
    CallableIdentity, CallableReference, SourceCallable, SourceQueryError, SourceSession,
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use tower_lsp_server::ls_types::{
    CallHierarchyIncomingCall, CallHierarchyItem, CallHierarchyOutgoingCall,
    CallHierarchyPrepareParams, Range, SymbolKind, Uri,
};

enum Request {
    Prepare(CallHierarchyPrepareParams),
    Incoming(CallHierarchyItem),
    Outgoing(CallHierarchyItem),
}
enum Reply {
    Prepare(Vec<CallHierarchyItem>),
    Incoming(Vec<CallHierarchyIncomingCall>),
    Outgoing(Vec<CallHierarchyOutgoingCall>),
}

impl SynchronizedDocuments {
    pub(crate) async fn prepare_call_hierarchy(
        &self,
        params: CallHierarchyPrepareParams,
    ) -> Result<Vec<CallHierarchyItem>, DocumentRequestError> {
        match self.call_hierarchy(Request::Prepare(params)).await? {
            Reply::Prepare(items) => Ok(items),
            _ => Err(failure("invalid_request", "Unexpected hierarchy reply.")),
        }
    }
    pub(crate) async fn incoming_calls(
        &self,
        item: CallHierarchyItem,
    ) -> Result<Vec<CallHierarchyIncomingCall>, DocumentRequestError> {
        match self.call_hierarchy(Request::Incoming(item)).await? {
            Reply::Incoming(items) => Ok(items),
            _ => Err(failure("invalid_request", "Unexpected hierarchy reply.")),
        }
    }
    pub(crate) async fn outgoing_calls(
        &self,
        item: CallHierarchyItem,
    ) -> Result<Vec<CallHierarchyOutgoingCall>, DocumentRequestError> {
        match self.call_hierarchy(Request::Outgoing(item)).await? {
            Reply::Outgoing(items) => Ok(items),
            _ => Err(failure("invalid_request", "Unexpected hierarchy reply.")),
        }
    }

    async fn call_hierarchy(&self, request: Request) -> Result<Reply, DocumentRequestError> {
        let (reply, versions, inventory) = tasks::run(&self.service, move |service, cancellation| {
            let reference = match &request {
                Request::Prepare(_) => None,
                Request::Incoming(item) | Request::Outgoing(item) => {
                    let data = item.data.as_ref().ok_or_else(|| failure("invalid_request", "Missing hierarchy data."))?;
                    if serde_json::to_vec(data).map_or(true, |data| data.len() > 8 * 1024 * 1024) {
                        return Err(failure("invalid_request", "Hierarchy data exceeds its input limit."));
                    }
                    Some(serde_json::from_value::<CallableReference>(data.clone())
                        .map_err(|_| failure("invalid_request", "Malformed hierarchy data."))?)
                }
            };
            if let Some(reference) = &reference { reference.validate()?; }
            let position = match &request {
                Request::Prepare(params) => Some((&params.text_document_position_params.text_document.uri, params.text_document_position_params.position)),
                _ => None,
            };
            let source = position.map(|(uri, _)| crate::convert::file_uri_to_path(uri)).transpose()
                .map_err(|e| failure("invalid_request", &e.to_string()))?;
            let input = reference.as_ref().map(|r| PathBuf::from(&r.context.input))
                .or_else(|| service.workspace().manifest_path().map(Path::to_path_buf)).or_else(|| source.clone())
                .ok_or_else(|| failure("invalid_request", "Missing hierarchy context."))?;
            let library = service.standard_library_root().map(Path::to_path_buf);
            let mut session = SourceSession::load(&input, library.as_deref(), &service.documents().open_snapshots(), cancellation)?;
            let versions = Versions::capture(service, &session);
            let reply = if let Some((_, position)) = position {
                let path = source.as_ref().ok_or_else(|| failure("invalid_request", "Missing source."))?;
                let snapshot = session.service().snapshot(path)?;
                let offset = crate::convert::position_to_byte_offset(&snapshot, position)?;
                let owner = session.callable_at(path, offset, cancellation)?;
                Reply::Prepare(owner.as_ref().map(|owner| hierarchy_item(&session, owner)).transpose()?.into_iter().collect())
            } else {
                let reference = reference.as_ref().ok_or_else(|| failure("invalid_request", "Missing callable identity."))?;
                let incoming = matches!(request, Request::Incoming(_));
                let calls = session.query_callable(reference, incoming, 1000, cancellation)?;
                if !calls.direct_edges_complete || !calls.direct_calls.items_complete {
                    return Err(failure("analysis_incomplete", "Direct call hierarchy is incomplete; use fpas/sourceQuery for coverage and bounded results."));
                }
                let mut grouped = BTreeMap::<CallableIdentity, (CallHierarchyItem, Vec<Range>)>::new();
                for relation in &calls.direct_calls.items {
                    cancellation.check()?;
                    let owner = if incoming { Some(&relation.containing) } else { relation.target.as_ref() };
                    let Some(owner) = owner.filter(|owner| owner.snapshot.is_some()) else { continue; };
                    let item = hierarchy_item(&session, owner)?;
                    let range = crate::navigation::span_range(&relation.snapshot, relation.span)
                        .map_err(|e| failure("invalid_location", &e.to_string()))?;
                    grouped.entry(owner.identity.clone()).or_insert_with(|| (item, Vec::new())).1.push(range);
                }
                for (_, ranges) in grouped.values_mut() {
                    ranges.sort_by_key(|r| (r.start.line, r.start.character, r.end.line, r.end.character));
                    ranges.dedup();
                }
                if incoming { Reply::Incoming(grouped.into_values().map(|(from, from_ranges)| CallHierarchyIncomingCall { from, from_ranges }).collect()) }
                else { Reply::Outgoing(grouped.into_values().map(|(to, from_ranges)| CallHierarchyOutgoingCall { to, from_ranges }).collect()) }
            };
            let bytes = match &reply {
                Reply::Prepare(items) => serde_json::to_vec(items),
                Reply::Incoming(items) => serde_json::to_vec(items),
                Reply::Outgoing(items) => serde_json::to_vec(items),
            }.map_err(|e| failure("invalid_location", &e.to_string()))?;
            if bytes.len() > 8 * 1024 * 1024 {
                return Err(failure("result_limit", "The complete standard hierarchy exceeds its byte limit; use the bounded fpas/sourceQuery projection."));
            }
            cancellation.check()?;
            if !session.verify()? { return Err(failure("source_changed", "Hierarchy sources changed during the request.")); }
            Ok((reply, versions, session.inventory().clone()))
        }).await?;
        self.verify_read_set(&versions, inventory).await?;
        Ok(reply)
    }
}

fn hierarchy_item(
    session: &SourceSession,
    owner: &SourceCallable,
) -> Result<CallHierarchyItem, DocumentRequestError> {
    let snapshot = owner.snapshot.as_ref().ok_or_else(|| {
        failure(
            "no_source",
            "Native owners have no hierarchy source location.",
        )
    })?;
    let span = owner
        .span
        .ok_or_else(|| failure("invalid_location", "Missing owner range."))?;
    let selection = owner
        .selection_span
        .ok_or_else(|| failure("invalid_location", "Missing owner selection."))?;
    Ok(CallHierarchyItem {
        name: owner.identity.qualified_name.clone(), kind: SymbolKind::FUNCTION, tags: None,
        detail: Some("Proven direct source calls; runtime uncertainty is available through fpas/sourceQuery.".into()),
        uri: Uri::from_file_path(snapshot.path()).ok_or_else(|| failure("invalid_location", "Invalid source URI."))?,
        range: crate::navigation::span_range(snapshot, span).map_err(|e| failure("invalid_location", &e.to_string()))?,
        selection_range: crate::navigation::span_range(snapshot, selection).map_err(|e| failure("invalid_location", &e.to_string()))?,
        data: Some(serde_json::to_value(session.callable_reference(owner)).map_err(|e| failure("invalid_request", &e.to_string()))?),
    })
}
fn failure(code: &'static str, message: &str) -> DocumentRequestError {
    SourceQueryError {
        code,
        message: message.into(),
        diagnostics: Vec::new(),
    }
    .into()
}
