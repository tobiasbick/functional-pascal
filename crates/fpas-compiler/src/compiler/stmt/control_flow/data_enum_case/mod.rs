use super::super::super::Compiler;
use crate::error::{CompileError, compile_error};
use fpas_bytecode::{Op, Value};
use fpas_diagnostics::codes::SEMA_ENUM_FIELD_COUNT_MISMATCH;
use fpas_parser::{CaseArm, CaseLabel, Expr, Stmt};

mod pattern;

use pattern::DataEnumPattern;

impl Compiler {
    /// Compile `case` on an enum with associated data.
    ///
    /// Labels are parsed as `Expr::Call` or `Expr::Designator`. The variant name
    /// is extracted from the designator and `IsVariant` + `EnumField` ops are
    /// emitted for matching and binding. Nested patterns are supported.
    ///
    /// **Documentation:** `docs/pascal/06-pattern-matching.md`
    pub(super) fn compile_case_data_enum(
        &mut self,
        arms: &[CaseArm],
        else_body: Option<&[Stmt]>,
        case_slot: u16,
        enum_type_name: &str,
        line: u32,
        column: u32,
    ) -> Result<(), CompileError> {
        let mut end_patches = Vec::new();

        for arm in arms {
            for label in &arm.labels {
                let pattern = DataEnumPattern::analyze(label);

                self.validate_variant_field_counts(enum_type_name, &pattern)?;
                let mut fail_patches = Vec::new();

                self.emit_primary_data_enum_check(
                    label,
                    pattern.root_variant_name.as_deref(),
                    case_slot,
                    (line, column),
                    &mut fail_patches,
                )?;
                self.emit_nested_variant_checks(
                    case_slot,
                    &pattern.nested_variant_checks,
                    (line, column),
                    &mut fail_patches,
                );
                self.emit_value_checks(
                    case_slot,
                    &pattern.value_checks,
                    (line, column),
                    &mut fail_patches,
                )?;

                let binding_count = pattern.bindings.len();
                if binding_count > 0 {
                    self.begin_scope();
                    for (field_path, binding_name) in &pattern.bindings {
                        self.emit_case_value_path(case_slot, field_path, (line, column));
                        self.add_local(binding_name);
                    }
                }

                let guard_fail = if let Some(guard_expr) = &arm.guard {
                    self.compile_expr(guard_expr)?;
                    Some(self.emit(Op::JumpIfFalse(0), (line, column)))
                } else {
                    None
                };

                self.compile_stmt(&arm.body)?;

                if binding_count > 0 {
                    self.end_scope((line, column));
                }

                end_patches.push(self.emit(Op::Jump(0), (line, column)));

                if let Some(guard_patch) = guard_fail {
                    let guard_fail_addr = self.chunk.len() as u32;
                    self.patch_jump(guard_patch, guard_fail_addr, (line, column))?;
                    for _ in 0..binding_count {
                        self.emit(Op::Pop, (line, column));
                    }
                }

                let next_label_addr = self.chunk.len() as u32;
                for patch in fail_patches {
                    self.patch_jump(patch, next_label_addr, (line, column))?;
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
            self.patch_jump(patch, end_addr, (line, column))?;
        }

        Ok(())
    }

    fn emit_primary_data_enum_check(
        &mut self,
        label: &CaseLabel,
        root_variant_name: Option<&str>,
        case_slot: u16,
        location: (u32, u32),
        fail_patches: &mut Vec<usize>,
    ) -> Result<(), CompileError> {
        self.emit(Op::GetLocal(case_slot), location);
        if let Some(variant_name) = root_variant_name {
            let variant_idx = self.chunk.add_constant(Value::Str(variant_name.into()));
            self.emit(Op::IsVariant(variant_idx), location);
            fail_patches.push(self.emit(Op::JumpIfFalse(0), location));
            return Ok(());
        }

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

    fn emit_nested_variant_checks(
        &mut self,
        case_slot: u16,
        checks: &[(Vec<u8>, String)],
        location: (u32, u32),
        fail_patches: &mut Vec<usize>,
    ) {
        for (field_path, variant_name) in checks {
            self.emit_case_value_path(case_slot, field_path, location);
            let variant_idx = self.chunk.add_constant(Value::Str(variant_name.clone()));
            self.emit(Op::IsVariant(variant_idx), location);
            fail_patches.push(self.emit(Op::JumpIfFalse(0), location));
        }
    }

    fn emit_value_checks(
        &mut self,
        case_slot: u16,
        value_checks: &[(Vec<u8>, &Expr)],
        location: (u32, u32),
        fail_patches: &mut Vec<usize>,
    ) -> Result<(), CompileError> {
        for (field_path, value_expr) in value_checks {
            self.emit_case_value_path(case_slot, field_path, location);
            self.compile_expr(value_expr)?;
            self.emit(Self::pattern_equality_op(value_expr), location);
            fail_patches.push(self.emit(Op::JumpIfFalse(0), location));
        }
        Ok(())
    }

    fn emit_case_value_path(&mut self, case_slot: u16, field_path: &[u8], location: (u32, u32)) {
        self.emit(Op::GetLocal(case_slot), location);
        for &field_idx in field_path {
            self.emit(Op::EnumField(field_idx), location);
        }
    }

    fn pattern_equality_op(value_expr: &Expr) -> Op {
        match value_expr {
            Expr::Str(..) => Op::EqStr,
            Expr::Real(..) => Op::EqReal,
            Expr::Bool(..) => Op::EqBool,
            _ => Op::EqInt,
        }
    }

    /// Check that every variant call in the pattern supplies the correct number
    /// of arguments.  Nested patterns may reference variants from other enum
    /// types, so the lookup walks all registered enums.
    fn validate_variant_field_counts(
        &self,
        primary_enum: &str,
        pattern: &DataEnumPattern<'_>,
    ) -> Result<(), CompileError> {
        for call in &pattern.variant_calls {
            let expected = self
                .find_variant_field_count(primary_enum, &call.variant_name)
                .or_else(|| self.find_variant_field_count_any(&call.variant_name));

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
        }
        Ok(())
    }

    /// Look up the field count for a variant in a specific enum type.
    fn find_variant_field_count(&self, enum_name: &str, variant_name: &str) -> Option<usize> {
        self.enums.get(enum_name).and_then(|info| {
            info.variants
                .iter()
                .find(|v| v.name == variant_name)
                .map(|v| v.field_names.len())
        })
    }

    /// Search all registered enums for a variant (used for nested patterns that
    /// destructure a different enum type).
    fn find_variant_field_count_any(&self, variant_name: &str) -> Option<usize> {
        self.enums.values().find_map(|info| {
            info.variants
                .iter()
                .find(|v| v.name == variant_name)
                .map(|v| v.field_names.len())
        })
    }
}
