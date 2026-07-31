//! Validation and edit generation for project-aware symbol rename.

use std::fmt;
use std::path::{Path, PathBuf};

use fpas_diagnostics::SourceSpan;
use fpas_lexer::{Token, lex};

use super::NavigationDocument;
use super::references::{ResolvedTarget, find_references, resolve_target};
use crate::{CancellationToken, LanguageServiceError, SymbolKind};

/// The source token selected by a successful prepare-rename query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameTarget {
    /// Identifier range to select in the requesting document.
    pub range: SourceSpan,
    /// Current source spelling shown by the editor.
    pub placeholder: String,
}

/// One protocol-independent source edit produced by rename.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameEdit {
    /// Source file to edit.
    pub path: PathBuf,
    /// Identifier range to replace.
    pub range: SourceSpan,
    /// Validated replacement identifier.
    pub new_text: String,
}

/// A recoverable reason why a requested rename cannot be performed safely.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenameError {
    /// Loading a source snapshot or project context failed.
    Service(LanguageServiceError),
    /// No renameable declaration resolves at the requested source position.
    NoSymbol,
    /// Programs and units require file or manifest changes and are not text-only renames.
    CompilationUnit,
    /// The declaration is outside the opened editor root.
    OutsideWorkspace {
        /// Declaration source that would otherwise be edited.
        path: PathBuf,
    },
    /// The replacement is not an ordinary Functional Pascal identifier.
    InvalidIdentifier {
        /// Rejected replacement text.
        name: String,
    },
    /// Another declaration in the same lexical scope already has the replacement name.
    Conflict {
        /// Requested replacement name.
        name: String,
    },
}

impl fmt::Display for RenameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Service(error) => error.fmt(formatter),
            Self::NoSymbol => formatter.write_str(
                "No renameable Functional Pascal declaration resolves at this position.",
            ),
            Self::CompilationUnit => formatter.write_str(
                "Program and unit names cannot be renamed because that also requires file or manifest changes.",
            ),
            Self::OutsideWorkspace { path } => write!(
                formatter,
                "Cannot rename a declaration outside the opened editor folder: `{}`.",
                path.display()
            ),
            Self::InvalidIdentifier { name } => write!(
                formatter,
                "`{name}` is not a valid rename target. Use an ASCII identifier beginning with a letter or `_`, and do not use a Pascal keyword."
            ),
            Self::Conflict { name } => write!(
                formatter,
                "Cannot rename to `{name}` because that name is already declared in the same scope."
            ),
        }
    }
}

impl std::error::Error for RenameError {}

impl From<LanguageServiceError> for RenameError {
    fn from(error: LanguageServiceError) -> Self {
        Self::Service(error)
    }
}

pub(crate) fn prepare_rename(
    documents: &[NavigationDocument],
    target_index: usize,
    offset: usize,
    workspace_root: &Path,
) -> Option<RenameTarget> {
    let target = resolve_target(documents, target_index, offset)?;
    renameable_target(documents, &target, workspace_root).ok()?;
    let document = &documents[target_index];
    let end = target
        .occurrence_span
        .offset
        .saturating_add(target.occurrence_span.length);
    let placeholder = document
        .snapshot
        .source()
        .get(target.occurrence_span.offset..end)?
        .to_owned();
    Some(RenameTarget {
        range: target.occurrence_span,
        placeholder,
    })
}

pub(crate) fn rename_symbol(
    documents: &[NavigationDocument],
    target_index: usize,
    offset: usize,
    workspace_root: &Path,
    new_name: &str,
    cancellation: &CancellationToken,
) -> Result<Vec<RenameEdit>, RenameError> {
    cancellation.check()?;
    validate_identifier(new_name)?;
    let target = resolve_target(documents, target_index, offset).ok_or(RenameError::NoSymbol)?;
    renameable_target(documents, &target, workspace_root)?;
    reject_same_scope_conflict(documents, &target, new_name)?;

    let mut edits = find_references(documents, &target, true, cancellation)?
        .into_iter()
        .map(|location| RenameEdit {
            path: location.path,
            range: location.span,
            new_text: new_name.to_owned(),
        })
        .collect::<Vec<_>>();
    edits.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| right.range.offset.cmp(&left.range.offset))
    });
    Ok(edits)
}

fn renameable_target(
    documents: &[NavigationDocument],
    target: &ResolvedTarget,
    workspace_root: &Path,
) -> Result<(), RenameError> {
    if matches!(target.symbol.kind, SymbolKind::Program | SymbolKind::Unit) {
        return Err(RenameError::CompilationUnit);
    }
    let declaration_path = &documents[target.document_index].path;
    if !declaration_path.starts_with(workspace_root) {
        return Err(RenameError::OutsideWorkspace {
            path: declaration_path.clone(),
        });
    }
    Ok(())
}

fn validate_identifier(name: &str) -> Result<(), RenameError> {
    let (tokens, errors) = lex(name);
    let valid = errors.is_empty()
        && matches!(
            tokens.as_slice(),
            [token, eof]
                if matches!(&token.token, Token::Ident(value) if value == name)
                    && token.span.offset == 0
                    && token.span.length == name.len()
                    && matches!(eof.token, Token::Eof)
        );
    if valid {
        Ok(())
    } else {
        Err(RenameError::InvalidIdentifier {
            name: name.to_owned(),
        })
    }
}

fn reject_same_scope_conflict(
    documents: &[NavigationDocument],
    target: &ResolvedTarget,
    new_name: &str,
) -> Result<(), RenameError> {
    let target_document = &documents[target.document_index];
    let conflict = target_document.all_symbols().into_iter().any(|symbol| {
        symbol.selection_span != target.symbol.selection_span
            && symbol.name.eq_ignore_ascii_case(new_name)
            && symbol.scope_span == target.symbol.scope_span
    });
    if conflict {
        Err(RenameError::Conflict {
            name: new_name.to_owned(),
        })
    } else {
        Ok(())
    }
}
