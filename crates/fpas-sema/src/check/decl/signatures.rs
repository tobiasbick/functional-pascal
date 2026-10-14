//! Scope-wide named routine signatures and ordered value checking.
//! **Documentation:** `docs/pascal/language/types/declaration-order.md`.

use super::Checker;
use crate::scope::{Symbol, SymbolKind};
use fpas_parser::Decl;

impl Checker {
    /// Recognize a collected declaration only in its currently active lexical scope.
    pub(super) fn has_routine_signature(&self, name: &str, span: fpas_lexer::Span) -> bool {
        self.scopes
            .lookup_with_scope_and_declaration(name)
            .is_some_and(|(scope, symbol, declaration)| {
                scope + 1 == self.scopes.scope_count()
                    && declaration == Some(span)
                    && matches!(symbol.kind, SymbolKind::Function | SymbolKind::Procedure)
            })
    }
    /// Collect every sibling signature in the current lexical scope.
    pub(crate) fn collect_routine_signatures(&mut self, declarations: &[Decl]) {
        for declaration in declarations {
            match declaration {
                Decl::Function(function)
                    if !self.has_routine_signature(&function.name, function.span) =>
                {
                    self.collect_function_signature(function)
                }
                Decl::Procedure(procedure)
                    if !self.has_routine_signature(&procedure.name, procedure.span) =>
                {
                    self.collect_procedure_signature(procedure)
                }
                _ => {}
            }
        }
    }

    /// Check ordered values before deferred bodies resolve their complete lexical environment.
    pub(crate) fn check_scope_declarations(&mut self, declarations: &[Decl]) {
        self.collect_routine_signatures(declarations);
        for declaration in declarations {
            let (name, annotation, span, mutable) = match declaration {
                Decl::Const(value) => (&value.name, &value.type_expr, value.span, false),
                Decl::Var(value) => (&value.name, &value.type_expr, value.span, true),
                _ => continue,
            };
            let ty = self.resolve_type_expr(annotation);
            if self.scopes.define_with_declaration(
                name,
                Symbol {
                    constant: None,
                    ty,
                    mutable,
                    kind: if mutable {
                        SymbolKind::Var
                    } else {
                        SymbolKind::Const
                    },
                    task_bound: false,
                },
                span,
            ) {
                self.predeclared_bindings
                    .insert((span.source_id, span.offset));
            }
        }
        loop {
            let mut changed = false;
            for declaration in declarations {
                let Decl::Const(value) = declaration else {
                    continue;
                };
                if !self
                    .predeclared_bindings
                    .contains(&(value.span.source_id, value.span.offset))
                {
                    continue;
                }
                let Some(ty) = self
                    .scopes
                    .lookup(&value.name)
                    .map(|symbol| symbol.ty.clone())
                else {
                    continue;
                };
                let info = self.binding_constant_info(&value.value, &ty);
                if let Some(symbol) = self.scopes.lookup_mut(&value.name)
                    && symbol.constant.as_ref() != Some(&info)
                {
                    symbol.constant = Some(info);
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        for declaration in declarations {
            if matches!(declaration, Decl::Const(_) | Decl::Var(_)) {
                self.check_decl(declaration);
            }
        }
        for declaration in declarations {
            if !matches!(declaration, Decl::Const(_) | Decl::Var(_)) {
                self.check_decl(declaration);
            }
        }
    }
}
