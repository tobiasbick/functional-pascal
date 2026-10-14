//! Callable dependencies of imported routines and native catalog callbacks.
//! **Documentation:** `docs/pascal/language/concurrency/go.md`.

use super::walker::{Origin, Value, Walker};
use fpas_lexer::Span;

impl Walker<'_, '_> {
    /// Apply an independently compiled routine's callable dependencies.
    pub(super) fn imported(&mut self, name: &str, values: &[Value], span: Span) -> Value {
        let canonical = self.checker.scopes.imports.canonical_name(name);
        let Some(summary) = self.checker.initialization.imported.get(&canonical) else {
            self.unknown(span);
            return [Origin::Unknown].into_iter().collect();
        };
        self.worker_globals(summary.mutable_globals.iter().cloned(), span);
        for &index in &summary.started_parameters {
            self.invoke_worker(values.get(index).unwrap_or(&Value::new()), &[], span);
        }
        if summary.unknown_worker {
            self.worker_unknown(span);
        }
        for &index in &summary.called_parameters {
            self.invoke(values.get(index).unwrap_or(&Value::new()), &[], span);
        }
        if summary.unknown_call {
            self.unknown(span);
        }
        let mut returned: Value = summary
            .returned_parameters
            .iter()
            .flat_map(|&index| values.get(index).cloned().unwrap_or_default())
            .collect();
        if summary.unknown_return {
            returned.insert(Origin::Unknown);
        }
        returned
    }

    /// Check declared catalog callback roles without invoking ordinary stored values.
    pub fn standard(&mut self, name: &str, values: &[Value], receiver: Value, span: Span) -> Value {
        if self
            .checker
            .initialization
            .imported
            .contains_key(&self.checker.scopes.imports.canonical_name(name))
        {
            return self.imported(name, values, span);
        }
        let parameters = crate::std_registry::native_operation_by_implementation(name)
            .map(|operation| (operation.signature)().params)
            .or_else(|| {
                self.checker
                    .scopes
                    .lookup(name)
                    .and_then(|symbol| match &symbol.ty {
                        crate::types::Ty::Function(function) => Some(function.params.clone()),
                        crate::types::Ty::Procedure(procedure) => Some(procedure.params.clone()),
                        _ => None,
                    })
            })
            .unwrap_or_default();
        let starts_worker = matches!(
            name.rsplit('.')
                .next()
                .unwrap_or("")
                .to_ascii_lowercase()
                .as_str(),
            "starttaskingroup" | "startsupervisedtask"
        );
        if starts_worker {
            // These polymorphic intrinsics do not expose their callback through
            // an ordinary concrete function parameter in the catalog signature.
            self.invoke_worker(values.get(1).unwrap_or(&Value::new()), &[], span);
        }
        for (index, parameter) in parameters.iter().enumerate() {
            if matches!(
                parameter.ty,
                crate::types::Ty::Function(_) | crate::types::Ty::Procedure(_)
            ) && !(starts_worker && index == 1)
            {
                self.invoke(values.get(index).unwrap_or(&Value::new()), &[], span);
            }
        }
        let mut returned: Value = receiver
            .into_iter()
            .chain(values.iter().flatten().copied())
            .collect();
        if !returned.is_empty() {
            returned.insert(Origin::Unknown);
        }
        returned
    }
}
