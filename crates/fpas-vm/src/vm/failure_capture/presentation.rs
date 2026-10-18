//! Budgeted diagnostic copies for retained relationships and terminal delivery.

use super::{CaptureState, Lease, MAX_STRING, ValueBudget};
use crate::vm::VmError;
use fpas_diagnostics::Diagnostic;
use std::{ops::Deref, sync::Arc};

/// Bounded immutable diagnostic presentation; the ordinary VM error is unchanged.
pub struct FailureDiagnostic {
    /// Structured diagnostic fields, with text bounded by the shared capture budget.
    pub diagnostic: VmError,
    /// Stable reason for omitted diagnostic text, when any.
    pub reason: Option<&'static str>,
    _lease: Lease,
}

impl std::fmt::Debug for FailureDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FailureDiagnostic")
            .field("diagnostic", &self.diagnostic)
            .field("reason", &self.reason)
            .finish()
    }
}

impl Deref for FailureDiagnostic {
    type Target = Diagnostic;
    fn deref(&self) -> &Diagnostic {
        &self.diagnostic
    }
}

impl CaptureState {
    /// Copy text only after reserving from the same budget as frame values.
    pub(super) fn presentation(self: &Arc<Self>, error: &VmError) -> Arc<FailureDiagnostic> {
        self.presentation_with_budget(error, false)
    }

    /// The selected diagnostic has a reserved allowance that optional values cannot consume.
    pub(super) fn terminal_presentation(
        self: &Arc<Self>,
        error: &VmError,
    ) -> Arc<FailureDiagnostic> {
        self.presentation_with_budget(error, true)
    }

    fn presentation_with_budget(
        self: &Arc<Self>,
        error: &VmError,
        terminal: bool,
    ) -> Arc<FailureDiagnostic> {
        let (mut usage, mut budget) = if terminal {
            self.terminal_budget()
        } else {
            self.model_budget()
        };
        let before = (budget.values, budget.bytes);
        let mut copy = Box::new(Diagnostic::error_without_source(error.code, "", None));
        copy.severity = error.severity;
        copy.span = error.span;
        let mut reason = None;
        if budget
            .charge(std::mem::size_of::<FailureDiagnostic>() + std::mem::size_of::<Diagnostic>())
        {
            copy.message = bounded_text(&mut budget, &mut reason, &error.message);
            copy.help = error
                .help
                .as_deref()
                .map(|text| bounded_text(&mut budget, &mut reason, text));
            copy.expected = error
                .expected
                .as_deref()
                .map(|text| bounded_text(&mut budget, &mut reason, text).into_boxed_str());
            copy.found = error
                .found
                .as_deref()
                .map(|text| bounded_text(&mut budget, &mut reason, text).into_boxed_str());
        } else {
            reason = budget.exhausted;
        }
        let mut lease = Lease::default();
        lease.charge(self, &mut usage, before, &budget);
        Arc::new(FailureDiagnostic {
            diagnostic: copy,
            reason,
            _lease: lease,
        })
    }
}

fn bounded_text(
    budget: &mut ValueBudget,
    reason: &mut Option<&'static str>,
    value: &str,
) -> String {
    let (value, truncated) = budget.text(value, MAX_STRING);
    if truncated {
        *reason = Some(budget.exhausted.unwrap_or("string_limit"));
    }
    value
}
