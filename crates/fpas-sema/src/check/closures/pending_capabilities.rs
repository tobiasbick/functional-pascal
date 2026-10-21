//! Preserve task and discard decisions while recursive captures are incomplete.
//! See `docs/pascal/language/functions/closures.md` and
//! `docs/pascal/language/functions/discard.md`.

use std::collections::HashMap;

use fpas_diagnostics::codes::SEMA_TASK_BOUND_CALLABLE;
use fpas_lexer::Span;
use fpas_parser::{Designator, DesignatorPart, Expr};

use super::{CaptureBinding, capability_from_captures};
use crate::TransferCapability;
use crate::check::Checker;

pub(super) type BindingKey = (String, u32, usize, usize);

/// Pending routine provenance through bindings and accepted capability boundaries.
#[derive(Default)]
pub(super) struct PendingCapabilities {
    pub(super) bindings: HashMap<BindingKey, Vec<usize>>,
    pub(super) direct: HashMap<BindingKey, TransferCapability>,
    checks: Vec<(Vec<usize>, Span)>,
}

impl Checker {
    /// Retains pending capture dependencies through immutable aggregates and pattern bindings.
    pub(in crate::check) fn record_pending_capture_binding(
        &mut self,
        name: &str,
        value: Option<&Expr>,
    ) {
        let Some((_, symbol, Some(span))) = self.scopes.lookup_with_scope_and_declaration(name)
        else {
            return;
        };
        let scalar = !self.type_can_contain_callable(&symbol.ty);
        let targets = if scalar {
            Vec::new()
        } else {
            value
                .map(|expr| self.pending_expr_captures(expr))
                .unwrap_or_default()
        };
        let direct = if scalar {
            TransferCapability::Transferable
        } else {
            value.map_or(TransferCapability::Unknown, |expr| {
                self.direct_expr_capability(expr)
            })
        };
        self.capture_dependencies
            .capabilities
            .direct
            .insert(binding_key(name, span), direct);
        if !targets.is_empty() {
            self.capture_dependencies
                .capabilities
                .bindings
                .insert(binding_key(name, span), targets);
        } else {
            self.capture_dependencies
                .capabilities
                .bindings
                .remove(&binding_key(name, span));
        }
    }

    /// Rechecks an accepted task call once recursive capture capabilities are complete.
    pub(in crate::check) fn defer_capture_task_call(&mut self, call: &Expr, span: Span) {
        let targets = match call {
            Expr::Call { designator, .. } => self.pending_designator_captures(designator),
            Expr::Postfix {
                base, operations, ..
            } => self
                .pending_postfix_captures(base, &operations[..operations.len().saturating_sub(1)]),
            _ => Vec::new(),
        };
        self.defer_capture_boundary(targets, span);
    }

    /// Rechecks a callable transferred through a channel or task-group boundary.
    /// See `docs/pascal/language/functions/closures.md#concurrency`.
    pub(crate) fn defer_capture_transfer(&mut self, expr: &Expr, span: Span) {
        self.defer_capture_boundary(self.pending_expr_captures(expr), span);
    }

    fn defer_capture_boundary(&mut self, targets: Vec<usize>, span: Span) {
        let checks = &mut self.capture_dependencies.capabilities.checks;
        if !targets.is_empty()
            && !checks
                .iter()
                .any(|(existing, location)| existing == &targets && *location == span)
        {
            checks.push((targets, span));
        }
    }

    pub(super) fn pending_expr_captures(&self, expr: &Expr) -> Vec<usize> {
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
            | Expr::NamedArgument { value: inner, .. } => self.pending_expr_captures(inner),
            Expr::Postfix {
                base, operations, ..
            } => self.pending_postfix_captures(base, operations),
            Expr::ArrayLiteral(values, _) => values
                .iter()
                .flat_map(|value| self.pending_expr_captures(value))
                .collect(),
            Expr::If {
                branches,
                else_value,
                ..
            } => branches
                .iter()
                .map(|branch| &branch.value)
                .chain(std::iter::once(else_value.as_ref()))
                .flat_map(|value| self.pending_expr_captures(value))
                .collect(),
            Expr::Case { arms, else_arm, .. } => arms
                .iter()
                .map(|arm| &arm.value)
                .chain(else_arm.iter().map(|else_arm| &else_arm.value))
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
            Expr::Call {
                designator, args, ..
            } if self.is_value_constructor(Self::expr_lookup_key(expr), designator) => args
                .iter()
                .flat_map(|arg| self.pending_expr_captures(arg))
                .collect(),
            Expr::Call {
                designator, args, ..
            } => {
                let key = Self::expr_lookup_key(expr);
                let operation = self
                    .fluent_calls
                    .get(&key)
                    .map(|target| target.name.as_str())
                    .or_else(|| self.intrinsic_calls.get(&key).map(String::as_str))
                    .and_then(crate::std_registry::native_operation_by_implementation);
                if let Some(operation) = operation
                    && let Some(result) = self.expr_types.get(&key)
                {
                    self.pending_native_captures(
                        operation,
                        result,
                        self.pending_designator_captures(designator),
                        args,
                    )
                } else {
                    Vec::new()
                }
            }
            _ => Vec::new(),
        };
        targets.sort_unstable();
        targets.dedup();
        targets
    }

    fn pending_designator_captures(&self, designator: &Designator) -> Vec<usize> {
        if designator.parts.iter().any(|part| {
            let span = match part {
                DesignatorPart::Ident(_, span) | DesignatorPart::Index(_, span) => span,
            };
            self.path_types
                .get(&(span.source_id, span.offset))
                .is_some_and(|ty| !self.type_can_contain_callable(ty))
        }) {
            return Vec::new();
        }
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

    /// Completes captured callable dependencies and diagnoses provisional task transfers.
    pub(in crate::check) fn finish_pending_capture_checks(&mut self) {
        // Start at direct facts, rather than joining provisional Unknown metadata forever.
        // Subsequent joins are monotone over the finite capability lattice.
        let direct = &self.capture_dependencies.capabilities.direct;
        for info in self.closure_infos.values_mut() {
            complete_capabilities(&mut info.captures, direct);
            info.capability = capability_from_captures(&info.captures);
        }
        for info in self.nested_routine_captures.values_mut() {
            complete_capabilities(&mut info.captures, direct);
            info.capability = capability_from_captures(&info.captures);
        }
        loop {
            let mut states = self.capture_dependencies.capabilities.direct.clone();
            for (binding, targets) in &self.capture_dependencies.capabilities.bindings {
                let capability = targets.iter().fold(
                    states.get(binding).copied().unwrap_or_default(),
                    |state, key| {
                        state.join(
                            self.nested_routine_captures
                                .get(key)
                                .map_or(TransferCapability::Unknown, |info| info.capability),
                        )
                    },
                );
                states.insert(binding.clone(), capability);
            }
            let mut changed = false;
            for info in self.closure_infos.values_mut() {
                changed |= complete_capabilities(&mut info.captures, &states);
                let next = capability_from_captures(&info.captures);
                changed |= info.capability != next;
                info.capability = next;
            }
            for info in self.nested_routine_captures.values_mut() {
                changed |= complete_capabilities(&mut info.captures, &states);
                let next = capability_from_captures(&info.captures);
                changed |= info.capability != next;
                info.capability = next;
            }
            if !changed {
                break;
            }
        }
        for (&key, info) in &self.nested_routine_captures {
            self.scopes
                .update_routine_capture_capabilities(key, info.capability);
        }
        for (&key, info) in &self.closure_infos {
            self.expr_capabilities.insert(key, info.capability);
        }
        for (targets, span) in std::mem::take(&mut self.capture_dependencies.capabilities.checks) {
            let invalid = targets.iter().any(|key| {
                self.nested_routine_captures
                    .get(key)
                    .is_some_and(|info| info.capability.is_task_bound())
            });
            if invalid {
                self.error_with_code(SEMA_TASK_BOUND_CALLABLE,
                    "Cannot transfer a task-bound callable across a task boundary",
                    "Mutable captures make a closure task-bound. Pass immutable values instead, or invoke the closure on the same task.", span);
            }
        }
    }
}

fn complete_capabilities(
    captures: &mut [CaptureBinding],
    states: &HashMap<BindingKey, TransferCapability>,
) -> bool {
    let mut changed = false;
    for capture in captures {
        if capture.mutable {
            // Mutable cells already make their closure local; an initializer does
            // not describe the cell's contents when this routine is invoked.
            continue;
        }
        if let Some(&capability) = states.get(&binding_key(&capture.name, capture.declaration)) {
            changed |= capture.capability != capability;
            capture.capability = capability;
        }
    }
    changed
}

pub(super) fn binding_key(name: &str, span: Span) -> BindingKey {
    (
        name.to_ascii_lowercase(),
        span.source_id,
        span.offset,
        span.length,
    )
}
