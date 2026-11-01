//! Traverse actual checked allocations, retaining independent owners for every source body.

mod expressions;
mod statements;

use super::{index::Index, model::*, resolution};
use crate::{CancellationToken, DocumentAnalysis, SourceQueryError};
use fpas_lexer::Span;
use fpas_parser::{
    CompilationUnit, Decl, FuncBody, FunctionDecl, ProcedureDecl, RecordMethod, TypeBody,
};
use fpas_sema::AnalysisMetadata;
use std::{collections::BTreeSet, sync::Arc};

pub(super) struct Walker<'a> {
    analysis: &'a DocumentAnalysis,
    metadata: &'a AnalysisMetadata,
    index: &'a Index,
    context: &'a str,
    cancellation: &'a CancellationToken,
    emit: &'a mut dyn FnMut(CallRelation) -> Result<(), SourceQueryError>,
    owner: &'a mut dyn FnMut(&Arc<SourceCallable>),
    pub(super) reasons: BTreeSet<&'static str>,
}

struct Callback {
    receiving: Option<Arc<SourceCallable>>,
    parameter: Option<usize>,
}

struct CallSyntax {
    invocation: InvocationKind,
    arguments: Vec<CallArgument>,
    arguments_complete: bool,
}

impl CallSyntax {
    fn value() -> Self {
        Self {
            invocation: InvocationKind::Ordinary,
            arguments: Vec::new(),
            arguments_complete: true,
        }
    }
}

impl<'a> Walker<'a> {
    pub(super) fn document(
        analysis: &'a DocumentAnalysis,
        index: &'a Index,
        context: &'a str,
        cancellation: &'a CancellationToken,
        emit: &'a mut dyn FnMut(CallRelation) -> Result<(), SourceQueryError>,
        observe_owner: &'a mut dyn FnMut(&Arc<SourceCallable>),
    ) -> Result<BTreeSet<&'static str>, SourceQueryError> {
        let Some(metadata) = analysis.semantic().map(|s| s.metadata()) else {
            return Ok(BTreeSet::from(["analysis_unavailable"]));
        };
        let (span, declarations, body) = match analysis.snapshot().compilation_unit() {
            CompilationUnit::Program(p) => (p.span, p.declarations.as_slice(), p.body.as_slice()),
            CompilationUnit::Unit(u) => (u.span, u.declarations.as_slice(), &[][..]),
        };
        let Some(owner) = index.declaration(analysis.snapshot().path(), span.offset) else {
            return Ok(BTreeSet::from(["containing_callable_unavailable"]));
        };
        observe_owner(&owner);
        let mut walker = Self {
            analysis,
            metadata,
            index,
            context,
            cancellation,
            emit,
            owner: observe_owner,
            reasons: BTreeSet::new(),
        };
        walker.declarations(declarations, &owner)?;
        walker.statements(body, &owner)?;
        Ok(walker.reasons)
    }

    fn declarations(
        &mut self,
        declarations: &[Decl],
        owner: &Arc<SourceCallable>,
    ) -> Result<(), SourceQueryError> {
        for declaration in declarations {
            self.cancellation.check()?;
            let span = declaration_span(declaration);
            let retained = self
                .metadata
                .static_bound_declarations
                .iter()
                .find(|candidate| declaration_span(candidate) == span);
            let actual = retained.map_or(declaration, |retained| retained.as_ref());
            let static_owner = retained.map(|_| {
                self.index.anonymous(
                    self.analysis,
                    owner,
                    CallableKind::StaticBound,
                    span,
                    &format!("<static@{}>", span.offset),
                )
            });
            let value_owner = static_owner.as_ref().unwrap_or(owner);
            if let Some(owner) = &static_owner {
                (self.owner)(owner);
            }
            match actual {
                Decl::Const(c) => {
                    self.expr(&c.value, value_owner, InvocationKind::Ordinary, None)?
                }
                Decl::Var(v) => self.expr(&v.value, owner, InvocationKind::Ordinary, None)?,
                Decl::Function(f) => self.function(f)?,
                Decl::Procedure(p) => self.procedure(p)?,
                Decl::TypeDef(definition) => match &definition.body {
                    TypeBody::Subrange { lower, upper } => {
                        self.expr(lower, value_owner, InvocationKind::Ordinary, None)?;
                        self.expr(upper, value_owner, InvocationKind::Ordinary, None)?;
                    }
                    TypeBody::Record(record) => {
                        for field in &record.fields {
                            if let Some(original) = &field.default_value {
                                let retained = self
                                    .metadata
                                    .record_defaults
                                    .get(&definition.name)
                                    .and_then(|fields| {
                                        fields.iter().find(|(name, _)| {
                                            name.eq_ignore_ascii_case(&field.name)
                                        })
                                    })
                                    .and_then(|(_, value)| value.as_ref())
                                    .filter(|value| value.span() == original.span());
                                if let Some(default) = retained {
                                    let mut default_owner = self.index.anonymous(
                                        self.analysis,
                                        owner,
                                        CallableKind::RecordDefault,
                                        field.span,
                                        &format!("{}.{}.<default>", definition.name, field.name),
                                    );
                                    Arc::make_mut(&mut default_owner).generic |=
                                        !definition.type_params.is_empty();
                                    (self.owner)(&default_owner);
                                    self.expr(
                                        default,
                                        &default_owner,
                                        InvocationKind::Ordinary,
                                        None,
                                    )?;
                                } else {
                                    self.reasons.insert("checked_default_unavailable");
                                }
                            }
                        }
                        for method in &record.methods {
                            match method {
                                RecordMethod::Function(f) | RecordMethod::StaticFunction(f) => {
                                    self.function(f)?
                                }
                                RecordMethod::Procedure(p) | RecordMethod::StaticProcedure(p) => {
                                    self.procedure(p)?
                                }
                            }
                        }
                    }
                    TypeBody::Enum(_) | TypeBody::Alias(_) | TypeBody::Distinct(_) => {}
                },
            }
        }
        Ok(())
    }

    fn function(&mut self, function: &FunctionDecl) -> Result<(), SourceQueryError> {
        let Some(owner) = self
            .index
            .declaration(self.analysis.snapshot().path(), function.span.offset)
        else {
            self.reasons.insert("containing_callable_unavailable");
            return Ok(());
        };
        (self.owner)(&owner);
        for clause in &function.requires {
            self.expr(&clause.condition, &owner, InvocationKind::Ordinary, None)?;
        }
        for clause in &function.ensures {
            self.expr(&clause.condition, &owner, InvocationKind::Ordinary, None)?;
        }
        self.body(&function.body, &owner)
    }
    fn procedure(&mut self, procedure: &ProcedureDecl) -> Result<(), SourceQueryError> {
        let Some(owner) = self
            .index
            .declaration(self.analysis.snapshot().path(), procedure.span.offset)
        else {
            self.reasons.insert("containing_callable_unavailable");
            return Ok(());
        };
        (self.owner)(&owner);
        for clause in &procedure.requires {
            self.expr(&clause.condition, &owner, InvocationKind::Ordinary, None)?;
        }
        for clause in &procedure.ensures {
            self.expr(&clause.condition, &owner, InvocationKind::Ordinary, None)?;
        }
        self.body(&procedure.body, &owner)
    }
    fn body(
        &mut self,
        body: &FuncBody,
        owner: &Arc<SourceCallable>,
    ) -> Result<(), SourceQueryError> {
        let FuncBody::Block { nested, stmts } = body;
        self.declarations(nested, owner)?;
        self.statements(stmts, owner)
    }

    fn emit(
        &mut self,
        owner: &Arc<SourceCallable>,
        span: Span,
        kind: CallRelationKind,
        resolved: &resolution::Resolved,
        syntax: CallSyntax,
        callback: Option<&Callback>,
    ) -> Result<(), SourceQueryError> {
        let CallSyntax {
            invocation,
            arguments,
            arguments_complete,
        } = syntax;
        (self.emit)(CallRelation {
            kind,
            containing: Arc::clone(owner),
            target: resolved.target.clone(),
            snapshot: Arc::clone(self.analysis.snapshot()),
            span: span.diagnostic_span_or_synthetic(),
            context: self.context.into(),
            invocation,
            arguments,
            arguments_complete,
            implicit_receiver: resolved.receiver,
            receiving: callback.and_then(|c| c.receiving.clone()),
            receiving_parameter: callback.and_then(|c| c.parameter),
            unresolved_reason: resolved
                .reason
                .or_else(|| (!arguments_complete).then_some("argument_mapping_unavailable")),
        })
    }
}

fn declaration_span(declaration: &Decl) -> Span {
    match declaration {
        Decl::Const(c) => c.span,
        Decl::Var(v) => v.span,
        Decl::TypeDef(t) => t.span,
        Decl::Function(f) => f.span,
        Decl::Procedure(p) => p.span,
    }
}
