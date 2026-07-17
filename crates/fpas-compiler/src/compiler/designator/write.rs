use crate::error::{CompileError, compile_error};
use fpas_bytecode::{Op, SourceLocation, Value};
use fpas_diagnostics::codes::COMPILE_INVALID_ASSIGNMENT_TARGET;
use fpas_parser::{Designator, DesignatorPart, Expr};

use super::super::canonical_name;
use super::{Compiler, LocalRef};

impl Compiler {
    /// Compile a designator assignment (e.g. `X := val`, `Arr[i] := val`, `P.X := val`).
    pub(in super::super) fn compile_designator_write(
        &mut self,
        target: &Designator,
        value: &Expr,
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        let full_name = Self::resolve_designator_name(target);
        let qualified = self.qualify_name(&full_name).to_string();

        let mut parts = target.parts.iter();
        let base_name = match parts.next() {
            Some(DesignatorPart::Ident(name, _)) => name.clone(),
            _ => {
                return Err(compile_error(
                    COMPILE_INVALID_ASSIGNMENT_TARGET,
                    "Expected identifier in assignment",
                    "The left-hand side of an assignment must be a variable or field.",
                    target.span,
                ));
            }
        };

        let remaining: Vec<_> = parts.collect();

        // Whole-variable assignment: `X := v`, a linked qualified name registered as a
        // local (e.g. `Unit.__private__.State := v`), or a dotted module global.
        // A dotted chain rooted in a global record (e.g. `State.CenterX := v`) is NOT a
        // whole-variable write and must compile as a field-write chain on the base global,
        // mirroring `compile_designator_read`.
        let all_idents = target
            .parts
            .iter()
            .all(|part| matches!(part, DesignatorPart::Ident(_, _)));
        let is_simple_target = all_idents
            && (target.parts.len() == 1
                || (self.resolve_local(&base_name).is_none()
                    && (self.resolve_local(&qualified).is_some()
                        || self.module_globals.contains(&canonical_name(&qualified)))));

        if is_simple_target {
            if let Some(local_ref) = self.resolve_local(&qualified) {
                match local_ref {
                    LocalRef::Local(slot) => {
                        self.emit_local_write(slot, value, location)?;
                    }
                    LocalRef::Enclosing(depth, slot) => {
                        self.compile_expr(value)?;
                        self.emit(Op::SetEnclosing(depth, slot), location);
                    }
                };
                self.emit(Op::Pop, location);
            } else {
                self.compile_expr(value)?;
                let idx = self.add_constant(Value::Str(qualified), location)?;
                self.emit(Op::SetGlobal(idx), location);
                self.emit(Op::Pop, location);
            }
            return Ok(());
        }

        if remaining.is_empty() {
            if let Some(local_ref) = self.resolve_local(&base_name) {
                match local_ref {
                    LocalRef::Local(slot) => {
                        self.emit_local_write(slot, value, location)?;
                    }
                    LocalRef::Enclosing(depth, slot) => {
                        self.compile_expr(value)?;
                        self.emit(Op::SetEnclosing(depth, slot), location);
                    }
                };
                self.emit(Op::Pop, location);
            } else {
                self.compile_expr(value)?;
                let idx = self.add_constant(Value::Str(base_name), location)?;
                self.emit(Op::SetGlobal(idx), location);
                self.emit(Op::Pop, location);
            }
        } else {
            if let Some(local_ref) = self.resolve_local(&base_name) {
                self.emit_local_ref_update_start(local_ref, location);
            } else {
                let idx = self.add_constant(Value::Str(base_name.clone()), location)?;
                self.emit(Op::GetGlobal(idx), location);
            }

            for part in &remaining[..remaining.len() - 1] {
                match part {
                    DesignatorPart::Ident(field, _) => {
                        let idx = self.add_constant(Value::Str(field.clone()), location)?;
                        self.emit(Op::FieldGet(idx), location);
                    }
                    DesignatorPart::Index(expr, _) => {
                        self.compile_expr(expr)?;
                        self.emit(Op::IndexGet, location);
                    }
                }
            }

            let Some(last_part) = remaining.last() else {
                return Ok(());
            };

            match last_part {
                DesignatorPart::Ident(field, _) => {
                    self.compile_expr(value)?;
                    let idx = self.add_constant(Value::Str(field.clone()), location)?;
                    self.emit(Op::FieldSet(idx), location);
                }
                DesignatorPart::Index(expr, _) => {
                    self.compile_expr(expr)?;
                    self.compile_expr(value)?;
                    self.emit(Op::IndexSet, location);
                }
            }

            if let Some(local_ref) = self.resolve_local(&base_name) {
                self.emit_local_ref_update_finish(local_ref, location);
                self.emit(Op::Pop, location);
            } else {
                let idx = self.add_constant(Value::Str(base_name), location)?;
                self.emit(Op::SetGlobal(idx), location);
                self.emit(Op::Pop, location);
            }
        }

        Ok(())
    }
}
