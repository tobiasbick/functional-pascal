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
        if let Some((global_name, consumed)) = self.module_global_prefix(target) {
            if consumed == target.parts.len() {
                self.compile_expr(value)?;
                let idx = self.add_constant(Value::Str(global_name.into()), location)?;
                self.emit(Op::SetGlobal(idx), location);
                self.emit(Op::Pop, location);
                return Ok(());
            }

            let suffix = &target.parts[consumed..];
            if suffix
                .iter()
                .all(|part| matches!(part, DesignatorPart::Index(_, _)))
            {
                let index_count =
                    Self::checked_u8_at(suffix.len(), "global index operations", location)?;
                for part in suffix {
                    if let DesignatorPart::Index(expr, _) = part {
                        self.compile_expr(expr)?;
                    }
                }
                self.compile_expr(value)?;
                let idx = self.add_constant(Value::Str(global_name.into()), location)?;
                self.emit(Op::GlobalIndexSet(idx, index_count), location);
                self.emit(Op::Pop, location);
                return Ok(());
            }
            let idx = self.add_constant(Value::Str(global_name.clone().into()), location)?;
            self.emit(Op::GetGlobal(idx), location);
            for part in &suffix[..suffix.len() - 1] {
                match part {
                    DesignatorPart::Ident(field, _) => {
                        let idx = self.add_constant(Value::Str(field.clone().into()), location)?;
                        self.emit(Op::FieldGet(idx), location);
                    }
                    DesignatorPart::Index(expr, _) => {
                        self.compile_expr(expr)?;
                        self.emit(Op::IndexGet, location);
                    }
                }
            }

            match suffix.last() {
                Some(DesignatorPart::Ident(field, _)) => {
                    self.compile_expr(value)?;
                    let idx = self.add_constant(Value::Str(field.clone().into()), location)?;
                    self.emit(Op::FieldSet(idx), location);
                }
                Some(DesignatorPart::Index(expr, _)) => {
                    self.compile_expr(expr)?;
                    self.compile_expr(value)?;
                    self.emit(Op::IndexSet, location);
                }
                None => return Ok(()),
            }

            let idx = self.add_constant(Value::Str(global_name.into()), location)?;
            self.emit(Op::SetGlobal(idx), location);
            self.emit(Op::Pop, location);
            return Ok(());
        }

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
            if let Some(local_ref) = self
                .resolve_local(&base_name)
                .or_else(|| self.resolve_local(&qualified))
            {
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
                let idx = self.add_constant(Value::Str(qualified.into()), location)?;
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
                let idx = self.add_constant(Value::Str(base_name.into()), location)?;
                self.emit(Op::SetGlobal(idx), location);
                self.emit(Op::Pop, location);
            }
        } else {
            if let Some(local_ref) = self.resolve_local(&base_name) {
                self.emit_local_ref_update_start(local_ref, location);
            } else {
                let idx = self.add_constant(Value::Str(base_name.clone().into()), location)?;
                self.emit(Op::GetGlobal(idx), location);
            }

            for part in &remaining[..remaining.len() - 1] {
                match part {
                    DesignatorPart::Ident(field, _) => {
                        let idx = self.add_constant(Value::Str(field.clone().into()), location)?;
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
                    let idx = self.add_constant(Value::Str(field.clone().into()), location)?;
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
                let idx = self.add_constant(Value::Str(base_name.into()), location)?;
                self.emit(Op::SetGlobal(idx), location);
                self.emit(Op::Pop, location);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fpas_bytecode::{Op, SourceLocation};
    use fpas_diagnostics::codes::COMPILE_BYTECODE_OPERAND_OVERFLOW;
    use fpas_lexer::Span;
    use fpas_parser::{Designator, DesignatorPart, Expr};
    use fpas_sema::AnalysisMetadata;

    use super::Compiler;

    fn span() -> Span {
        Span {
            offset: 0,
            length: 0,
            line: 1,
            column: 1,
            source_id: 0,
        }
    }

    fn indexed_global(index_count: usize) -> Designator {
        let span = span();
        let mut parts = Vec::with_capacity(index_count + 1);
        parts.push(DesignatorPart::Ident("Values".to_string(), span));
        parts.extend(
            std::iter::repeat_with(|| DesignatorPart::Index(Expr::Integer(0, span), span))
                .take(index_count),
        );
        Designator { parts, span }
    }

    fn compiler_with_global() -> Compiler {
        let mut compiler = Compiler::new(AnalysisMetadata::default());
        compiler.module_globals.insert("values".to_string());
        compiler
    }

    #[test]
    fn global_index_write_accepts_u8_max_indices() {
        let mut compiler = compiler_with_global();
        let target = indexed_global(usize::from(u8::MAX));

        compiler
            .compile_designator_write(
                &target,
                &Expr::Integer(1, span()),
                SourceLocation::new(1, 1),
            )
            .expect("u8::MAX indices should compile");

        assert!(
            compiler
                .finish()
                .code()
                .iter()
                .any(|op| matches!(op, Op::GlobalIndexSet(_, count) if *count == u8::MAX))
        );
    }

    #[test]
    fn global_index_write_rejects_more_than_u8_max_indices() {
        let mut compiler = compiler_with_global();
        let target = indexed_global(usize::from(u8::MAX) + 1);

        let error = compiler
            .compile_designator_write(
                &target,
                &Expr::Integer(1, span()),
                SourceLocation::new(1, 1),
            )
            .expect_err("an index count wider than the bytecode operand must fail");

        assert_eq!(error.code, COMPILE_BYTECODE_OPERAND_OVERFLOW);
    }
}
