//! Deferred reference-lifetime checks for active recursive routine declarations.
//! See `docs/pascal/language/functions/var-parameters.md`.

use std::collections::HashSet;

use fpas_diagnostics::codes::SEMA_VAR_PARAMETER_ESCAPE;
use fpas_lexer::Span;

use super::super::Checker;

/// An escaping use whose named target's captures are not yet available.
#[derive(Clone)]
pub(crate) enum PendingRoutineUse {
    /// A routine used as a first-class value.
    Value(String, Span),
    /// A routine started as a separate task.
    Task(String, Span),
    /// An anonymous wrapper that requires the named target's captures.
    Closure(Span),
}

impl Checker {
    /// Defers lifetime checks for anonymous wrappers around active named routines.
    /// See `docs/pascal/language/functions/var-parameters.md#lifetime`.
    pub(in crate::check) fn defer_closure_var_captures(&mut self, routines: &[usize], span: Span) {
        for &key in routines {
            self.check_pending_routine_use(key, PendingRoutineUse::Closure(span));
        }
    }

    /// Checks or defers an escaping use at its resolved lexical declaration.
    pub(super) fn defer_routine_reference_use(&mut self, name: &str, span: Span, task: bool) {
        if let Some(key) = self.scopes.routine_capture_key(name) {
            let usage = if task {
                PendingRoutineUse::Task(name.to_string(), span)
            } else {
                PendingRoutineUse::Value(name.to_string(), span)
            };
            self.check_pending_routine_use(key, usage);
        }
    }

    /// Follows completed wrappers and queues unresolved targets without revisiting cycles.
    pub(super) fn check_pending_routine_use(&mut self, key: usize, usage: PendingRoutineUse) {
        let mut pending = vec![key];
        let mut visited = HashSet::new();
        while let Some(key) = pending.pop() {
            if !visited.insert(key) {
                continue;
            }
            if let Some(parameter) = self.var_parameter_routines.get(&key).cloned() {
                self.report_pending_routine_use(&usage, &parameter);
                return;
            }
            if !self.nested_routine_captures.contains_key(&key) {
                self.pending_var_parameter_uses
                    .entry(key)
                    .or_default()
                    .push(usage.clone());
            }
            if let Some(dependencies) = self.pending_routine_captures.get(&key) {
                pending.extend(dependencies);
            }
        }
    }

    /// Finds a reference capture through completed wrappers and recursive dependencies.
    pub(super) fn var_parameter_routine(&self, name: &str) -> Option<String> {
        let mut pending = vec![self.scopes.routine_capture_key(name)?];
        let mut visited = HashSet::new();
        while let Some(key) = pending.pop() {
            if !visited.insert(key) {
                continue;
            }
            if let Some(parameter) = self.var_parameter_routines.get(&key) {
                return Some(parameter.clone());
            }
            if let Some(dependencies) = self.pending_routine_captures.get(&key) {
                pending.extend(dependencies);
            }
        }
        None
    }

    /// Reports the original escaping use after the target's captures are available.
    pub(super) fn report_pending_routine_use(
        &mut self,
        usage: &PendingRoutineUse,
        parameter: &str,
    ) {
        match usage {
            PendingRoutineUse::Value(name, span) => {
                self.report_routine_reference_escape(name, parameter, *span, false);
            }
            PendingRoutineUse::Task(name, span) => {
                self.report_routine_reference_escape(name, parameter, *span, true);
            }
            PendingRoutineUse::Closure(span) => self.error_with_code(
                SEMA_VAR_PARAMETER_ESCAPE,
                format!("A closure cannot capture `var` parameter `{parameter}` through a named routine"),
                "The closure could outlive the call. Copy the value into a local first, and capture the copy.",
                *span,
            ),
        }
    }
}
