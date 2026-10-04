//! Explicit caller-storage arguments and forbidden reference escape.
//!
//! **Documentation:** `docs/pascal/language/functions/var-parameters.md`

use super::super::Checker;
use crate::{check::closures::CaptureBinding, scope::SymbolKind, types::Ty};
use fpas_diagnostics::codes::{SEMA_IMMUTABLE_ASSIGNMENT, SEMA_TYPE_MISMATCH};
use fpas_lexer::Span;
use fpas_parser::{DesignatorPart, Expr};

impl Checker {
    /// Reject reference markers wherever a value snapshot is required.
    pub(crate) fn reject_var_argument_value(&mut self, span: Span) -> Ty {
        self.error_with_code(
            SEMA_TYPE_MISMATCH,
            "A `var` argument is caller storage, not a value",
            "Use `Routine(var Target)` only for a declared `var` parameter; read `Target` normally to obtain a value snapshot.",
            span,
        );
        Ty::Error
    }

    /// Resolve writable selected storage without applying a value conversion.
    pub(crate) fn check_var_argument(&mut self, arg: &Expr) -> Ty {
        let Expr::VarArgument(target, span) = arg else {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                "A `var` parameter requires an explicit `var` argument",
                "Pass writable caller storage as `var Target`, or use a value parameter and an explicit local copy.",
                arg.span(),
            );
            return self.check_expr(arg);
        };
        let checkpoint = self.errors.len();
        let ty = self.check_designator_expr(target);
        if ty.is_error() || self.errors.len() != checkpoint {
            return Ty::Error;
        }
        if self.reject_string_index_assignment(target) {
            return Ty::Error;
        }
        let writable = self
            .designator_root_symbol(&target.parts)
            .is_some_and(|(symbol, _)| {
                symbol.mutable && matches!(symbol.kind, SymbolKind::Var | SymbolKind::VarParam)
            });
        if !writable {
            self.error_with_code(
                SEMA_IMMUTABLE_ASSIGNMENT,
                "A `var` argument requires a writable storage root",
                "Use a mutable binding or a forwarded `var` parameter; const bindings and value parameters cannot supply caller storage.",
                *span,
            );
            return Ty::Error;
        }
        self.expr_types
            .insert(Self::expr_lookup_key(arg), ty.clone());
        ty
    }

    /// Canonical source root for detecting duplicate formal storage arguments.
    pub(in crate::check) fn var_argument_root(&self, arg: &Expr) -> Option<String> {
        let Expr::VarArgument(target, _) = arg else {
            return None;
        };
        let (_, consumed) = self.designator_root_symbol(&target.parts)?;
        let name = target.parts[..consumed]
            .iter()
            .filter_map(|part| match part {
                DesignatorPart::Ident(name, _) => Some(name.as_str()),
                DesignatorPart::Index(_, _) => None,
            })
            .collect::<Vec<_>>()
            .join(".");
        Some(self.qualified_import_name(&name).to_ascii_lowercase())
    }

    /// Prevent synchronous caller authority from entering a closure environment.
    pub(crate) fn reject_var_parameter_captures(&mut self, captures: &[CaptureBinding]) {
        for capture in captures {
            if self
                .scopes
                .lookup_with_scope_and_declaration(&capture.name)
                .is_some_and(|(_, symbol, declaration)| {
                    symbol.kind == SymbolKind::VarParam && declaration == Some(capture.declaration)
                })
            {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!("A closure or nested routine cannot capture var parameter `{}`", capture.name),
                    "Read the var parameter into a local value snapshot and capture that value instead.",
                    capture.declaration,
                );
            }
        }
    }
}
