//! Restricted contract expressions and declaration-scoped field defaults.
//! **Documentation:** `docs/pascal/language/functions/contracts.md`.

mod expressions;
mod postconditions;

use super::Checker;
use crate::scope::SymbolKind;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_INVALID_CONTRACT_EXPRESSION;
use fpas_lexer::Span;
use fpas_parser::{Designator, DesignatorPart, EnsuresClause, Expr, FuncBody, RequiresClause};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Checking state that preserves the original binding identity of default expressions.
#[derive(Default)]
pub(crate) struct ContractState {
    scope: Option<usize>,
    reads: HashMap<usize, bool>,
    routine_values: HashSet<usize>,
    defaults: Vec<(Arc<Expr>, Span)>,
}

/// The declaration body and its heading clauses share one checking context.
#[derive(Clone, Copy)]
pub(crate) struct RoutineBody<'a> {
    /// Executable declarations and statements.
    pub body: &'a FuncBody,
    /// Clauses evaluated before the body's local scope is introduced.
    pub requires: &'a [RequiresClause],
    /// Return clauses checked in the heading scope with a clause-local result binding.
    pub ensures: &'a [EnsuresClause],
}

impl Checker {
    /// Check clauses in the parameter scope, before introducing body-local declarations.
    pub(crate) fn check_preconditions(&mut self, clauses: &[RequiresClause]) {
        let previous = self.contracts.scope.replace(self.scopes.scope_count() - 1);
        for clause in clauses {
            let ty = self.check_expr_with_expected(&clause.condition, Some(&Ty::Boolean));
            self.check_type_compat(&Ty::Boolean, &ty, "precondition", clause.condition.span());
            self.check_contract_expression(&clause.condition, clause.span);
        }
        self.contracts.scope = previous;
    }

    /// Retain read permissions while the actual declaration scope is available.
    pub(crate) fn record_contract_read(
        &mut self,
        designator: &Designator,
        parts: &[DesignatorPart],
    ) {
        let name = parts
            .iter()
            .filter_map(|part| match part {
                DesignatorPart::Ident(name, _) => Some(name.as_str()),
                DesignatorPart::Index(..) => None,
            })
            .collect::<Vec<_>>()
            .join(".");
        let root = self.scopes.lookup_with_scope(&name).or_else(|| {
            let (_, count) = self.designator_root_symbol(parts)?;
            let name = parts[..count]
                .iter()
                .filter_map(|part| match part {
                    DesignatorPart::Ident(name, _) => Some(name.as_str()),
                    DesignatorPart::Index(..) => None,
                })
                .collect::<Vec<_>>()
                .join(".");
            self.scopes.lookup_with_scope(&name)
        });
        if let Some((scope, symbol)) = root {
            let routine_value = matches!(
                symbol.kind,
                SymbolKind::Function | SymbolKind::Procedure | SymbolKind::BuiltinStd
            );
            let allowed = !symbol.mutable
                || (symbol.kind == SymbolKind::Param && self.contracts.scope == Some(scope));
            if routine_value {
                self.contracts
                    .routine_values
                    .insert(crate::designator_lookup_key(designator));
            }
            self.contracts
                .reads
                .entry(crate::designator_lookup_key(designator))
                .and_modify(|read| *read &= allowed)
                .or_insert(allowed);
        }
    }

    /// Validate omitted defaults after their original declaration contexts have been checked.
    pub(crate) fn finish_contract_checks(&mut self) {
        let mut seen = HashSet::new();
        while let Some((expression, clause_span)) = self.contracts.defaults.pop() {
            if seen.insert((
                Self::expr_lookup_key(&expression),
                clause_span.source_id,
                clause_span.offset,
            )) {
                self.check_contract_expression(&expression, clause_span);
            }
        }
    }

    fn reject_contract_expression(&mut self, reason: &str, span: Span) {
        self.error_with_code(
            SEMA_INVALID_CONTRACT_EXPRESSION,
            format!("Invalid contract expression: {reason}"),
            "Use parameters, immutable values, operators (including `in`), construction, and the native Length or IsEmpty operations; do not call user routines or read mutable outer state.",
            span,
        );
    }

    fn contract_designator_allowed(&self, designator: &Designator) -> bool {
        let key = crate::designator_lookup_key(designator);
        self.contracts.reads.get(&key).copied().unwrap_or(true)
    }

    fn contract_native_allowed(&self, key: usize) -> bool {
        self.fluent_calls.get(&key).is_some_and(|target| {
            matches!(
                target.name.as_str(),
                "Std.Errors.Message"
                    | "Std.Str.Length"
                    | "Std.Str.IsEmpty"
                    | "Std.Arrays.Length"
                    | "Std.Arrays.IsEmpty"
                    | "Std.Dictionaries.Length"
                    | "Std.Dictionaries.IsEmpty"
            )
        })
    }

    fn contract_bound_method(&self, key: usize) -> bool {
        self.bound_methods.contains_key(&key)
    }

    fn contract_call_allowed(&self, expr: &Expr) -> bool {
        let key = Self::expr_lookup_key(expr);
        self.record_constructions.contains(&key)
            || self.enum_constructions.contains(&key)
            || self.distinct_conversions.contains(&key)
            || self.subrange_conversions.contains_key(&key)
            || self.contract_native_allowed(key)
    }

    fn queue_contract_defaults(&mut self, expr: &Expr, args: &[Expr], span: Span) {
        let key = Self::expr_lookup_key(expr);
        if !self.record_constructions.contains(&key) {
            return;
        }
        let Some(Ty::Record(record)) = self.expr_types.get(&key) else {
            return;
        };
        let Some(defaults) = self.record_defaults.get(&record.name) else {
            return;
        };
        for (name, expression) in defaults {
            if !args.iter().any(|argument| {
                argument
                    .argument_name()
                    .is_some_and(|written| written.eq_ignore_ascii_case(name))
            }) && let Some(expression) = expression
            {
                self.contracts.defaults.push((Arc::clone(expression), span));
            }
        }
    }
}
