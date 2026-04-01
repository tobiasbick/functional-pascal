use crate::error::{CompileError, compile_error};
use fpas_bytecode::{Op, Value};
use fpas_diagnostics::codes::COMPILE_INVALID_DESIGNATOR_BASE;
use fpas_parser::{Designator, DesignatorPart};

use super::{Compiler, LocalRef};

impl Compiler {
    /// Compile a designator for reading (e.g. `Arr[0]`, `P.X`, `X`).
    pub(in super::super) fn compile_designator_read(
        &mut self,
        d: &Designator,
    ) -> Result<(), CompileError> {
        let location = Self::location_of(&d.span);
        if self.try_emit_enum_constant(d, location)? {
            return Ok(());
        }

        let mut parts = d.parts.iter();
        let base_name = match parts.next() {
            Some(DesignatorPart::Ident(name, _)) => name.clone(),
            _ => {
                return Err(compile_error(
                    COMPILE_INVALID_DESIGNATOR_BASE,
                    "Expected identifier",
                    "Designator must start with a variable or constant name.",
                    d.span,
                ));
            }
        };

        if let Some(local_ref) = self.resolve_local(&base_name) {
            match local_ref {
                LocalRef::Local(slot) => self.emit(Op::GetLocal(slot), location),
                LocalRef::Enclosing(depth, slot) => {
                    self.emit(Op::GetEnclosing(depth, slot), location)
                }
            };

            for part in parts {
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
        } else {
            let raw_name = Self::resolve_designator_name(d);
            let name = self.qualify_name(&raw_name).to_string();
            if let Some(value) = Self::builtin_const_value(&name) {
                self.emit_constant(value, location)?;
                return Ok(());
            }

            // Qualified names from linked units (e.g. `App.Config.MaxSize`) are registered
            // as locals under their full dotted name. Try the joined name before falling
            // through to GetGlobal.
            if let Some(local_ref) = self.resolve_local(&name) {
                match local_ref {
                    LocalRef::Local(slot) => self.emit(Op::GetLocal(slot), location),
                    LocalRef::Enclosing(depth, slot) => {
                        self.emit(Op::GetEnclosing(depth, slot), location)
                    }
                };
                return Ok(());
            }

            // If the name resolves to a known function, emit a function reference value.
            if let Some((_code_start, _arity)) = self.chunk.functions.get(&name) {
                self.emit_constant(
                    Value::Function {
                        name: name.clone(),
                        captures: vec![],
                    },
                    location,
                )?;
                return Ok(());
            }

            let idx = self.add_constant(Value::Str(name), location)?;
            self.emit(Op::GetGlobal(idx), location);
            for part in &d.parts {
                if let DesignatorPart::Index(expr, _) = part {
                    self.compile_expr(expr)?;
                    self.emit(Op::IndexGet, location);
                }
            }
        }

        Ok(())
    }
}
