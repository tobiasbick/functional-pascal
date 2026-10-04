//! Lexical callable lookup and fallback runtime result types.
//!
//! **Documentation:** `docs/pascal/language/functions/closures.md`.

use fpas_ir::{FunctionId, Operation, TypeId, ValueId};
use fpas_lexer::Span;

use crate::CompileError;

use super::{Callable, CaptureInput, LoweringContext};

impl LoweringContext {
    /// Resolve explicit qualified names or the nearest lexical routine declaration.
    pub(in crate::lowering) fn resolve_callable(&self, name: &str) -> Option<Callable> {
        let canonical = self.qualified_import_name(name).to_ascii_lowercase();
        if self.function_id == FunctionId::new(0) || canonical.contains('.') {
            return self.callables.get(&canonical).cloned();
        }
        let mut scope = self.program_name.as_str();
        loop {
            let child = format!("{scope}.{canonical}");
            if let Some(callable) = self.callables.get(&child) {
                return Some(callable.clone());
            }
            let Some((parent, tail)) = scope.rsplit_once('.') else {
                break;
            };
            if tail == canonical
                && let Some(callable) = self.callables.get(scope)
            {
                return Some(callable.clone());
            }
            scope = parent;
        }
        self.callables.get(&canonical).cloned()
    }

    /// Preserve a selected routine's environment when another closure captures it.
    pub(in crate::lowering) fn read_callable_value(
        &mut self,
        name: &str,
        span: Span,
    ) -> Result<ValueId, CompileError> {
        let callable = self
            .resolve_callable(name)
            .ok_or_else(|| super::unsupported(span, "unresolved callable value"))?;
        let captures = callable
            .captures
            .iter()
            .map(|capture| self.read_closure_capture(capture, span))
            .collect::<Result<Vec<_>, _>>()?;
        self.emit_value(
            Operation::MakeClosure {
                function: callable.function,
                captures,
            },
            callable.value_type,
            span,
        )
    }

    /// Materialize captured routines in their owner with exact debug provenance.
    pub(in crate::lowering) fn read_closure_capture(
        &mut self,
        capture: &CaptureInput,
        span: Span,
    ) -> Result<ValueId, CompileError> {
        if self.has_binding(&capture.name) || self.resolve_callable(&capture.name).is_none() {
            return self.read_capture(&capture.name, span);
        }
        let declaration = capture.declaration.ok_or_else(|| {
            super::unsupported(span, "routine capture without declaration identity")
        })?;
        let value = self.read_callable_value(&capture.name, span)?;
        let local = self
            .debug
            .bindings
            .iter()
            .find(|binding| binding.declaration == Some(declaration) && binding.ty == capture.ty)
            .map(|binding| binding.local);
        let local = if let Some(local) = local {
            local
        } else {
            let local = self.declare_local(&capture.name, capture.ty, false, declaration.into())?;
            // This environment slot must not change lexical routine resolution.
            self.bindings.pop();
            local
        };
        // Every creation writes the slot, including creations in separate branches.
        self.write_local(local, value, span)?;
        Ok(value)
    }

    /// Runtime result fallback when no checked source expression is available.
    pub(in crate::lowering) fn call_result_type(&self, name: &str) -> Option<TypeId> {
        self.bindings
            .iter()
            .rev()
            .find(|binding| binding.name.eq_ignore_ascii_case(name))
            .and_then(|binding| self.type_table.function_result(binding.ty))
            .or_else(|| {
                self.globals
                    .get(&self.qualified_import_name(name).to_ascii_lowercase())
                    .and_then(|global| self.type_table.function_result(global.ty))
            })
            .or_else(|| self.resolve_callable(name).map(|callable| callable.result))
    }
}
