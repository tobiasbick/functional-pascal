//! Resolved identifier classification over recovered source snapshots.

use std::path::Path;

use fpas_diagnostics::SourceSpan;
use fpas_lexer::Token;

use super::{SemanticToken, SemanticTokenKind, SemanticTokenModifiers};
use crate::navigation::{NavigationDocument, NavigationResult, resolve};
use crate::{LanguageService, LanguageServiceError, SymbolKind, SymbolVisibility};

impl LanguageService {
    /// Returns stable full-document semantic identifiers for the current snapshot.
    pub fn semantic_tokens(
        &mut self,
        path: &Path,
    ) -> Result<NavigationResult<Vec<SemanticToken>>, LanguageServiceError> {
        let context = self.navigation_context(path)?;
        let value = context
            .target_index
            .map(|target_index| classify_document(&context.documents, target_index))
            .unwrap_or_default();
        Ok(NavigationResult {
            snapshot: context.snapshot,
            value,
        })
    }
}

fn classify_document(documents: &[NavigationDocument], target_index: usize) -> Vec<SemanticToken> {
    let target = &documents[target_index];
    target
        .tokens
        .iter()
        .enumerate()
        .filter_map(|(token_index, token)| {
            let Token::Ident(_) = &token.token else {
                return None;
            };
            let span = token.span.diagnostic_span_or_synthetic();
            if let Some((document_index, symbol, _)) =
                resolve(documents, target_index, span.offset())
            {
                let declaration = document_index == target_index
                    && contains(symbol.selection_span, span.offset());
                return Some(SemanticToken {
                    span,
                    kind: token_kind(symbol.kind),
                    modifiers: SemanticTokenModifiers {
                        declaration,
                        readonly: readonly(symbol.kind, &symbol.detail),
                        public: symbol.visibility == SymbolVisibility::Public,
                    },
                });
            }
            namespace_component(documents, target_index, token_index).then_some(SemanticToken {
                span,
                kind: SemanticTokenKind::Namespace,
                modifiers: SemanticTokenModifiers::default(),
            })
        })
        .collect()
}

fn token_kind(kind: SymbolKind) -> SemanticTokenKind {
    match kind {
        SymbolKind::Program | SymbolKind::Unit => SemanticTokenKind::Namespace,
        SymbolKind::Constant => SemanticTokenKind::Constant,
        SymbolKind::Variable | SymbolKind::MutableVariable | SymbolKind::LoopVariable => {
            SemanticTokenKind::Variable
        }
        SymbolKind::Type => SemanticTokenKind::Type,
        SymbolKind::Enum => SemanticTokenKind::Enum,
        SymbolKind::Function => SemanticTokenKind::Function,
        SymbolKind::Procedure => SemanticTokenKind::Procedure,
        SymbolKind::Method => SemanticTokenKind::Method,
        SymbolKind::TypeParameter => SemanticTokenKind::TypeParameter,
        SymbolKind::Parameter => SemanticTokenKind::Parameter,
        SymbolKind::Field => SemanticTokenKind::Field,
        SymbolKind::Property => SemanticTokenKind::Property,
        SymbolKind::Event => SemanticTokenKind::Event,
        SymbolKind::EnumMember => SemanticTokenKind::EnumMember,
    }
}

fn readonly(kind: SymbolKind, detail: &str) -> bool {
    matches!(
        kind,
        SymbolKind::Constant | SymbolKind::Variable | SymbolKind::EnumMember
    ) || (kind == SymbolKind::Parameter && !detail.starts_with("mutable parameter "))
}

fn namespace_component(
    documents: &[NavigationDocument],
    target_index: usize,
    selected: usize,
) -> bool {
    let target = &documents[target_index];
    let mut start = selected;
    while start >= 2
        && matches!(target.tokens[start - 1].token, Token::Dot)
        && matches!(target.tokens[start - 2].token, Token::Ident(_))
    {
        start -= 2;
    }
    let mut names = Vec::<&str>::new();
    let mut selected_part = None;
    let mut index = start;
    while let Some(Token::Ident(name)) = target.tokens.get(index).map(|token| &token.token) {
        if index == selected {
            selected_part = Some(names.len());
        }
        names.push(name);
        if !matches!(
            target.tokens.get(index + 1).map(|token| &token.token),
            Some(Token::Dot)
        ) {
            break;
        }
        index += 2;
    }
    let selected_part = selected_part.unwrap_or(usize::MAX);
    documents.iter().any(|document| {
        if !target.uses_owner(&document.owner) {
            return false;
        }
        let owner = document.owner.split('.').collect::<Vec<_>>();
        selected_part < owner.len()
            && names.len() >= owner.len()
            && names[..owner.len()]
                .iter()
                .zip(owner)
                .all(|(left, right)| left.eq_ignore_ascii_case(right))
    })
}

fn contains(span: SourceSpan, offset: usize) -> bool {
    span.offset() <= offset && offset < span.end()
}
