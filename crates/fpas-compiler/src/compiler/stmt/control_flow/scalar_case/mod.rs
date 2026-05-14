use super::super::super::Compiler;
use crate::error::CompileError;
use fpas_bytecode::{Op, SourceLocation};
use fpas_parser::{CaseArm, Stmt};
use fpas_sema::Ty;

mod bindings;
mod matching;

impl Compiler {
    /// Compile `case` on scalar types (integer, real, string, boolean, simple enum).
    pub(super) fn compile_case_scalar(
        &mut self,
        arms: &[CaseArm],
        else_body: Option<&[Stmt]>,
        case_slot: u16,
        case_ty: &Ty,
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        let (eq_op, ge_op, le_op) = Self::scalar_case_compare_ops(case_ty);
        let mut end_patches = Vec::new();

        for arm in arms {
            for label in &arm.labels {
                self.emit_case_label_match(label, case_slot, eq_op, ge_op, le_op, location)?;
                let fail_patch = self.emit(Op::JumpIfFalse(0), location);

                let scalar_binding = self.scalar_guard_binding_name(label);
                let binding = match label {
                    fpas_parser::CaseLabel::Destructure {
                        variant, binding, ..
                    } => binding.as_ref().map(|name| (*variant, name.clone())),
                    fpas_parser::CaseLabel::Value { .. } => None,
                };

                if let Some(name) = &scalar_binding {
                    self.begin_scope();
                    self.emit(Op::GetLocal(case_slot), location);
                    self.add_local(name);
                } else if let Some((variant, name)) = &binding {
                    self.begin_scope();
                    self.emit(Op::GetLocal(case_slot), location);
                    match variant {
                        fpas_parser::DestructureVariant::Ok
                        | fpas_parser::DestructureVariant::Some => {
                            self.emit(Op::UnwrapOk, location);
                        }
                        fpas_parser::DestructureVariant::Error => {
                            self.emit(Op::UnwrapErr, location);
                        }
                        fpas_parser::DestructureVariant::None => {}
                    }
                    self.add_local(name);
                }

                let guard_fail = if let Some(guard_expr) = &arm.guard {
                    self.compile_expr(guard_expr)?;
                    Some(self.emit(Op::JumpIfFalse(0), location))
                } else {
                    None
                };

                self.compile_stmt(&arm.body)?;

                if scalar_binding.is_some() || binding.is_some() {
                    self.end_scope(location);
                }

                end_patches.push(self.emit(Op::Jump(0), location));

                if let Some(guard_patch) = guard_fail {
                    let cleanup_addr = self.chunk.len() as u32;
                    self.patch_jump(guard_patch, cleanup_addr, location)?;
                    if scalar_binding.is_some() || binding.is_some() {
                        self.emit(Op::Pop, location);
                    }
                }

                let next_label_addr = self.chunk.len() as u32;
                self.patch_jump(fail_patch, next_label_addr, location)?;
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
}
