//! Bytecode emission for closure capture environments.
//!
//! **Documentation:** `docs/pascal/language/functions/closures.md`

use crate::error::CompileError;
use fpas_bytecode::{Op, SourceLocation, Value};
use fpas_parser::Expr;
use fpas_sema::CaptureBinding;

use super::super::{Compiler, LocalRef};

impl Compiler {
    fn local_ref_is_cell(&self, local_ref: LocalRef) -> bool {
        match local_ref {
            LocalRef::Local(slot) => self.local_is_cell(slot),
            LocalRef::Enclosing(depth, slot) => self.enclosing_is_cell(depth, slot),
        }
    }

    fn enclosing_is_cell(&self, depth: u16, slot: u16) -> bool {
        self.enclosing_locals
            .iter()
            .rev()
            .nth((depth as usize).saturating_sub(1))
            .and_then(|frame| frame.iter().rev().find(|local| local.slot == slot))
            .is_some_and(|local| local.is_cell)
    }

    fn emit_local_ref_raw_read(&mut self, local_ref: LocalRef, location: SourceLocation) {
        match local_ref {
            LocalRef::Local(slot) => self.emit(Op::GetLocal(slot), location),
            LocalRef::Enclosing(depth, slot) => self.emit(Op::GetEnclosing(depth, slot), location),
        };
    }

    /// Load a resolved local, dereferencing its capture cell when necessary.
    pub(in crate::compiler) fn emit_local_ref_read(
        &mut self,
        local_ref: LocalRef,
        location: SourceLocation,
    ) {
        self.emit_local_ref_raw_read(local_ref, location);
        if self.local_ref_is_cell(local_ref) {
            self.emit(Op::CellGet, location);
        }
    }

    /// Load a mutable aggregate and retain its cell handle for the later write-back.
    pub(in crate::compiler) fn emit_local_ref_update_start(
        &mut self,
        local_ref: LocalRef,
        location: SourceLocation,
    ) {
        if self.local_ref_is_cell(local_ref) {
            self.emit_local_ref_raw_read(local_ref, location);
        }
        self.emit_local_ref_read(local_ref, location);
    }

    /// Store an aggregate update into its original local or capture cell.
    pub(in crate::compiler) fn emit_local_ref_update_finish(
        &mut self,
        local_ref: LocalRef,
        location: SourceLocation,
    ) {
        if self.local_ref_is_cell(local_ref) {
            self.emit(Op::CellSet, location);
        } else {
            match local_ref {
                LocalRef::Local(slot) => self.emit(Op::SetLocal(slot), location),
                LocalRef::Enclosing(depth, slot) => {
                    self.emit(Op::SetEnclosing(depth, slot), location)
                }
            };
        }
    }

    /// Promote a local stack slot to a shared closure cell once.
    pub(in crate::compiler) fn ensure_local_is_cell(
        &mut self,
        slot: u16,
        location: SourceLocation,
    ) {
        let Some(index) = self.locals.iter().position(|local| local.slot == slot) else {
            return;
        };
        if self.locals[index].is_cell {
            return;
        }

        self.emit(Op::GetLocal(slot), location);
        self.emit(Op::MakeCell, location);
        self.emit(Op::SetLocal(slot), location);
        self.emit(Op::Pop, location);
        self.locals[index].is_cell = true;
    }

    /// Push a current binding in the representation needed by a closure capture.
    pub(in crate::compiler) fn emit_load_capture(
        &mut self,
        capture: &CaptureBinding,
        location: SourceLocation,
    ) {
        match self.resolve_local(&capture.name) {
            Some(LocalRef::Local(slot)) => {
                if capture.mutable {
                    self.ensure_local_is_cell(slot, location);
                }
                self.emit(Op::GetLocal(slot), location);
            }
            Some(LocalRef::Enclosing(depth, slot)) => {
                if capture.mutable {
                    self.ensure_enclosing_is_cell(depth, slot, location);
                }
                self.emit(Op::GetEnclosing(depth, slot), location);
            }
            None => {}
        }
    }

    /// Promote an enclosing-frame local to a shared closure cell once.
    pub(in crate::compiler) fn ensure_enclosing_is_cell(
        &mut self,
        depth: u16,
        slot: u16,
        location: SourceLocation,
    ) {
        let already_cell = self.enclosing_is_cell(depth, slot);
        if already_cell {
            return;
        }

        self.emit(Op::GetEnclosing(depth, slot), location);
        self.emit(Op::MakeCell, location);
        self.emit(Op::SetEnclosing(depth, slot), location);
        self.emit(Op::Pop, location);

        if let Some(frame) = self
            .enclosing_locals
            .iter_mut()
            .rev()
            .nth((depth as usize).saturating_sub(1))
            && let Some(local) = frame.iter_mut().find(|local| local.slot == slot)
        {
            local.is_cell = true;
        }
    }

    /// Build a closure value for a named nested routine when it captures bindings.
    pub(in crate::compiler) fn emit_captured_routine_closure(
        &mut self,
        name: &str,
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        let canonical = super::super::canonical_name(name);
        let Some(capture_info) = self
            .nested_routine_captures
            .get(&canonical)
            .filter(|info| !info.captures.is_empty())
            .cloned()
        else {
            return Ok(false);
        };

        for capture in &capture_info.captures {
            self.emit_load_capture(capture, location);
        }
        let runtime_name = self.qualify_owned_name(&canonical);
        let name_idx = self.add_constant(
            Value::Str(super::super::canonical_name(&runtime_name).into()),
            location,
        )?;
        let capture_count =
            Self::checked_u8_at(capture_info.captures.len(), "routine captures", location)?;
        self.emit(Op::MakeClosure(name_idx, capture_count), location);
        Ok(true)
    }

    /// Load a current local, dereferencing a cell-backed binding.
    pub(in crate::compiler) fn emit_local_read(&mut self, slot: u16, location: SourceLocation) {
        self.emit(Op::GetLocal(slot), location);
        if self.local_is_cell(slot) {
            self.emit(Op::CellGet, location);
        }
    }

    /// Compile and store a value into a current local or its backing cell.
    pub(in crate::compiler) fn emit_local_write(
        &mut self,
        slot: u16,
        value: &Expr,
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        if self.local_is_cell(slot) {
            self.emit(Op::GetLocal(slot), location);
            self.compile_expr(value)?;
            self.emit(Op::CellSet, location);
        } else {
            self.compile_expr(value)?;
            self.emit(Op::SetLocal(slot), location);
        }
        Ok(())
    }

    /// Whether a current-frame slot holds a closure cell.
    pub(in crate::compiler) fn local_is_cell(&self, slot: u16) -> bool {
        self.locals
            .iter()
            .rev()
            .find(|local| local.slot == slot)
            .is_some_and(|local| local.is_cell)
    }
}
