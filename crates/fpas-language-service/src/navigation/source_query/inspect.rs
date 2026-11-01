//! Bounded source inspection follows compiler-bound type annotations exactly one hop.

mod builder;
mod declarations;
mod model;
mod projection;
pub use model::*;
pub use projection::{
    InspectionExcerptProjection, InspectionProjection, InspectionSectionProjection,
};

use super::{DeclarationOrigin, SourceQueryTarget};
use crate::{CancellationToken, DocumentSymbol, LanguageService, LanguageServiceError, SymbolKind};
use fpas_sema::Ty;
use std::collections::BTreeSet;

impl LanguageService {
    /// Describes a resolved declaration without executing its source or expanding callees.
    pub(crate) fn inspect_target(
        &mut self,
        target: &SourceQueryTarget,
        cancellation: &CancellationToken,
    ) -> Result<Inspection, LanguageServiceError> {
        cancellation.check()?;
        let mut result = Inspection::new();
        let Some((snapshot, symbol)) = target.snapshot.as_ref().zip(target.symbol.as_ref()) else {
            let registered = super::intrinsic::lookup(&target.qualified_name);
            let native = fpas_sema::native_operations()
                .find(|n| crate::api_lookup::native_qualified_name(n) == target.qualified_name);
            result.sections.push(InspectionSection {
                kind: InspectionSectionKind::Signature,
                reason: "selected_declaration",
                owner: None,
                qualified_name: target.qualified_name.clone(),
                origin: DeclarationOrigin::Builtin,
                checked: true,
                snapshot: None,
                span: None,
                source_sha256: None,
                detail: super::intrinsic::detail(&target.qualified_name)
                    .or_else(|| registered.map(|s| s.ty.to_string()))
                    .or_else(|| {
                        native.map(|n| crate::intellisense::native::signature(n, None).label)
                    })
                    .or_else(|| Some(target.name.clone())),
                documentation: native.map(|n| n.documentation.into()),
                excerpts: Vec::new(),
                full_bytes: 0,
                omitted_bytes: 0,
            });
            result.omit("implementation", "no_source", None);
            return Ok(result);
        };
        let generated = self.is_editor_api_source(snapshot.path());
        let origin = builder::origin(self, snapshot.path());
        let owner = builder::owner(self, snapshot.path());
        let declarations = declarations::declarations(snapshot.compilation_unit());
        let routine = declarations::routine(declarations, symbol.full_span.offset());
        let body_start = routine.and_then(|r| r.body_start(snapshot.source()));
        let signature_end = if matches!(symbol.kind, SymbolKind::Unit | SymbolKind::Program) {
            snapshot.source()[symbol.selection_span.end()..symbol.full_span.end()]
                .find(';')
                .map(|offset| symbol.selection_span.end() + offset + 1)
                .unwrap_or(symbol.selection_span.end())
        } else {
            body_start.unwrap_or(symbol.full_span.end())
        };
        let signature = builder::exact_span(
            snapshot,
            symbol.full_span.offset()..signature_end,
            symbol.full_span.source_id(),
        )
        .unwrap_or(symbol.full_span);
        result.source(
            snapshot,
            symbol,
            signature,
            builder::SourceSection {
                kind: InspectionSectionKind::Signature,
                reason: "selected_declaration",
                origin,
                owner: owner.clone(),
            },
            excluded_methods(snapshot, symbol),
        );
        if generated {
            result.omit("implementation", "intrinsic_implementation", None);
        } else if let Some(start) = body_start {
            if let Some(span) = builder::exact_span(
                snapshot,
                start..symbol.full_span.end(),
                symbol.full_span.source_id(),
            ) {
                result.source(
                    snapshot,
                    symbol,
                    span,
                    builder::SourceSection {
                        kind: InspectionSectionKind::Implementation,
                        reason: "selected_body",
                        origin,
                        owner,
                    },
                    Vec::new(),
                );
            }
        } else {
            result.omit("implementation", "not_callable", None);
        }
        let context = self.navigation_context(snapshot.path())?;
        let mut retained =
            BTreeSet::from([(snapshot.path().to_path_buf(), symbol.full_span.offset())]);
        if generated {
            if let Some(registered) = super::intrinsic::lookup(&target.qualified_name) {
                direct_nominals(&registered.ty, &mut |name| {
                    add_support(
                        self,
                        &context.documents,
                        snapshot.path(),
                        SupportingType {
                            declaration: None,
                            qualified: Some(name),
                            reason: "signature",
                        },
                        &mut retained,
                        &mut result,
                    )
                });
            }
        } else if let Some(routine) = routine {
            let analysis = self.analyze_document(snapshot.path())?;
            if let Some(semantic) = analysis.semantic() {
                let mut cancelled = false;
                routine.annotations(&mut |ty, reason| {
                    declarations::named_types(ty, &mut |id| {
                        if cancellation.is_cancelled() {
                            cancelled = true;
                            return false;
                        }
                        let Some(binding) = semantic
                            .metadata()
                            .type_references
                            .get(&(id.span.source_id, id.span.offset))
                        else {
                            result.analysis_complete = false;
                            result.omit("supporting_types", "binding_unavailable", None);
                            return true;
                        };
                        if matches!(binding.ty, Ty::GenericParam(..)) {
                            return true;
                        }
                        add_support(
                            self,
                            &context.documents,
                            snapshot.path(),
                            SupportingType {
                                declaration: binding.declaration,
                                qualified: binding.qualified_name.as_deref(),
                                reason,
                            },
                            &mut retained,
                            &mut result,
                        )
                    })
                });
                if cancelled {
                    cancellation.check()?;
                }
            } else {
                result.analysis_complete = false;
                result.omit("supporting_types", "analysis_unavailable", None);
            }
        }
        result.omit("supporting_types", "one_hop_scope", None);
        cancellation.check()?;
        Ok(result)
    }
}

struct SupportingType<'a> {
    declaration: Option<fpas_lexer::Span>,
    qualified: Option<&'a str>,
    reason: &'static str,
}

fn add_support(
    service: &LanguageService,
    documents: &[crate::navigation::NavigationDocument],
    selected_path: &std::path::Path,
    binding: SupportingType<'_>,
    retained: &mut BTreeSet<(std::path::PathBuf, usize)>,
    result: &mut Inspection,
) -> bool {
    let SupportingType {
        declaration,
        qualified,
        reason,
    } = binding;
    if declaration.is_none() && qualified.is_none() {
        return true;
    }
    let mut found: Option<(&crate::navigation::NavigationDocument, DocumentSymbol)> = None;
    let mut ambiguous = false;
    for document in documents {
        let mut visit = |symbol: &DocumentSymbol| {
            if !matches!(symbol.kind, SymbolKind::Type | SymbolKind::Enum) {
                return;
            }
            let matches = declaration.is_some_and(|span| {
                document.path == selected_path && symbol.full_span.offset() == span.offset
            }) || qualified
                .is_some_and(|name| symbol.qualified_name.eq_ignore_ascii_case(name));
            if matches {
                if let Some((previous, item)) = &found
                    && (previous.path != document.path
                        || item.selection_span != symbol.selection_span)
                {
                    ambiguous = true;
                }
                found = Some((document, symbol.clone()));
            }
        };
        walk(&document.roots, &mut visit);
    }
    if ambiguous {
        result.analysis_complete = false;
        result.omit("supporting_types", "ambiguous_declaration", None);
        return true;
    }
    let Some((document, symbol)) = found else {
        if qualified.and_then(super::intrinsic::lookup).is_some() {
            result.omit("supporting_types", "no_source", None);
            return true;
        }
        result.analysis_complete = false;
        result.omit("supporting_types", "declaration_unavailable", None);
        return true;
    };
    let identity = (document.path.clone(), symbol.full_span.offset());
    if retained.contains(&identity) {
        return true;
    }
    if result.supporting_types == INSPECT_SUPPORTING_TYPES {
        result.omit("supporting_types", "supporting_type_limit", None);
        return false;
    }
    retained.insert(identity);
    result.supporting_types += 1;
    let origin = builder::origin(service, &document.path);
    result.source(
        &document.snapshot,
        &symbol,
        symbol.full_span,
        builder::SourceSection {
            kind: InspectionSectionKind::SupportingType,
            reason,
            origin,
            owner: builder::owner(service, &document.path),
        },
        excluded_methods(&document.snapshot, &symbol),
    );
    true
}

fn excluded_methods(
    snapshot: &crate::DocumentSnapshot,
    symbol: &DocumentSymbol,
) -> Vec<std::ops::Range<usize>> {
    if !matches!(symbol.kind, SymbolKind::Type | SymbolKind::Enum) {
        return Vec::new();
    }
    let declarations = declarations::declarations(snapshot.compilation_unit());
    symbol
        .children
        .iter()
        .filter(|s| s.kind == SymbolKind::Method)
        .filter_map(|method| {
            let routine = declarations::routine(declarations, method.full_span.offset())?;
            Some(routine.body_start(snapshot.source())?..method.full_span.end())
        })
        .collect()
}

fn walk(symbols: &[DocumentSymbol], visit: &mut impl FnMut(&DocumentSymbol)) {
    for symbol in symbols {
        visit(symbol);
        walk(&symbol.children, visit);
    }
}

fn direct_nominals(ty: &Ty, visit: &mut impl FnMut(&str) -> bool) -> bool {
    match ty {
        Ty::Array(inner) | Ty::Channel(inner) | Ty::Task(inner) | Ty::Option(inner) => {
            direct_nominals(inner, visit)
        }
        Ty::Dict(left, right) | Ty::Result(left, right) => {
            direct_nominals(left, visit) && direct_nominals(right, visit)
        }
        Ty::Function(f) => {
            f.params.iter().all(|p| direct_nominals(&p.ty, visit))
                && direct_nominals(&f.return_type, visit)
        }
        Ty::Procedure(p) => p.params.iter().all(|p| direct_nominals(&p.ty, visit)),
        Ty::Record(r) => visit(&r.name) && r.type_args.iter().all(|t| direct_nominals(t, visit)),
        Ty::Enum(e) => visit(&e.name) && e.type_args.iter().all(|t| direct_nominals(t, visit)),
        Ty::Distinct(d) => visit(&d.name),
        Ty::Subrange(s) => visit(&s.name),
        _ => true,
    }
}
