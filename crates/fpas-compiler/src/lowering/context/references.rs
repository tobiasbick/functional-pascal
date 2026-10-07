//! Reference storage for `var` parameters and the roots of `var` arguments.
//!
//! **Documentation:** `docs/pascal/language/functions/var-parameters.md`

use fpas_ir::{Operation, TypeId, ValueId};
use fpas_lexer::Span;

use crate::CompileError;
use crate::error::internal_compiler_error;

use super::{BindingStorage, LoweringContext};

impl LoweringContext {
    /// Returns whether a binding stores a `var` parameter reference.
    pub(in crate::lowering) fn binding_is_reference(&self, name: &str) -> bool {
        self.bindings
            .iter()
            .rev()
            .find(|binding| binding.name.eq_ignore_ascii_case(name))
            .is_some_and(|binding| binding.reference)
    }

    /// Creates a reference to the local or global variable `name` with type `ty`.
    ///
    /// A forwarded `var` parameter passes its own reference; a local is cell-backed
    /// because it is passed as a `var` argument.
    pub(in crate::lowering) fn reference_to_variable(
        &mut self,
        name: &str,
        ty: TypeId,
        span: Span,
    ) -> Result<ValueId, CompileError> {
        let reference_ty = self.type_table.reference_type(ty, span)?;
        if let Some(binding) = self
            .bindings
            .iter()
            .rev()
            .find(|binding| binding.name.eq_ignore_ascii_case(name))
            .cloned()
        {
            let BindingStorage::Local(local) = binding.storage;
            if binding.reference {
                return self.emit_value(Operation::ReadLocal(local), reference_ty, span);
            }
            if binding.cell {
                let cell_ty = self.cell_type(ty, span)?;
                let cell = self.emit_value(Operation::ReadLocal(local), cell_ty, span)?;
                return self.emit_value(Operation::MakeCellReference(cell), reference_ty, span);
            }
            return Err(internal_compiler_error(
                format!("Local `{name}` is passed as a `var` argument but is not cell-backed."),
                "This is an internal compiler error. Re-run compilation and report the source program.",
                span.line,
                span.column,
            ));
        }
        let global = self
            .globals
            .get(&name.to_ascii_lowercase())
            .copied()
            .ok_or_else(|| {
                internal_compiler_error(
                    format!("Global `{name}` is missing from lowering metadata."),
                    "Re-run compilation and report the source program.",
                    span.line,
                    span.column,
                )
            })?;
        self.emit_value(
            Operation::MakeGlobalReference(global.id),
            reference_ty,
            span,
        )
    }

    pub(super) fn binding_storage_type(
        &mut self,
        ty: TypeId,
        cell: bool,
        reference: bool,
        span: Span,
    ) -> Result<TypeId, CompileError> {
        if cell {
            self.cell_type(ty, span)
        } else if reference {
            self.type_table.reference_type(ty, span)
        } else {
            Ok(ty)
        }
    }
}
