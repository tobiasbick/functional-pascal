use crate::error::{CompileError, compile_error, internal_compiler_error};
use fpas_bytecode::{Op, Value};
use fpas_diagnostics::codes::COMPILE_INVALID_DESIGNATOR_BASE;
use fpas_parser::{Designator, DesignatorPart};

use super::super::canonical_name;
use super::{Compiler, LocalRef};

impl Compiler {
    /// Compile a designator for reading (e.g. `Arr[0]`, `P.X`, `X`, `C.Add`).
    pub(in super::super) fn compile_designator_read(
        &mut self,
        d: &Designator,
    ) -> Result<(), CompileError> {
        let location = Self::location_of(&d.span);
        if self.try_emit_enum_constant(d, location)? {
            return Ok(());
        }

        let designator_key = fpas_sema::designator_lookup_key(d);
        if let Some(info) = self.bound_methods.get(&designator_key).cloned() {
            return self.compile_bound_method_designator(d, &info);
        }
        if let Some(infos) = self.property_reads.get(&designator_key).cloned() {
            return self.compile_property_read_designator(d, &infos);
        }

        self.compile_designator_prefix_read(d, d.parts.len())
    }

    /// Compile a leading portion of a designator without cloning contained expressions.
    pub(in crate::compiler) fn compile_designator_prefix_read(
        &mut self,
        d: &Designator,
        part_count: usize,
    ) -> Result<(), CompileError> {
        let location = Self::location_of(&d.span);
        let designator_parts = &d.parts[..part_count.min(d.parts.len())];

        let mut parts = designator_parts.iter();
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
            self.emit_local_ref_read(local_ref, location);

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
            if let Some((global_name, consumed)) = self.module_global_prefix(d)
                && consumed <= designator_parts.len()
            {
                let idx = self.add_constant(Value::Str(global_name), location)?;
                self.emit(Op::GetGlobal(idx), location);
                for part in designator_parts.iter().skip(consumed) {
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
                return Ok(());
            }

            let raw_name = designator_parts
                .iter()
                .filter_map(|part| match part {
                    DesignatorPart::Ident(name, _) => Some(name.as_str()),
                    DesignatorPart::Index(..) => None,
                })
                .collect::<Vec<_>>()
                .join(".");
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
                    LocalRef::Local(slot) => self.emit_local_read(slot, location),
                    LocalRef::Enclosing(depth, slot) => {
                        self.emit(Op::GetEnclosing(depth, slot), location);
                    }
                };
                return Ok(());
            }

            // If the name resolves to a known function, emit a function reference value.
            let canonical_function_name = canonical_name(&name);
            if self
                .chunk
                .functions()
                .contains_key(&canonical_function_name)
                || self.external_callables.contains(&canonical_function_name)
            {
                if self.emit_captured_routine_closure(&canonical_function_name, location)? {
                    return Ok(());
                }
                self.emit_constant(
                    Value::Function {
                        name: canonical_function_name,
                        captures: vec![],
                        task_bound: false,
                    },
                    location,
                )?;
                return Ok(());
            }

            let remaining: Vec<_> = designator_parts.iter().skip(1).collect();
            if remaining.is_empty() {
                let idx = self.add_constant(Value::Str(name), location)?;
                self.emit(Op::GetGlobal(idx), location);
            } else {
                let idx = self.add_constant(Value::Str(canonical_name(&base_name)), location)?;
                self.emit(Op::GetGlobal(idx), location);
                for part in remaining {
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
            }
        }

        Ok(())
    }

    fn compile_bound_method_designator(
        &mut self,
        d: &Designator,
        info: &fpas_sema::BoundMethodInfo,
    ) -> Result<(), CompileError> {
        let location = Self::location_of(&d.span);
        let receiver_part_count = info.receiver_part_count;
        d.parts.get(..receiver_part_count).ok_or_else(|| {
            internal_compiler_error(
                "Bound-method receiver metadata exceeds the designator path.",
                "Re-run compilation and report this internal compiler error.",
                d.span.line,
                d.span.column,
            )
        })?;
        self.compile_designator_prefix_read(d, receiver_part_count)?;

        self.emit_bound_method_from_receiver(info, location)
    }
}
