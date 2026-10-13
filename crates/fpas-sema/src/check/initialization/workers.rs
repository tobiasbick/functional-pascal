//! Worker safety using the declaration-resolved callable dependency summaries.
//! **Documentation:** `docs/pascal/language/functions/closures.md#concurrency`.

use super::collection::Owner;
use super::walker::{Failure, Value, Walker};
use fpas_lexer::Span;

impl Walker<'_, '_> {
    /// Apply callable dependencies as work executed in a separate task.
    pub(super) fn invoke_worker(&mut self, origins: &Value, values: &[Value], span: Span) -> Value {
        let previous = std::mem::replace(&mut self.worker, true);
        let returned = self.invoke(origins, values, span);
        self.worker = previous;
        returned
    }

    /// Preserve uncertainty; dynamic targets retain the VM's mandatory access checks.
    pub(super) fn worker_unknown(&mut self, _span: Span) {
        self.summary.unknown_worker = true;
    }

    /// Retain mutable-global dependencies and reject them only at worker invocation sites.
    pub(super) fn worker_globals(&mut self, names: impl IntoIterator<Item = String>, span: Span) {
        for name in names {
            self.summary.mutable_globals.insert(name.clone());
            if self.worker && self.validate {
                self.task_errors.push(Failure { span, name });
            }
        }
    }

    /// Substitute pending captured worker callables at their lexical owning routine.
    pub(super) fn worker_binding(&mut self, binding: super::BindingId, span: Span) {
        if self
            .model
            .bindings
            .get(&binding)
            .is_some_and(|binding| binding.owner == self.owner)
        {
            let origins = self.binding_value(binding);
            self.invoke_worker(&origins, &[], span);
        } else if self.owner == Owner::Root {
            self.worker_unknown(span);
        } else {
            self.summary.started_bindings.insert(binding);
        }
    }
}
