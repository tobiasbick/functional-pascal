//! Lexical bindings, callable lookup, and capture-cell access.

use fpas_ir::{Local, LocalId, Operation, TypeId, ValueId};
use fpas_lexer::Span;

use crate::CompileError;
use crate::error::internal_compiler_error;

use super::{Binding, BindingStorage, Callable, ClosureTarget, LoweringContext};

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
        self.cell_names.contains(&name.to_ascii_lowercase())
    }

    pub(in crate::lowering) fn mark_binding_cell(&mut self, name: &str, logical_ty: TypeId) {
        if let Some(binding) = self
            .bindings
            .iter_mut()
            .rev()
            .find(|binding| binding.name.eq_ignore_ascii_case(name))
        {
            binding.ty = logical_ty;
            binding.cell = true;
        }
    }

    pub(in crate::lowering) fn cell_type(
        &mut self,
        ty: TypeId,
        span: Span,
    ) -> Result<TypeId, CompileError> {
        self.type_table.cell_type(ty, span)
    }

    pub(in crate::lowering) fn array_type(
        &mut self,
        element: TypeId,
        span: Span,
    ) -> Result<TypeId, CompileError> {
        self.type_table.array_type(element, span)
    }

    pub(in crate::lowering) fn resolve_callable(&self, name: &str) -> Option<Callable> {
        self.callables.get(&name.to_ascii_lowercase()).cloned()
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
            .map(|binding| match binding.storage {
                BindingStorage::Local(local) => local,
            })
    }

    pub(in crate::lowering) fn binding_type(&self, name: &str) -> Option<TypeId> {
        self.bindings
            .iter()
            .rev()
            .find(|binding| binding.name.eq_ignore_ascii_case(name))
            .map(|binding| binding.ty)
    }

    pub(in crate::lowering) fn has_global(&self, name: &str) -> bool {
        self.globals.contains_key(&name.to_ascii_lowercase())
    }

    pub(in crate::lowering) fn constant(&self, name: &str) -> Option<fpas_ir::Constant> {
        self.constants.get(&name.to_ascii_lowercase()).cloned()
    }

    pub(in crate::lowering) fn current_result_type(&self) -> TypeId {
        self.result_type
    }

    pub(in crate::lowering) fn read_global(
        &mut self,
        name: &str,
        span: Span,
    ) -> Result<ValueId, CompileError> {
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
        self.emit_value(Operation::LoadGlobal(global.id), global.ty, span)
    }

    pub(in crate::lowering) fn write_global(
        &mut self,
        name: &str,
        value: ValueId,
        span: Span,
    ) -> Result<(), CompileError> {
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
        self.emit_effect(
            Operation::StoreGlobal {
                global: global.id,
                value,
            },
            span,
        )
    }

    /// Emits one typed update of an index-only path below a global snapshot.
    pub(in crate::lowering) fn write_global_index_path(
        &mut self,
        name: &str,
        root: ValueId,
        indexes: Vec<ValueId>,
        value: ValueId,
        span: Span,
    ) -> Result<(), CompileError> {
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
        self.emit_effect(
            Operation::StoreGlobalIndexPath {
                global: global.id,
                root,
                indexes,
                value,
            },
            span,
        )
    }

    /// Returns whether the global slot fits the compact direct-path opcode.
    pub(in crate::lowering) fn global_index_path_uses_u16_slot(&self, name: &str) -> bool {
        self.globals
            .get(&name.to_ascii_lowercase())
            .is_some_and(|global| u16::try_from(global.id.get()).is_ok())
    }

    pub(in crate::lowering) fn call_result_type(&self, name: &str) -> Option<TypeId> {
        self.bindings
            .iter()
            .rev()
            .find(|binding| binding.name.eq_ignore_ascii_case(name))
            .and_then(|binding| self.type_table.function_result(binding.ty))
            .or_else(|| self.resolve_callable(name).map(|callable| callable.result))
    }

    pub(in crate::lowering) fn root_type(&self, name: &str) -> Option<TypeId> {
        self.bindings
            .iter()
            .rev()
            .find(|binding| binding.name.eq_ignore_ascii_case(name))
            .map(|binding| binding.ty)
            .or_else(|| {
                self.globals
                    .get(&name.to_ascii_lowercase())
                    .map(|global| global.ty)
            })
    }

    pub(in crate::lowering) fn designator_type(
        &self,
        designator: &fpas_parser::Designator,
    ) -> Option<TypeId> {
        let fpas_parser::DesignatorPart::Ident(name, _) = designator.parts.first()? else {
            return None;
        };
        let mut ty = self.root_type(name)?;
        for part in &designator.parts[1..] {
            ty = match (part, self.type_kind(ty)?) {
                (fpas_parser::DesignatorPart::Ident(name, _), fpas_ir::IrType::Record(layout)) => {
                    self.record_field(layout, name)?.1
                }
                (fpas_parser::DesignatorPart::Index(_, _), fpas_ir::IrType::Array(element)) => {
                    element
                }
                (
                    fpas_parser::DesignatorPart::Index(_, _),
                    fpas_ir::IrType::Dictionary { value, .. },
                ) => value,
                (fpas_parser::DesignatorPart::Index(_, _), fpas_ir::IrType::String) => {
                    super::types::STRING
                }
                _ => return None,
            };
        }
        Some(ty)
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

    pub(in crate::lowering) fn record_layout_name(
        &self,
        layout: fpas_ir::RecordLayoutId,
    ) -> Option<&str> {
        self.type_table.record_layout_name(layout)
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

    pub(in crate::lowering) fn begin_scope(&mut self) {
        self.scope_depth = self.scope_depth.saturating_add(1);
    }

    pub(in crate::lowering) fn end_scope(&mut self) {
        let depth = self.scope_depth;
        self.bindings.retain(|binding| binding.depth < depth);
        self.scope_depth = self.scope_depth.saturating_sub(1);
    }
}
