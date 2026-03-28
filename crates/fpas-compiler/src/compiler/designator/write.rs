use crate::error::{CompileError, compile_error};
use fpas_bytecode::{Op, SourceLocation, Value};
use fpas_diagnostics::codes::COMPILE_INVALID_ASSIGNMENT_TARGET;
use fpas_parser::{Designator, DesignatorPart, Expr};

use super::{Compiler, LocalRef};

impl Compiler {
    /// Compile a designator assignment (e.g. `X := val`, `Arr[i] := val`, `P.X := val`).
    pub(in super::super) fn compile_designator_write(
        &mut self,
        target: &Designator,
        value: &Expr,
        location: SourceLocation,
    ) -> Result<(), CompileError> {
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

        if remaining.is_empty() {
            self.compile_expr(value)?;
            if let Some(local_ref) = self.resolve_local(&base_name) {
                match local_ref {
                    LocalRef::Local(slot) => self.emit(Op::SetLocal(slot), location),
                    LocalRef::Enclosing(depth, slot) => {
                        self.emit(Op::SetEnclosing(depth, slot), location)
                    }
                };
                self.emit(Op::Pop, location);
            } else {
                let idx = self.chunk.add_constant(Value::Str(base_name));
                self.emit(Op::SetGlobal(idx), location);
                self.emit(Op::Pop, location);
            }
        } else {
            if let Some(local_ref) = self.resolve_local(&base_name) {
                match local_ref {
                    LocalRef::Local(slot) => self.emit(Op::GetLocal(slot), location),
                    LocalRef::Enclosing(depth, slot) => {
                        self.emit(Op::GetEnclosing(depth, slot), location)
                    }
                };
            } else {
                let idx = self.chunk.add_constant(Value::Str(base_name.clone()));
                self.emit(Op::GetGlobal(idx), location);
            }

            for part in &remaining[..remaining.len() - 1] {
                match part {
                    DesignatorPart::Ident(field, _) => {
                        let idx = self.chunk.add_constant(Value::Str(field.clone()));
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
                    let idx = self.chunk.add_constant(Value::Str(field.clone()));
                    self.emit(Op::FieldSet(idx), location);
                }
                DesignatorPart::Index(expr, _) => {
                    self.compile_expr(expr)?;
                    self.compile_expr(value)?;
                    self.emit(Op::IndexSet, location);
                }
            }

            if let Some(local_ref) = self.resolve_local(&base_name) {
                match local_ref {
                    LocalRef::Local(slot) => self.emit(Op::SetLocal(slot), location),
                    LocalRef::Enclosing(depth, slot) => {
                        self.emit(Op::SetEnclosing(depth, slot), location)
                    }
                };
                self.emit(Op::Pop, location);
            } else {
                let idx = self.chunk.add_constant(Value::Str(base_name));
                self.emit(Op::SetGlobal(idx), location);
                self.emit(Op::Pop, location);
            }
        }

        Ok(())
    }
}
