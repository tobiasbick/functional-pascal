use super::super::super::Compiler;
use crate::error::{CompileError, compile_error};
use fpas_bytecode::{Op, SourceLocation, Value};
use fpas_diagnostics::codes::SEMA_ENUM_FIELD_COUNT_MISMATCH;
use fpas_parser::{CaseArm, CaseLabel, Stmt};

mod pattern;

use pattern::DataEnumPattern;

impl Compiler {
    /// Compile `case` on an enum with associated data.
    ///
    /// Labels are parsed as `Expr::Call` or `Expr::Designator`. The variant
    /// name is extracted and `IsVariant` + `EnumField` ops are emitted for
    /// matching and binding. Only single-level destructuring is supported.
    ///
    /// **Documentation:** `docs/pascal/06-pattern-matching.md`
    pub(super) fn compile_case_data_enum(
        &mut self,
        arms: &[CaseArm],
        else_body: Option<&[Stmt]>,
        case_slot: u16,
        enum_type_name: &str,
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        let mut end_patches = Vec::new();

        for arm in arms {
            for label in &arm.labels {
                let pattern = DataEnumPattern::analyze(label)?;

                self.validate_variant_field_count(enum_type_name, &pattern)?;
                let mut fail_patches = Vec::new();

                self.emit_variant_check(
                    label,
                    pattern.root_variant_name.as_deref(),
                    case_slot,
                    location,
                    &mut fail_patches,
                )?;

                let binding_count = pattern.bindings.len();
                if binding_count > 0 {
                    self.begin_scope();
                    for (field_idx, binding_name) in &pattern.bindings {
                        self.emit(Op::GetLocal(case_slot), location);
                        self.emit(Op::EnumField(*field_idx), location);
                        self.add_local(binding_name);
                    }
                }

                let guard_fail = if let Some(guard_expr) = &arm.guard {
                    self.compile_expr(guard_expr)?;
                    Some(self.emit(Op::JumpIfFalse(0), location))
                } else {
                    None
                };

                self.compile_stmt(&arm.body)?;

                if binding_count > 0 {
                    self.end_scope(location);
                }

                end_patches.push(self.emit(Op::Jump(0), location));

                if let Some(guard_patch) = guard_fail {
                    let guard_fail_addr = self.chunk.len() as u32;
                    self.patch_jump(guard_patch, guard_fail_addr, location)?;
                    for _ in 0..binding_count {
                        self.emit(Op::Pop, location);
                    }
                }

                let next_label_addr = self.chunk.len() as u32;
                for patch in fail_patches {
                    self.patch_jump(patch, next_label_addr, location)?;
                }
            }
        }

        if let Some(stmts) = else_body {
            for stmt in stmts {
                self.compile_stmt(stmt)?;
            }
        }

        let end_addr = self.chunk.len() as u32;
        for patch in end_patches {
            self.patch_jump(patch, end_addr, location)?;
        }

        Ok(())
    }

    /// Emit the `IsVariant` check for the root variant of a pattern.
    fn emit_variant_check(
        &mut self,
        label: &CaseLabel,
        root_variant_name: Option<&str>,
        case_slot: u16,
        location: SourceLocation,
        fail_patches: &mut Vec<usize>,
    ) -> Result<(), CompileError> {
        self.emit(Op::GetLocal(case_slot), location);
        if let Some(variant_name) = root_variant_name {
            let variant_idx = self.add_constant(Value::Str(variant_name.into()), location)?;
            self.emit(Op::IsVariant(variant_idx), location);
            fail_patches.push(self.emit(Op::JumpIfFalse(0), location));
            return Ok(());
        }

        // Fallback: scalar value comparison (fieldless variant reached via value expr)
        if let CaseLabel::Value {
            start, end: None, ..
        } = label
        {
            self.compile_expr(start)?;
            self.emit(Op::EqInt, location);
            fail_patches.push(self.emit(Op::JumpIfFalse(0), location));
        }
        Ok(())
    }

    /// Check that the pattern supplies the correct number of arguments for its
    /// variant.  Reports a [`CompileError`] on mismatch.
    fn validate_variant_field_count(
        &self,
        enum_name: &str,
        pattern: &DataEnumPattern,
    ) -> Result<(), CompileError> {
        let Some(call) = &pattern.variant_call else {
            return Ok(());
        };

        let expected = self.find_variant_field_count(enum_name, &call.variant_name);

        if let Some(expected) = expected
            && call.arg_count != expected
        {
            return Err(compile_error(
                SEMA_ENUM_FIELD_COUNT_MISMATCH,
                format!(
                    "Variant '{}' expects {} field{}, but {} {} supplied.",
                    call.variant_name,
                    expected,
                    if expected == 1 { "" } else { "s" },
                    call.arg_count,
                    if call.arg_count == 1 { "was" } else { "were" },
                ),
                format!(
                    "Use {} binding{} to match all fields of '{}'.",
                    expected,
                    if expected == 1 { "" } else { "s" },
                    call.variant_name,
                ),
                call.span,
            ));
        }
        Ok(())
    }

    /// Look up the field count for a variant in the given enum type.
    fn find_variant_field_count(&self, enum_name: &str, variant_name: &str) -> Option<usize> {
        self.enums.get(enum_name).and_then(|info| {
            info.variants
                .iter()
                .find(|v| v.name == variant_name)
                .map(|v| v.field_names.len())
        })
    }
}
