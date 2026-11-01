//! Declaration links use compiler identity and exact source spans, never display-name guesses.

use super::model::{CallableIdentity, CallableKind, SourceCallable};
use crate::{
    DeclarationOrigin, DocumentAnalysis, DocumentSymbol, SourceQueryError, SymbolKind,
    identity_path,
};
use fpas_lexer::Span;
use fpas_parser::{CompilationUnit, Decl, FuncBody, RecordMethod, TypeBody};
use fpas_sema::Ty;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::Arc,
};

pub(super) struct Index {
    declarations: BTreeMap<(PathBuf, usize), Arc<SourceCallable>>,
    qualified: BTreeMap<String, Vec<Arc<SourceCallable>>>,
    local_methods: BTreeMap<(PathBuf, String), Arc<SourceCallable>>,
    pub(super) signatures: BTreeMap<CallableIdentity, Ty>,
}

impl Index {
    pub(super) fn new(
        documents: &[Arc<DocumentAnalysis>],
        owned: &BTreeSet<PathBuf>,
        standard: &BTreeSet<PathBuf>,
    ) -> Result<Self, SourceQueryError> {
        let mut index = Self {
            declarations: BTreeMap::new(),
            qualified: BTreeMap::new(),
            local_methods: BTreeMap::new(),
            signatures: BTreeMap::new(),
        };
        for analysis in documents {
            let path = analysis.snapshot().path();
            let origin = if owned.contains(path) {
                DeclarationOrigin::Owned
            } else if standard.contains(path) {
                DeclarationOrigin::StandardLibrary
            } else {
                DeclarationOrigin::Dependency
            };
            let file = identity_path(path)
                .map_err(|e| SourceQueryError::new("unsupported_path_identity", e.to_string()))?;
            let mut generic = BTreeSet::new();
            let declarations = match analysis.snapshot().compilation_unit() {
                CompilationUnit::Unit(unit) => &unit.declarations,
                CompilationUnit::Program(program) => &program.declarations,
            };
            generic_declarations(declarations, false, &mut generic);
            index.add(
                analysis,
                analysis.symbols().entries(),
                &file,
                origin,
                &generic,
            );
        }
        Ok(index)
    }

    fn add(
        &mut self,
        analysis: &DocumentAnalysis,
        symbols: &[DocumentSymbol],
        file: &str,
        origin: DeclarationOrigin,
        generic: &BTreeSet<usize>,
    ) {
        for symbol in symbols {
            let kind = match symbol.kind {
                SymbolKind::Function | SymbolKind::Procedure | SymbolKind::Method => {
                    Some(CallableKind::Routine)
                }
                SymbolKind::Program => Some(CallableKind::ProgramEntry),
                SymbolKind::Unit => Some(CallableKind::UnitInitialization),
                _ => None,
            };
            if let Some(kind) = kind {
                let callable = Arc::new(SourceCallable {
                    identity: CallableIdentity {
                        file: Some(file.into()),
                        start: Some(symbol.full_span.offset()),
                        kind,
                        qualified_name: symbol.qualified_name.clone(),
                    },
                    snapshot: Some(Arc::clone(analysis.snapshot())),
                    span: Some(symbol.full_span),
                    selection_span: Some(symbol.selection_span),
                    origin,
                    generic: generic.contains(&symbol.full_span.offset()),
                });
                if let Some(metadata) = analysis.semantic().map(|s| s.metadata())
                    && let Some((_, ty)) = metadata
                        .declaration_types
                        .iter()
                        .find(|((_, offset), _)| *offset == symbol.full_span.offset())
                {
                    self.signatures
                        .insert(callable.identity.clone(), ty.clone());
                }
                self.declarations.insert(
                    (analysis.snapshot().path().into(), symbol.full_span.offset()),
                    Arc::clone(&callable),
                );
                self.qualified
                    .entry(symbol.qualified_name.to_ascii_lowercase())
                    .or_default()
                    .push(Arc::clone(&callable));
                if symbol.kind == SymbolKind::Method
                    && let Some(local) = symbol
                        .qualified_name
                        .get(analysis.symbols().owner().len() + 1..)
                {
                    self.local_methods.insert(
                        (
                            analysis.snapshot().path().into(),
                            local.to_ascii_lowercase(),
                        ),
                        callable,
                    );
                }
            }
            self.add(analysis, &symbol.children, file, origin, generic);
        }
    }

    pub(super) fn declaration(&self, path: &Path, start: usize) -> Option<Arc<SourceCallable>> {
        self.declarations.get(&(path.into(), start)).cloned()
    }

    pub(super) fn qualified(&self, name: &str) -> Option<Arc<SourceCallable>> {
        let declarations = self.qualified.get(&name.to_ascii_lowercase())?;
        let [declaration] = declarations.as_slice() else {
            return None;
        };
        Some(Arc::clone(declaration))
    }

    pub(super) fn method(&self, path: &Path, canonical: &str) -> Option<Arc<SourceCallable>> {
        self.local_methods
            .get(&(path.into(), canonical.to_ascii_lowercase()))
            .cloned()
            .or_else(|| self.qualified(canonical))
    }

    pub(super) fn anonymous(
        &self,
        analysis: &DocumentAnalysis,
        parent: &SourceCallable,
        kind: CallableKind,
        span: Span,
        name: &str,
    ) -> Arc<SourceCallable> {
        Arc::new(SourceCallable {
            identity: CallableIdentity {
                file: parent.identity.file.clone(),
                start: Some(span.offset),
                kind,
                qualified_name: format!("{}.{name}", parent.identity.qualified_name),
            },
            snapshot: Some(Arc::clone(analysis.snapshot())),
            span: Some(span.diagnostic_span_or_synthetic()),
            selection_span: Some(
                Span {
                    length: span.length.min(1),
                    ..span
                }
                .diagnostic_span_or_synthetic(),
            ),
            origin: parent.origin,
            generic: parent.generic,
        })
    }
}

pub(super) fn native(name: &str) -> Option<(Arc<SourceCallable>, Ty)> {
    let (name, ty) = if let Some(operation) = fpas_sema::native_operation_by_implementation(name) {
        (
            crate::api_lookup::native_qualified_name(operation),
            Ty::Function((operation.signature)()),
        )
    } else {
        let symbol = super::super::intrinsic::lookup(name)?;
        if !matches!(symbol.ty, Ty::Function(_) | Ty::Procedure(_)) {
            return None;
        }
        (symbol.qualified_name, symbol.ty)
    };
    Some((
        Arc::new(SourceCallable {
            identity: CallableIdentity {
                file: None,
                start: None,
                kind: CallableKind::Builtin,
                qualified_name: name,
            },
            snapshot: None,
            span: None,
            selection_span: None,
            origin: DeclarationOrigin::Builtin,
            generic: false,
        }),
        ty,
    ))
}

fn generic_declarations(declarations: &[Decl], inherited: bool, output: &mut BTreeSet<usize>) {
    for declaration in declarations {
        match declaration {
            Decl::Function(f) => generic_body(
                f.span,
                &f.body,
                inherited || !f.type_params.is_empty(),
                output,
            ),
            Decl::Procedure(p) => generic_body(
                p.span,
                &p.body,
                inherited || !p.type_params.is_empty(),
                output,
            ),
            Decl::TypeDef(t) => {
                if let TypeBody::Record(record) = &t.body {
                    for method in &record.methods {
                        let parent = inherited || !t.type_params.is_empty();
                        match method {
                            RecordMethod::Function(f) | RecordMethod::StaticFunction(f) => {
                                generic_body(
                                    f.span,
                                    &f.body,
                                    parent || !f.type_params.is_empty(),
                                    output,
                                )
                            }
                            RecordMethod::Procedure(p) | RecordMethod::StaticProcedure(p) => {
                                generic_body(
                                    p.span,
                                    &p.body,
                                    parent || !p.type_params.is_empty(),
                                    output,
                                )
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
fn generic_body(span: Span, body: &FuncBody, generic: bool, output: &mut BTreeSet<usize>) {
    if generic {
        output.insert(span.offset);
    }
    let FuncBody::Block { nested, .. } = body;
    generic_declarations(nested, generic, output);
}
