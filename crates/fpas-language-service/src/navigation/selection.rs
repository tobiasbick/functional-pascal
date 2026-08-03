//! Syntax-aware nested source selection ranges.

use std::path::Path;
use std::sync::Arc;

use fpas_diagnostics::SourceSpan;
use fpas_parser::{CompilationUnit, Decl, FuncBody, RecordMethod, Stmt, TypeBody};

use crate::{DocumentSnapshot, DocumentSymbols, LanguageService, LanguageServiceError};

/// One source range with an optional strictly containing parent range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionRange {
    /// Current selection boundary.
    pub span: SourceSpan,
    /// Next larger syntax boundary.
    pub parent: Option<Box<SelectionRange>>,
}

impl LanguageService {
    /// Returns one nested selection chain for every requested UTF-8 byte offset.
    pub fn selection_ranges(
        &mut self,
        path: &Path,
        offsets: &[usize],
    ) -> Result<(Arc<DocumentSnapshot>, Vec<SelectionRange>), LanguageServiceError> {
        self.ensure_source_context(path)?;
        let snapshot = self.snapshot(path)?;
        let ranges = offsets
            .iter()
            .map(|offset| selection_range(&snapshot, *offset))
            .collect();
        Ok((snapshot, ranges))
    }
}

fn selection_range(snapshot: &DocumentSnapshot, offset: usize) -> SelectionRange {
    let offset = offset.min(snapshot.source().len());
    let mut spans = vec![token_or_cursor_span(snapshot, offset)];
    if !snapshot.has_parse_errors() {
        collect_symbol_spans(snapshot, offset, &mut spans);
        collect_syntax_spans(snapshot.compilation_unit(), offset, &mut spans);
    }
    spans.retain(|span| valid_span(snapshot, *span) && contains_offset(*span, offset));
    spans.sort_by(|left, right| {
        left.length()
            .cmp(&right.length())
            .then_with(|| right.offset().cmp(&left.offset()))
    });
    spans
        .dedup_by(|left, right| left.offset() == right.offset() && left.length() == right.length());

    let mut nested = Vec::new();
    for span in spans {
        if nested
            .last()
            .is_none_or(|child| contains_span(span, *child))
        {
            nested.push(span);
        }
    }
    let mut parent = None;
    for span in nested.into_iter().rev() {
        parent = Some(Box::new(SelectionRange { span, parent }));
    }
    parent.map(|range| *range).unwrap_or(SelectionRange {
        span: SourceSpan::new(offset, 0, 1, 1),
        parent: None,
    })
}

fn token_or_cursor_span(snapshot: &DocumentSnapshot, offset: usize) -> SourceSpan {
    let (tokens, _) = fpas_lexer::lex(snapshot.source());
    tokens
        .into_iter()
        .find(|token| {
            token.span.length > 0
                && token.span.offset <= offset
                && offset < token.span.offset.saturating_add(token.span.length)
        })
        .map(|token| token.span.diagnostic_span_or_synthetic())
        .unwrap_or_else(|| SourceSpan::new(offset, 0, 1, 1))
}

fn collect_symbol_spans(snapshot: &DocumentSnapshot, offset: usize, spans: &mut Vec<SourceSpan>) {
    for symbol in DocumentSymbols::from_snapshot(snapshot)
        .entries()
        .iter()
        .flat_map(all_symbols)
    {
        if contains_offset(symbol.full_span, offset) {
            spans.push(symbol.full_span);
        }
    }
}

fn collect_syntax_spans(unit: &CompilationUnit, offset: usize, spans: &mut Vec<SourceSpan>) {
    match unit {
        CompilationUnit::Program(program) => {
            spans.push(program.span.diagnostic_span_or_synthetic());
            collect_declarations(&program.declarations, offset, spans);
            collect_statements(&program.body, offset, spans);
        }
        CompilationUnit::Unit(unit) => {
            spans.push(unit.span.diagnostic_span_or_synthetic());
            collect_declarations(&unit.declarations, offset, spans);
        }
    }
}

fn collect_declarations(declarations: &[Decl], offset: usize, spans: &mut Vec<SourceSpan>) {
    for declaration in declarations {
        match declaration {
            Decl::Function(function) => collect_body(&function.body, offset, spans),
            Decl::Procedure(procedure) => collect_body(&procedure.body, offset, spans),
            Decl::TypeDef(definition) => {
                if let TypeBody::Record(record) = &definition.body {
                    for method in &record.methods {
                        match method {
                            RecordMethod::Function(function)
                            | RecordMethod::StaticFunction(function) => {
                                collect_body(&function.body, offset, spans);
                            }
                            RecordMethod::Procedure(procedure)
                            | RecordMethod::StaticProcedure(procedure) => {
                                collect_body(&procedure.body, offset, spans);
                            }
                        }
                    }
                }
            }
            Decl::Const(_) | Decl::Var(_) | Decl::MutableVar(_) => {}
        }
    }
}

fn collect_body(body: &FuncBody, offset: usize, spans: &mut Vec<SourceSpan>) {
    let FuncBody::Block { nested, stmts } = body;
    collect_declarations(nested, offset, spans);
    collect_statements(stmts, offset, spans);
}

fn collect_statements(statements: &[Stmt], offset: usize, spans: &mut Vec<SourceSpan>) {
    for statement in statements {
        let span = statement_span(statement);
        if !contains_offset(span, offset) {
            continue;
        }
        spans.push(span);
        match statement {
            Stmt::Block(statements, _) => collect_statements(statements, offset, spans),
            Stmt::If {
                then_branch,
                else_branch,
                ..
            } => {
                collect_statements(std::slice::from_ref(then_branch), offset, spans);
                if let Some(else_branch) = else_branch {
                    collect_statements(std::slice::from_ref(else_branch), offset, spans);
                }
            }
            Stmt::Case {
                arms, else_body, ..
            } => {
                for arm in arms {
                    collect_statements(std::slice::from_ref(&arm.body), offset, spans);
                }
                if let Some(else_body) = else_body {
                    collect_statements(else_body, offset, spans);
                }
            }
            Stmt::For { body, .. } | Stmt::ForIn { body, .. } | Stmt::While { body, .. } => {
                collect_statements(std::slice::from_ref(body), offset, spans);
            }
            Stmt::Repeat { body, .. } => collect_statements(body, offset, spans),
            Stmt::Var(_)
            | Stmt::MutableVar(_)
            | Stmt::Assign { .. }
            | Stmt::Return(_, _)
            | Stmt::Panic(_, _)
            | Stmt::Break(_)
            | Stmt::Continue(_)
            | Stmt::Call { .. }
            | Stmt::Expression { .. }
            | Stmt::Go { .. } => {}
        }
    }
}

fn statement_span(statement: &Stmt) -> SourceSpan {
    match statement {
        Stmt::Block(_, span)
        | Stmt::Return(_, span)
        | Stmt::Panic(_, span)
        | Stmt::Break(span)
        | Stmt::Continue(span) => span.diagnostic_span_or_synthetic(),
        Stmt::Var(value) | Stmt::MutableVar(value) => value.span.diagnostic_span_or_synthetic(),
        Stmt::Assign { span, .. }
        | Stmt::If { span, .. }
        | Stmt::Case { span, .. }
        | Stmt::For { span, .. }
        | Stmt::ForIn { span, .. }
        | Stmt::While { span, .. }
        | Stmt::Repeat { span, .. }
        | Stmt::Call { span, .. }
        | Stmt::Expression { span, .. }
        | Stmt::Go { span, .. } => span.diagnostic_span_or_synthetic(),
    }
}

fn all_symbols(
    symbol: &crate::DocumentSymbol,
) -> Box<dyn Iterator<Item = &crate::DocumentSymbol> + '_> {
    Box::new(std::iter::once(symbol).chain(symbol.children.iter().flat_map(all_symbols)))
}

fn valid_span(snapshot: &DocumentSnapshot, span: SourceSpan) -> bool {
    span.offset() <= snapshot.source().len() && span.end() <= snapshot.source().len()
}

fn contains_offset(span: SourceSpan, offset: usize) -> bool {
    span.offset() <= offset && offset <= span.end()
}

fn contains_span(parent: SourceSpan, child: SourceSpan) -> bool {
    parent.offset() <= child.offset() && child.end() <= parent.end()
}
