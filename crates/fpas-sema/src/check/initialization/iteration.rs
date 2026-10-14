//! Read-only for-in storage using the existing transitive write analysis.
//! **Documentation:** `docs/pascal/language/control-flow/for-in.md`.

use super::collection::Owner;
use super::walker::{Failure, Walker};
use super::{BindingId, identity};
use crate::scope::SymbolKind;
use fpas_lexer::Span;
use fpas_parser::{Designator, Expr, PostfixOperation};

/// Declaration identity or canonical imported identity of protected storage.
#[derive(Clone, PartialEq, Eq)]
pub(super) enum Root {
    Binding(BindingId),
    Imported(String),
}

/// Writable source protected for the lexical duration of a for-in body.
pub(super) struct Source {
    pub(super) root: Root,
    name: String,
}

impl Walker<'_, '_> {
    /// Select storage without treating a computed collection as an alias of its inputs.
    pub(super) fn iteration_source(&self, expr: &Expr) -> Option<Source> {
        match expr {
            Expr::Paren(inner, _) => self.iteration_source(inner),
            Expr::Postfix {
                base, operations, ..
            } if operations.iter().all(|operation| {
                matches!(
                    operation,
                    PostfixOperation::Field { .. } | PostfixOperation::Index { .. }
                )
            }) =>
            {
                self.iteration_source(base)
            }
            Expr::Designator(designator) => {
                let reference = self
                    .checker
                    .initialization
                    .references
                    .get(&identity(designator.span))?;
                if let Some(binding) = reference.declaration {
                    let info = self.model.bindings.get(&binding)?;
                    info.mutable.then(|| Source {
                        root: Root::Binding(binding),
                        name: info.name.clone(),
                    })
                } else {
                    (reference.kind == SymbolKind::Var).then(|| Source {
                        root: Root::Imported(reference.name.to_ascii_lowercase()),
                        name: reference.name.clone(),
                    })
                }
            }
            _ => None,
        }
    }

    /// Reject a write against every active outer or inner iteration source.
    fn iteration_write(&mut self, root: &Root, span: Span) {
        if self.validate {
            for source in &self.iteration_sources {
                if self.iteration_roots_overlap(&source.root, root) {
                    self.iteration_errors.push(Failure {
                        span,
                        name: source.name.clone(),
                    });
                }
            }
        }
    }

    /// A var parameter may alias a foreign root even when declaration identities differ.
    fn iteration_roots_overlap(&self, left: &Root, right: &Root) -> bool {
        if left == right {
            return true;
        }
        match (left, right) {
            (Root::Binding(left), Root::Binding(right)) => {
                self.reference_may_alias(*left, *right) || self.reference_may_alias(*right, *left)
            }
            (Root::Binding(binding), Root::Imported(_))
            | (Root::Imported(_), Root::Binding(binding)) => self
                .model
                .bindings
                .get(binding)
                .is_some_and(|binding| binding.mutable && binding.parameter.is_some()),
            _ => false,
        }
    }

    fn reference_may_alias(&self, reference: BindingId, other: BindingId) -> bool {
        let Some(reference) = self.model.bindings.get(&reference) else {
            return false;
        };
        let Some(other) = self.model.bindings.get(&other) else {
            return false;
        };
        reference.mutable
            && reference.parameter.is_some()
            && other.mutable
            && (other.parameter.is_some()
                || (other.owner != reference.owner && other.owner != self.owner))
    }

    /// Check a declaration-resolved local/captured write and retain global effects.
    pub(super) fn iteration_binding_write(&mut self, binding: BindingId, span: Span) {
        self.iteration_write(&Root::Binding(binding), span);
        if let Some(info) = self.model.bindings.get(&binding)
            && info.owner == Owner::Root
        {
            self.summary
                .written_globals
                .insert(info.name.to_ascii_lowercase());
        }
    }

    /// Retain imported/transitive global writes and compare their canonical identities.
    pub(super) fn iteration_global_writes(
        &mut self,
        names: impl IntoIterator<Item = String>,
        span: Span,
    ) {
        for name in names {
            let name = name.to_ascii_lowercase();
            self.iteration_write(&Root::Imported(name.clone()), span);
            self.summary.written_globals.insert(name);
        }
    }

    /// Record a real storage write, including imported variables with no local declaration.
    pub(super) fn write_designator(&mut self, designator: &Designator, span: Span) {
        let Some(reference) = self
            .checker
            .initialization
            .references
            .get(&identity(designator.span))
        else {
            return;
        };
        if let Some(binding) = reference.declaration {
            self.write(binding, span);
        } else if reference.kind == SymbolKind::Var {
            self.iteration_global_writes([reference.name.clone()], span);
        }
    }

    /// Reject opaque callable effects while a writable source must remain unchanged.
    pub(super) fn iteration_unknown(&mut self, span: Span) {
        if self.validate {
            for source in &self.iteration_sources {
                self.iteration_errors.push(Failure {
                    span,
                    name: format!("{} (callable write effects are unknown)", source.name),
                });
            }
        }
    }
}
