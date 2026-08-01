//! Same-document declaration, read, and write highlights.

use std::path::Path;

use fpas_diagnostics::SourceSpan;
use fpas_parser::{CompilationUnit, Decl, DesignatorPart, FuncBody, RecordMethod, Stmt, TypeBody};

use super::{find_references, resolve_target};
use crate::{CancellationToken, LanguageService, LanguageServiceError, NavigationResult};

/// Semantic category for one highlighted symbol occurrence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighlightKind {
    /// The defining declaration.
    Declaration,
    /// A value read.
    Read,
    /// An assignment target.
    Write,
}

/// One resolved occurrence highlighted in the current document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DocumentHighlight {
    /// Exact identifier source range.
    pub span: SourceSpan,
    /// Resolved access category.
    pub kind: HighlightKind,
}

impl LanguageService {
    /// Returns resolved occurrences of the selected symbol in the requesting document.
    pub fn document_highlights(
        &mut self,
        path: &Path,
        offset: usize,
    ) -> Result<NavigationResult<Vec<DocumentHighlight>>, LanguageServiceError> {
        let context = self.navigation_context(path)?;
        let Some(target_index) = context.target_index else {
            return Ok(NavigationResult {
                snapshot: context.snapshot,
                value: Vec::new(),
            });
        };
        let Some(target) = resolve_target(&context.documents, target_index, offset) else {
            return Ok(NavigationResult {
                snapshot: context.snapshot,
                value: Vec::new(),
            });
        };
        let references =
            find_references(&context.documents, &target, true, &CancellationToken::new())?;
        let document = &context.documents[target_index];
        let value = references
            .into_iter()
            .filter(|reference| reference.path == document.path)
            .map(|reference| DocumentHighlight {
                span: reference.span,
                kind: if reference.is_declaration {
                    HighlightKind::Declaration
                } else if is_write(document, reference.span) {
                    HighlightKind::Write
                } else {
                    HighlightKind::Read
                },
            })
            .collect();
        Ok(NavigationResult {
            snapshot: context.snapshot,
            value,
        })
    }
}

fn is_write(document: &super::NavigationDocument, span: SourceSpan) -> bool {
    match document.snapshot.compilation_unit() {
        CompilationUnit::Program(program) => {
            declarations_write(&program.declarations, span) || statements_write(&program.body, span)
        }
        CompilationUnit::Unit(unit) => declarations_write(&unit.declarations, span),
    }
}

fn declarations_write(declarations: &[Decl], span: SourceSpan) -> bool {
    declarations.iter().any(|declaration| match declaration {
        Decl::Function(function) => body_writes(&function.body, span),
        Decl::Procedure(procedure) => body_writes(&procedure.body, span),
        Decl::TypeDef(definition) => match &definition.body {
            TypeBody::Record(record) => record.methods.iter().any(|method| match method {
                RecordMethod::Function(function) | RecordMethod::StaticFunction(function) => {
                    body_writes(&function.body, span)
                }
                RecordMethod::Procedure(procedure) | RecordMethod::StaticProcedure(procedure) => {
                    body_writes(&procedure.body, span)
                }
            }),
            TypeBody::Alias(_) | TypeBody::Enum(_) => false,
        },
        Decl::Const(_) | Decl::Var(_) | Decl::MutableVar(_) => false,
    })
}

fn body_writes(body: &FuncBody, span: SourceSpan) -> bool {
    let FuncBody::Block { nested, stmts } = body;
    declarations_write(nested, span) || statements_write(stmts, span)
}

fn statements_write(statements: &[Stmt], span: SourceSpan) -> bool {
    statements.iter().any(|statement| match statement {
        Stmt::Block(statements, _) => statements_write(statements, span),
        Stmt::Assign { target, .. } => target.parts.iter().any(|part| {
            matches!(part, DesignatorPart::Ident(_, part_span) if SourceSpan::from(*part_span) == span)
        }),
        Stmt::If {
            then_branch,
            else_branch,
            ..
        } => {
            statements_write(std::slice::from_ref(then_branch), span)
                || else_branch.as_ref().is_some_and(|else_branch| {
                    statements_write(std::slice::from_ref(else_branch), span)
                })
        }
        Stmt::Case {
            arms, else_body, ..
        } => {
            arms.iter()
                .any(|arm| statements_write(std::slice::from_ref(&arm.body), span))
                || else_body
                    .as_ref()
                    .is_some_and(|body| statements_write(body, span))
        }
        Stmt::For { body, .. } | Stmt::ForIn { body, .. } | Stmt::While { body, .. } => {
            statements_write(std::slice::from_ref(body), span)
        }
        Stmt::Repeat { body, .. } => statements_write(body, span),
        Stmt::Var(_)
        | Stmt::MutableVar(_)
        | Stmt::Return(_, _)
        | Stmt::Panic(_, _)
        | Stmt::Break(_)
        | Stmt::Continue(_)
        | Stmt::Call { .. }
        | Stmt::Expression { .. }
        | Stmt::Go { .. } => false,
    })
}
