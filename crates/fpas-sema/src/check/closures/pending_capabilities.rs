//! Preserve task and discard decisions while recursive captures are incomplete.
//! See `docs/pascal/language/functions/closures.md` and
//! `docs/pascal/language/functions/discard.md`.

use std::collections::HashMap;

use fpas_diagnostics::codes::{SEMA_TASK_BOUND_CALLABLE, SEMA_UNSAFE_DISCARD};
use fpas_lexer::Span;
use fpas_parser::{Designator, DesignatorPart, Expr};

use super::{CaptureBinding, task_bound_from_captures};
use crate::check::Checker;

type BindingKey = (String, u32, usize, usize);

/// Pending routine provenance through bindings and accepted capability boundaries.
#[derive(Default)]
pub(super) struct PendingCapabilities {
    bindings: HashMap<BindingKey, Vec<usize>>,
    checks: Vec<(Vec<usize>, Span, bool)>,
}

impl Checker {
    /// Retains pending capture dependencies through immutable aggregates and pattern bindings.
    pub(in crate::check) fn record_pending_capture_binding(
        &mut self,
        name: &str,
        value: Option<&Expr>,
    ) {
        let Some((_, _, Some(span))) = self.scopes.lookup_with_scope_and_declaration(name) else {
            return;
        };
        let targets = value
            .map(|expr| self.pending_expr_captures(expr))
            .unwrap_or_default();
        if !targets.is_empty() {
            self.capture_dependencies
                .capabilities
                .bindings
                .insert(binding_key(name, span), targets);
        }
    }

    /// Rechecks an accepted task call once recursive capture capabilities are complete.
    pub(in crate::check) fn defer_capture_task_call(&mut self, call: &Expr, span: Span) {
        let targets = match call {
            Expr::Call { designator, .. } => self.pending_designator_captures(designator),
            Expr::Postfix { base, .. } => self.pending_expr_captures(base),
            _ => Vec::new(),
        };
        self.defer_capture_boundary(targets, span, true);
    }

    /// Rechecks a provisional discard proof once all reachable captures are known.
    pub(in crate::check) fn defer_capture_discard(&mut self, expr: &Expr, span: Span) {
        self.defer_capture_boundary(self.pending_expr_captures(expr), span, false);
    }

    /// Rechecks a callable transferred through a channel or task-group boundary.
    /// See `docs/pascal/language/functions/closures.md#concurrency`.
    pub(crate) fn defer_capture_transfer(&mut self, expr: &Expr, span: Span) {
        self.defer_capture_boundary(self.pending_expr_captures(expr), span, true);
    }

    fn defer_capture_boundary(&mut self, targets: Vec<usize>, span: Span, task: bool) {
        if !targets.is_empty() {
            self.capture_dependencies
                .capabilities
                .checks
                .push((targets, span, task));
        }
    }

    fn pending_expr_captures(&self, expr: &Expr) -> Vec<usize> {
        if self
            .expr_types
            .get(&Self::expr_lookup_key(expr))
            .is_some_and(|ty| !self.type_can_contain_callable(ty))
        {
            return Vec::new();
        }
        let mut targets = match expr {
            Expr::Closure(_) => {
                let key = Self::expr_lookup_key(expr);
                let mut targets = self
                    .capture_dependencies
                    .closures
                    .get(&key)
                    .cloned()
                    .unwrap_or_default();
                if let Some(info) = self.closure_infos.get(&key) {
                    for capture in &info.captures {
                        if let Some(pending) = self
                            .capture_dependencies
                            .capabilities
                            .bindings
                            .get(&binding_key(&capture.name, capture.declaration))
                        {
                            targets.extend(pending);
                        }
                    }
                }
                targets
            }
            Expr::Designator(designator) => self.pending_designator_captures(designator),
            Expr::Paren(inner, _)
            | Expr::ResultOk(inner, _)
            | Expr::ResultError(inner, _)
            | Expr::OptionSome(inner, _)
            | Expr::Try(inner, _)
            | Expr::NamedArgument { value: inner, .. }
            | Expr::Postfix { base: inner, .. } => self.pending_expr_captures(inner),
            Expr::ArrayLiteral(values, _) => values
                .iter()
                .flat_map(|value| self.pending_expr_captures(value))
                .collect(),
            Expr::DictLiteral(values, _) => values
                .iter()
                .flat_map(|(key, value)| {
                    self.pending_expr_captures(key)
                        .into_iter()
                        .chain(self.pending_expr_captures(value))
                })
                .collect(),
            Expr::RecordUpdate { base, fields, .. } => self
                .pending_expr_captures(base)
                .into_iter()
                .chain(
                    fields
                        .iter()
                        .flat_map(|field| self.pending_expr_captures(&field.value)),
                )
                .collect(),
            Expr::Call { args, .. }
                if self
                    .record_constructions
                    .contains(&Self::expr_lookup_key(expr)) =>
            {
                args.iter()
                    .flat_map(|arg| self.pending_expr_captures(arg))
                    .collect()
            }
            Expr::Call { designator, .. }
                if self.fluent_calls.contains_key(&Self::expr_lookup_key(expr)) =>
            {
                self.pending_designator_captures(designator)
            }
            _ => Vec::new(),
        };
        targets.sort_unstable();
        targets.dedup();
        targets
    }

    fn pending_designator_captures(&self, designator: &Designator) -> Vec<usize> {
        let full = Self::resolve_designator_name(designator);
        let name = if self.scopes.lookup(&full).is_some() {
            full.as_str()
        } else if let Some(DesignatorPart::Ident(root, _)) = designator.parts.first() {
            root.as_str()
        } else {
            return Vec::new();
        };
        if let Some(key) = self.scopes.routine_capture_key(name) {
            return vec![key];
        }
        self.scopes
            .lookup_with_scope_and_declaration(name)
            .and_then(|(_, _, span)| span)
            .and_then(|span| {
                self.capture_dependencies
                    .capabilities
                    .bindings
                    .get(&binding_key(name, span))
            })
            .cloned()
            .unwrap_or_default()
    }

    /// Completes captured callable proofs and diagnoses provisional task/discard uses.
    pub(in crate::check) fn finish_pending_capture_checks(&mut self) {
        loop {
            let states: HashMap<_, _> = self
                .capture_dependencies
                .capabilities
                .bindings
                .iter()
                .map(|(binding, targets)| {
                    let task_bound = targets.iter().any(|key| {
                        self.nested_routine_captures
                            .get(key)
                            .is_some_and(|info| info.task_bound)
                    });
                    let task_free = targets.iter().all(|key| {
                        self.nested_routine_captures.get(key).is_some_and(|info| {
                            info.captures.iter().all(|capture| capture.task_free)
                        })
                    });
                    (binding.clone(), (task_bound, task_free))
                })
                .collect();
            let mut changed = false;
            for info in self.closure_infos.values_mut() {
                changed |= complete_capabilities(&mut info.captures, &states);
                info.task_bound = task_bound_from_captures(&info.captures);
            }
            for info in self.nested_routine_captures.values_mut() {
                changed |= complete_capabilities(&mut info.captures, &states);
                info.task_bound = task_bound_from_captures(&info.captures);
            }
            if !changed {
                break;
            }
        }
        for (&key, info) in &self.nested_routine_captures {
            self.scopes.update_routine_capture_capabilities(
                key,
                info.task_bound,
                info.captures.iter().all(|capture| capture.task_free),
            );
        }
        for (&key, info) in &self.closure_infos {
            if info.task_bound {
                self.task_bound_exprs.insert(key);
            }
            if let Some(discard) = self.discard_exprs.get_mut(&key) {
                discard.value = info.captures.iter().all(|capture| capture.task_free);
            }
        }
        for (targets, span, task) in
            std::mem::take(&mut self.capture_dependencies.capabilities.checks)
        {
            let invalid = targets.iter().any(|key| {
                self.nested_routine_captures.get(key).is_some_and(|info| {
                    if task {
                        info.task_bound
                    } else {
                        info.captures.iter().any(|capture| !capture.task_free)
                    }
                })
            });
            if invalid {
                let (code, message, hint) = if task {
                    (
                        SEMA_TASK_BOUND_CALLABLE,
                        "Cannot transfer a task-bound callable across a task boundary",
                        "Mutable captures make a closure task-bound. Pass immutable values instead, or invoke the closure on the same task.",
                    )
                } else {
                    (
                        SEMA_UNSAFE_DISCARD,
                        "Cannot discard callable captures containing task handles",
                        "Retain and consume task handles. Capture only statically task-free values in a discarded callable.",
                    )
                };
                self.error_with_code(code, message, hint, span);
            }
        }
    }
}

fn complete_capabilities(
    captures: &mut [CaptureBinding],
    states: &HashMap<BindingKey, (bool, bool)>,
) -> bool {
    let mut changed = false;
    for capture in captures {
        if let Some(&(task_bound, task_free)) =
            states.get(&binding_key(&capture.name, capture.declaration))
        {
            let next_bound = capture.task_bound || task_bound;
            let next_free = capture.task_free && task_free;
            changed |= capture.task_bound != next_bound || capture.task_free != next_free;
            capture.task_bound = next_bound;
            capture.task_free = next_free;
        }
    }
    changed
}

fn binding_key(name: &str, span: Span) -> BindingKey {
    (
        name.to_ascii_lowercase(),
        span.source_id,
        span.offset,
        span.length,
    )
}
