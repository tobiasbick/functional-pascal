//! Pattern declarations shared by statement and expression decisions.
//! See `docs/pascal/language/pattern-matching/is-test.md`.

use fpas_diagnostics::SourceSpan;
use fpas_lexer::Span;
use fpas_parser::{CaseLabel, Pattern, Visibility};

use super::member_symbol;
use crate::{DocumentSnapshot, DocumentSymbol, SymbolKind};

/// Extracts arm binders that are visible only after the labels have matched.
pub(super) fn collect_case_bindings(
    snapshot: &DocumentSnapshot,
    owner: &str,
    labels: &[CaseLabel],
    scope_span: SourceSpan,
    output: &mut Vec<DocumentSymbol>,
) {
    let visible_from = labels
        .iter()
        .map(|label| match label {
            CaseLabel::Value { span, .. } | CaseLabel::Binding { span, .. } => {
                span.diagnostic_span_or_synthetic().end()
            }
            CaseLabel::Pattern(pattern) => pattern.span().diagnostic_span_or_synthetic().end(),
        })
        .max()
        .unwrap_or(scope_span.offset());
    for label in labels {
        match label {
            CaseLabel::Binding { name, span } => {
                push_binding(
                    snapshot,
                    owner,
                    name,
                    *span,
                    scope_span,
                    visible_from,
                    output,
                );
            }
            CaseLabel::Pattern(pattern) => {
                collect_pattern(snapshot, owner, pattern, scope_span, visible_from, output);
            }
            CaseLabel::Value { .. } => {}
        }
    }
}

fn collect_pattern(
    snapshot: &DocumentSnapshot,
    owner: &str,
    pattern: &Pattern,
    scope_span: SourceSpan,
    visible_from: usize,
    output: &mut Vec<DocumentSymbol>,
) {
    match pattern {
        Pattern::Binding { name, span } => {
            push_binding(
                snapshot,
                owner,
                name,
                *span,
                scope_span,
                visible_from,
                output,
            );
        }
        Pattern::Variant { fields, .. } | Pattern::Destructure { fields, .. } => {
            for field in fields {
                collect_pattern(
                    snapshot,
                    owner,
                    &field.pattern,
                    scope_span,
                    visible_from,
                    output,
                );
            }
        }
        Pattern::Wildcard(_) | Pattern::Value(_) => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn push_binding(
    snapshot: &DocumentSnapshot,
    owner: &str,
    name: &str,
    span: Span,
    scope_span: SourceSpan,
    visible_from: usize,
    output: &mut Vec<DocumentSymbol>,
) {
    let mut symbol = member_symbol(
        snapshot,
        owner,
        name,
        SymbolKind::Constant,
        span,
        Visibility::Private,
        None,
        format!("const {name}"),
        scope_span,
        Vec::new(),
    );
    symbol.visible_from = visible_from;
    output.push(symbol);
}
