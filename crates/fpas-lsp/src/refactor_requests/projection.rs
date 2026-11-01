//! Convert only checked core edits to versioned standard LSP edits.

use super::{EditorRecipe, EditorSelector};
use crate::documents::{DocumentRequestError, read_set::failure};
use fpas_language_service::{
    RefactorRecipe, SourceEdit, SourceSelector, SourceSession, identity_path,
};
use std::collections::BTreeMap;
use tower_lsp_server::ls_types::{
    DocumentChanges, OneOf, OptionalVersionedTextDocumentIdentifier, TextDocumentEdit, TextEdit,
    Uri, WorkspaceEdit,
};

pub(super) fn recipe(
    session: &mut SourceSession,
    recipe: EditorRecipe,
) -> Result<RefactorRecipe, DocumentRequestError> {
    Ok(match recipe {
        EditorRecipe::AddParameter {
            selector,
            name,
            r#type,
            mode,
            argument,
        } => RefactorRecipe::AddParameter {
            selector: select(session, selector)?,
            name,
            r#type,
            mode,
            argument,
        },
        EditorRecipe::Rename { selector, name } => RefactorRecipe::Rename {
            selector: select(session, selector)?,
            name,
        },
        EditorRecipe::ExtractVariable { uri, range } => {
            let path = super::uri_path(&uri)?;
            let snapshot = session.service().snapshot(&path)?;
            RefactorRecipe::ExtractVariable {
                path: identity_path(&path)
                    .map_err(|e| failure("invalid_location", &e.to_string()))?,
                start: crate::convert::position_to_byte_offset(&snapshot, range.start)?,
                end: crate::convert::position_to_byte_offset(&snapshot, range.end)?,
                name: None,
            }
        }
        EditorRecipe::OrganizeImports { uri } => RefactorRecipe::OrganizeImports {
            path: identity_path(&super::uri_path(&uri)?)
                .map_err(|e| failure("invalid_location", &e.to_string()))?,
        },
    })
}

fn select(
    session: &mut SourceSession,
    selector: EditorSelector,
) -> Result<SourceSelector, DocumentRequestError> {
    Ok(match selector {
        EditorSelector::Position { uri, position } => {
            let path = super::uri_path(&uri)?;
            let snapshot = session.service().snapshot(&path)?;
            SourceSelector::Position {
                path: identity_path(&path)
                    .map_err(|e| failure("invalid_location", &e.to_string()))?,
                offset: crate::convert::position_to_byte_offset(&snapshot, position)?,
            }
        }
        EditorSelector::Symbol { name } => SourceSelector::Symbol { name },
        EditorSelector::Reference { target_ref } => SourceSelector::Reference { target_ref },
    })
}

pub(crate) fn workspace_edit(
    edits: Vec<SourceEdit>,
) -> Result<WorkspaceEdit, DocumentRequestError> {
    let mut changes = BTreeMap::<String, (Uri, Option<i32>, Vec<TextEdit>)>::new();
    for edit in edits {
        let (uri, version, text) = crate::navigation::rename_edit(edit)
            .map_err(|e| failure("invalid_location", &e.to_string()))?;
        changes
            .entry(uri.to_string())
            .or_insert_with(|| (uri, version, Vec::new()))
            .2
            .push(text);
    }
    Ok(WorkspaceEdit {
        document_changes: Some(DocumentChanges::Edits(
            changes
                .into_values()
                .map(|(uri, version, edits)| TextDocumentEdit {
                    text_document: OptionalVersionedTextDocumentIdentifier { uri, version },
                    edits: edits.into_iter().map(OneOf::Left).collect(),
                })
                .collect(),
        )),
        ..WorkspaceEdit::default()
    })
}
