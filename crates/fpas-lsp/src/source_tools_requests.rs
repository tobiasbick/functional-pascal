//! Rich source queries use the CLI's frozen core and revalidate all participating buffers.

mod bounds;
mod model;
mod projection;
use model::*;
pub(crate) use model::{SourceQueryParams, SourceQueryResponse};
pub(crate) use projection::diagnostics;

use crate::documents::read_set::Versions;
use crate::documents::{DocumentRequestError, SynchronizedDocuments, tasks};
use fpas_language_service::{QueryPolicy, SourceQueryError, SourceQueryOperation, SourceSession};
use std::path::{Path, PathBuf};
use tower_lsp_server::ls_types::Uri;

impl SynchronizedDocuments {
    pub(crate) async fn source_query(
        &self,
        params: SourceQueryParams,
    ) -> Result<SourceQueryResponse, DocumentRequestError> {
        let (mut response, versions, inventory) = tasks::run(&self.service, move |service, cancellation| {
            if params.version != 1 || params.limit.is_some_and(|n| n == 0 || n > 10000) {
                return Err(invalid("Unsupported source query version or limit."));
            }
            let policy = params.policy.unwrap_or(QueryPolicy::CheckedContext);
            if policy == QueryPolicy::EditorCatalog && params.operation != SourceOperation::Find {
                return Err(invalid("Declaration selection requires checked_context policy."));
            }
            if (params.operation == SourceOperation::Find) != matches!(params.selector, SourceSelector::Search { .. }) {
                return Err(invalid("Find requires search text; declaration queries require a position, symbol or reference."));
            }
            if let SourceSelector::Reference { target_ref } = &params.selector { target_ref.validate()?; }
            let explicit = params.context.as_ref().map(uri_path).transpose()?;
            let input = match (&params.selector, explicit) {
                (_, Some(path)) => path,
                (SourceSelector::Position { uri, .. }, None) => uri_path(uri)?,
                (SourceSelector::Reference { target_ref }, None) => PathBuf::from(&target_ref.context.input),
                (_, None) if policy == QueryPolicy::EditorCatalog => service.workspace().manifest_path().unwrap_or(service.workspace().root()).into(),
                _ => return Err(invalid("Name search and selection require an explicit manifest context.")),
            };
            let library = service.standard_library_root().map(Path::to_path_buf);
            // Editor configuration is authoritative; reference reuse checks that it still agrees.
            let mut session = SourceSession::load_with_policy(&input, library.as_deref(), &service.documents().open_snapshots(), policy, cancellation)?;
            let mut response = SourceQueryResponse { version: 1, outcome: "ok", context: session.context().clone(), snapshot: session.snapshot().clone(),
                target: None, candidates: Vec::new(), result: SourcePayload::Find { symbols: Vec::new() }, diagnostics: Vec::new(),
                coverage: SourceCoverage::default() };
            let limit = params.limit.unwrap_or(if params.operation == SourceOperation::Find { 100 } else { 1000 });
            if let SourceSelector::Search { text } = &params.selector {
                let result = if policy == QueryPolicy::EditorCatalog { session.find_catalog(text, limit, cancellation)? }
                    else { session.find_declarations(text, false, limit, cancellation)? };
                projection::find(&mut response, result)?;
            } else {
                let operation = match params.operation { SourceOperation::Refs => SourceQueryOperation::References,
                    SourceOperation::Inspect => SourceQueryOperation::Inspect,
                    SourceOperation::Callers => SourceQueryOperation::Callers,
                    SourceOperation::Callees => SourceQueryOperation::Callees,
                    SourceOperation::Definition => SourceQueryOperation::Definition, SourceOperation::Type => SourceQueryOperation::Type,
                    SourceOperation::Find => return Err(invalid("Unexpected find operation.")) };
                let (result, selected) = match &params.selector {
                    SourceSelector::Position { uri, position } => {
                        let path = uri_path(uri)?;
                        let selected = session.service().snapshot(&path)?;
                        let offset = crate::convert::position_to_byte_offset(&selected, *position)?;
                        (session.query_position(&path, offset, operation, limit, cancellation)?, Some(selected))
                    }
                    SourceSelector::Reference { target_ref } => {
                        let result = session.query_reference(target_ref, operation, limit, cancellation)?;
                        let selected = result.target.as_ref().and_then(|t| t.snapshot.clone()); (result, selected)
                    }
                    SourceSelector::Symbol { name } => {
                        let result = session.query_symbol(name, operation, limit, cancellation)?;
                        let selected = result.target.as_ref().and_then(|t| t.snapshot.clone()); (result, selected)
                    }
                    SourceSelector::Search { .. } => return Err(invalid("Unexpected search selector.")),
                };
                projection::query(&mut response, result, selected.as_deref(), params.operation)?;
            }
            bounds::apply(&mut response)?;
            cancellation.check()?;
            if !session.verify()? { invalidate(&mut response, "source_changed"); }
            Ok((response, Versions::capture(service, &session), session.inventory().clone()))
        }).await?;
        if let Err(error) = self.verify_read_set(&versions, inventory).await {
            match error {
                DocumentRequestError::SourceTool(error) if error.code == "source_changed" => {
                    invalidate(&mut response, "source_changed")
                }
                error => return Err(error),
            }
        }
        Ok(response)
    }
}

fn invalidate(response: &mut SourceQueryResponse, reason: &'static str) {
    response.outcome = "incomplete";
    response.coverage.analysis_complete = false;
    response.coverage.reasons.insert(reason.into());
    for collection in response.coverage.collections.values_mut() {
        collection.complete = false;
        collection.known_total = None;
    }
    let invalidate = |target: &mut SourceTarget| {
        target.target_ref = None;
        target.reference_unavailable = Some(reason);
    };
    if let Some(target) = &mut response.target {
        invalidate(target);
    }
    for candidate in &mut response.candidates {
        invalidate(candidate);
    }
    if let SourcePayload::Find { symbols } = &mut response.result {
        for target in symbols {
            invalidate(target);
        }
    }
    if let SourcePayload::Callers { calls } | SourcePayload::Callees { calls } =
        &mut response.result
    {
        calls.direct_edges_complete = false;
        calls.runtime_targets_complete = false;
        for collection in calls.collections.values_mut() {
            collection.complete = false;
            collection.known_total = None;
        }
    }
}

fn uri_path(uri: &Uri) -> Result<PathBuf, DocumentRequestError> {
    crate::convert::file_uri_to_path(uri).map_err(|e| invalid(&e.to_string()))
}
fn invalid(message: &str) -> DocumentRequestError {
    SourceQueryError {
        code: "invalid_request",
        message: message.into(),
        diagnostics: Vec::new(),
    }
    .into()
}
