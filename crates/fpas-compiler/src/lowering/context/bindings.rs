//! Lexical value storage and capture-cell access.

use fpas_ir::{Local, LocalId, Operation, TypeId, ValueId};
use fpas_lexer::Span;

use crate::CompileError;
use crate::error::internal_compiler_error;

use super::{Binding, BindingStorage, ClosureTarget, LoweringContext};

impl LoweringContext {
    pub(in crate::lowering) fn declare_local(
        &mut self,
        name: &str,
        ty: TypeId,
        mutable: bool,
        span: Span,
    ) -> Result<LocalId, CompileError> {
        let local = LocalId::try_from_index(self.locals.len()).map_err(|error| {
            internal_compiler_error(
                error.to_string(),
                "Split the program into smaller functions.",
                span.line,
                span.column,
            )
        })?;
        self.locals.push(Local {
            id: local,
            ty,
            mutable,
            capture: None,
        });
        let hidden = name.starts_with("$p4_");
        self.debug.bindings.push(fpas_ir::DebugBinding {
            local,
            name: name.to_string(),
            kind: fpas_ir::DebugBindingKind::Local,
            ty,
            mutable,
            scope: self.debug_scope,
            declaration: Some(span.diagnostic_span_or_synthetic()),
            hidden,
            cell_backed: false,
            initializer: None,
        });
        self.bindings.push(Binding {
            name: name.to_ascii_lowercase(),
            storage: BindingStorage::Local(local),
            ty,
            depth: self.scope_depth,
            cell: false,
        });
        Ok(local)
    }

    pub(in crate::lowering) fn declare_hidden_local(
        &mut self,
        ty: TypeId,
        span: Span,
    ) -> Result<LocalId, CompileError> {
        let name = format!("$p4_{}", self.locals.len());
        self.declare_local(&name, ty, true, span)
    }

    fn resolve_local(
        &self,
        name: &str,
        span: Span,
    ) -> Result<(BindingStorage, TypeId), CompileError> {
        self.bindings.iter().rev().find(|binding| binding.name.eq_ignore_ascii_case(name)).map(|binding| (binding.storage, binding.ty)).ok_or_else(|| internal_compiler_error(format!("Local `{name}` was not present in register-lowering scope metadata."), "This is an internal compiler error. Re-run compilation and report the source program.", span.line, span.column))
    }

    fn binding_is_cell(&self, name: &str) -> bool {
        self.bindings
            .iter()
            .rev()
            .find(|binding| binding.name.eq_ignore_ascii_case(name))
            .is_some_and(|binding| binding.cell)
    }

    pub(in crate::lowering) fn read_named_local(
        &mut self,
        name: &str,
        span: Span,
    ) -> Result<ValueId, CompileError> {
        let (storage, ty) = self.resolve_local(name, span)?;
        let cell = self.binding_is_cell(name);
        match storage {
            BindingStorage::Reference(local) => {
                let storage_ty = self.reference_type(ty, span)?;
                let reference = self.emit_value(Operation::ReadLocal(local), storage_ty, span)?;
                self.emit_value(
                    Operation::Reference(fpas_ir::ReferenceOperation::Read(reference)),
                    ty,
                    span,
                )
            }
            BindingStorage::Local(local) => {
                let storage_ty = if cell { self.cell_type(ty, span)? } else { ty };
                let value = self.emit_value(Operation::ReadLocal(local), storage_ty, span)?;
                if cell {
                    self.emit_value(Operation::CellRead(value), ty, span)
                } else {
                    Ok(value)
                }
            }
        }
    }

    pub(in crate::lowering) fn read_capture(
        &mut self,
        name: &str,
        span: Span,
    ) -> Result<ValueId, CompileError> {
        let (storage, ty) = self.resolve_local(name, span)?;
        let cell = self.binding_is_cell(name);
        match storage {
            BindingStorage::Reference(_) => {
                Err(super::unsupported(span, "capturing a var parameter"))
            }
            BindingStorage::Local(local) => {
                let storage_ty = if cell { self.cell_type(ty, span)? } else { ty };
                self.emit_value(Operation::ReadLocal(local), storage_ty, span)
            }
        }
    }

    pub(in crate::lowering) fn write_named_local(
        &mut self,
        name: &str,
        value: ValueId,
        span: Span,
    ) -> Result<(), CompileError> {
        let (storage, ty) = self.resolve_local(name, span)?;
        match (storage, self.binding_is_cell(name)) {
            (BindingStorage::Reference(local), _) => {
                let reference_ty = self.reference_type(ty, span)?;
                let reference = self.emit_value(Operation::ReadLocal(local), reference_ty, span)?;
                self.emit_effect(
                    Operation::Reference(fpas_ir::ReferenceOperation::Write { reference, value }),
                    span,
                )
            }
            (BindingStorage::Local(local), false) => self.write_local(local, value, span),
            (BindingStorage::Local(local), true) => {
                let cell_ty = self.cell_type(ty, span)?;
                let cell = self.emit_value(Operation::ReadLocal(local), cell_ty, span)?;
                self.emit_effect(Operation::CellWrite { cell, value }, span)
            }
        }
    }

    pub(in crate::lowering) fn closure_target(
        &self,
        expression: &fpas_parser::Expr,
    ) -> Option<ClosureTarget> {
        self.closure_targets
            .get(&fpas_sema::expr_lookup_key(expression))
            .cloned()
    }

    pub(in crate::lowering) fn is_cell_backed(&self, name: &str) -> bool {
        self.cell_names
            .contains(&self.qualified_import_name(name).to_ascii_lowercase())
    }

    pub(in crate::lowering) fn mark_binding_cell(&mut self, name: &str, logical_ty: TypeId) {
        if let Some(binding) = self
            .bindings
            .iter_mut()
            .rev()
            .find(|binding| binding.name.eq_ignore_ascii_case(name))
        {
            let BindingStorage::Local(local) = binding.storage else {
                return;
            };
            binding.ty = logical_ty;
            binding.cell = true;
            if let Some(debug) = self
                .debug
                .bindings
                .iter_mut()
                .find(|candidate| candidate.local == local)
            {
                debug.ty = logical_ty;
                debug.cell_backed = true;
            }
        }
    }

    pub(in crate::lowering) fn cell_type(
        &mut self,
        ty: TypeId,
        span: Span,
    ) -> Result<TypeId, CompileError> {
        self.type_table.cell_type(ty, span)
    }

    pub(in crate::lowering) fn reference_type(
        &mut self,
        ty: TypeId,
        span: Span,
    ) -> Result<TypeId, CompileError> {
        self.type_table.reference_type(ty, span)
    }

    pub(in crate::lowering) fn read_local_storage_root(
        &mut self,
        name: &str,
        span: Span,
    ) -> Result<ValueId, CompileError> {
        let (storage, ty) = self.resolve_local(name, span)?;
        match storage {
            BindingStorage::Reference(local) => {
                let reference_ty = self.reference_type(ty, span)?;
                self.emit_value(Operation::ReadLocal(local), reference_ty, span)
            }
            BindingStorage::Local(_) if self.binding_is_cell(name) => self.read_capture(name, span),
            BindingStorage::Local(_) => Err(super::unsupported(
                span,
                "var root without stable cell storage",
            )),
        }
    }

    pub(in crate::lowering) fn array_type(
        &mut self,
        element: TypeId,
        span: Span,
    ) -> Result<TypeId, CompileError> {
        self.type_table.array_type(element, span)
    }

    pub(in crate::lowering) fn has_binding(&self, name: &str) -> bool {
        self.bindings
            .iter()
            .rev()
            .any(|binding| binding.name.eq_ignore_ascii_case(name))
    }

    /// Resolve a local whose value can be updated without capture-cell indirection.
    pub(in crate::lowering) fn direct_local(&self, name: &str) -> Option<LocalId> {
        if self.binding_is_cell(name) {
            return None;
        }
        self.bindings
            .iter()
            .rev()
            .find(|binding| binding.name.eq_ignore_ascii_case(name))
            .and_then(|binding| match binding.storage {
                BindingStorage::Local(local) => Some(local),
                BindingStorage::Reference(_) => None,
            })
    }

    pub(in crate::lowering) fn current_result_type(&self) -> TypeId {
        self.result_type
    }

    pub(in crate::lowering) fn root_type(&self, name: &str) -> Option<TypeId> {
        self.bindings
            .iter()
            .rev()
            .find(|binding| binding.name.eq_ignore_ascii_case(name))
            .map(|binding| binding.ty)
            .or_else(|| {
                self.globals
                    .get(&self.qualified_import_name(name).to_ascii_lowercase())
                    .map(|global| global.ty)
            })
    }

    pub(in crate::lowering) fn type_kind(&self, ty: TypeId) -> Option<fpas_ir::IrType> {
        self.type_table.kind(ty).cloned()
    }

    pub(in crate::lowering) fn record_field(
        &self,
        layout: fpas_ir::RecordLayoutId,
        name: &str,
    ) -> Option<(fpas_ir::FieldId, TypeId)> {
        self.type_table.record_field(layout, name)
    }

    pub(in crate::lowering) fn record_fields(
        &self,
        layout: fpas_ir::RecordLayoutId,
    ) -> Option<Vec<(String, TypeId)>> {
        self.type_table.record_fields(layout)
    }

    pub(in crate::lowering) fn record_layout_id(
        &self,
        ty: TypeId,
    ) -> Option<fpas_ir::RecordLayoutId> {
        self.type_table.record_layout_id(ty)
    }

    pub(in crate::lowering) fn enum_variant(
        &self,
        layout: fpas_ir::EnumLayoutId,
        name: &str,
    ) -> Option<(fpas_ir::VariantId, Vec<TypeId>)> {
        self.type_table.enum_variant(layout, name)
    }

    pub(in crate::lowering) fn record_call_arguments(
        &mut self,
        count: usize,
        span: Span,
    ) -> Result<(), CompileError> {
        let count = fpas_ir::checked_count("call argument count", count).map_err(|error| {
            internal_compiler_error(
                error.to_string(),
                "Split this call into smaller operations.",
                span.line,
                span.column,
            )
        })?;
        self.max_call_arguments = self.max_call_arguments.max(count);
        Ok(())
    }

    pub(in crate::lowering) fn write_local(
        &mut self,
        local: LocalId,
        value: ValueId,
        span: Span,
    ) -> Result<(), CompileError> {
        self.emit_effect(Operation::WriteLocal { value, local }, span)
    }

    pub(in crate::lowering) fn initialize_local(
        &mut self,
        local: LocalId,
        value: ValueId,
        span: Span,
    ) -> Result<(), CompileError> {
        let location = self.emit_initializer_store(Operation::WriteLocal { value, local }, span)?;
        let binding = self
            .debug
            .bindings
            .iter_mut()
            .find(|binding| binding.local == local)
            .ok_or_else(|| {
                internal_compiler_error(
                    format!(
                        "Local {} has no debugger binding for its initializer.",
                        local.get()
                    ),
                    "This is an internal compiler error. Re-run compilation and report the source program.",
                    span.line,
                    span.column,
                )
            })?;
        binding.initializer = Some(location);
        Ok(())
    }

    pub(in crate::lowering) fn begin_scope(&mut self) {
        self.scope_depth = self.scope_depth.saturating_add(1);
        self.begin_debug_scope();
    }

    pub(in crate::lowering) fn end_scope(&mut self) {
        let depth = self.scope_depth;
        self.bindings.retain(|binding| binding.depth < depth);
        self.scope_depth = self.scope_depth.saturating_sub(1);
        self.end_debug_scope();
    }
}
