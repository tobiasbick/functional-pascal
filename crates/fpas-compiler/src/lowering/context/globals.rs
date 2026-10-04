//! Global bindings, constants, and writes to imported unit values.

use super::LoweringContext;
use crate::{CompileError, error::internal_compiler_error};
use fpas_ir::{Operation, ValueId};
use fpas_lexer::Span;

impl LoweringContext {
    /// Reports whether a lexical or alias-qualified name identifies a global slot.
    pub(in crate::lowering) fn has_global(&self, name: &str) -> bool {
        self.globals
            .contains_key(&self.qualified_import_name(name).to_ascii_lowercase())
    }

    /// Resolves a lexical or alias-qualified compile-time constant.
    pub(in crate::lowering) fn constant(&self, name: &str) -> Option<fpas_ir::Constant> {
        self.constants
            .get(&self.qualified_import_name(name).to_ascii_lowercase())
            .cloned()
    }

    /// Emits a read from a resolved global slot.
    pub(in crate::lowering) fn read_global(
        &mut self,
        name: &str,
        span: Span,
    ) -> Result<ValueId, CompileError> {
        let global = self
            .globals
            .get(&self.qualified_import_name(name).to_ascii_lowercase())
            .copied()
            .ok_or_else(|| {
                internal_compiler_error(
                    format!("Global `{name}` is missing from lowering metadata."),
                    "Re-run compilation and report the source program.",
                    span.line,
                    span.column,
                )
            })?;
        if global.cell {
            let storage_ty = self.cell_type(global.ty, span)?;
            let cell = self.emit_value(Operation::LoadGlobal(global.id), storage_ty, span)?;
            self.emit_value(Operation::CellRead(cell), global.ty, span)
        } else {
            self.emit_value(Operation::LoadGlobal(global.id), global.ty, span)
        }
    }

    /// Emits a write to a resolved global slot.
    pub(in crate::lowering) fn write_global(
        &mut self,
        name: &str,
        value: ValueId,
        span: Span,
    ) -> Result<(), CompileError> {
        let global = self
            .globals
            .get(&self.qualified_import_name(name).to_ascii_lowercase())
            .copied()
            .ok_or_else(|| {
                internal_compiler_error(
                    format!("Global `{name}` is missing from lowering metadata."),
                    "Re-run compilation and report the source program.",
                    span.line,
                    span.column,
                )
            })?;
        if global.cell {
            let storage_ty = self.cell_type(global.ty, span)?;
            let cell = self.emit_value(Operation::LoadGlobal(global.id), storage_ty, span)?;
            return self.emit_effect(Operation::CellWrite { cell, value }, span);
        }
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
            .get(&self.qualified_import_name(name).to_ascii_lowercase())
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
            .get(&self.qualified_import_name(name).to_ascii_lowercase())
            .is_some_and(|global| !global.cell && u16::try_from(global.id.get()).is_ok())
    }

    /// Obtain a mutable global's stable root before reserving caller storage.
    pub(in crate::lowering) fn read_global_storage_root(
        &mut self,
        name: &str,
        span: Span,
    ) -> Result<ValueId, CompileError> {
        let global = self
            .globals
            .get(&self.qualified_import_name(name).to_ascii_lowercase())
            .copied()
            .filter(|global| global.cell)
            .ok_or_else(|| super::unsupported(span, "var global without writable cell storage"))?;
        let storage_ty = self.cell_type(global.ty, span)?;
        self.emit_value(Operation::LoadGlobal(global.id), storage_ty, span)
    }
}
