use super::super::super::Compiler;
use crate::error::CompileError;
use fpas_bytecode::Op;
use fpas_parser::{CaseArm, Stmt};
use fpas_sema::Ty;

impl Compiler {
    /// Compile `case` on scalar types (integer, real, string, boolean, simple enum).
    pub(super) fn compile_case_scalar(
        &mut self,
        arms: &[CaseArm],
        else_body: Option<&[Stmt]>,
        case_slot: u16,
        case_ty: &Ty,
        line: u32,
        column: u32,
    ) -> Result<(), CompileError> {
        let (eq_op, ge_op, le_op) = match case_ty {
            Ty::String => (Op::EqStr, Op::GeStr, Op::LeStr),
            Ty::Real => (Op::EqReal, Op::GeReal, Op::LeReal),
            Ty::Boolean => (Op::EqBool, Op::GeInt, Op::LeInt),
            _ => (Op::EqInt, Op::GeInt, Op::LeInt),
        };

        let mut end_patches = Vec::new();

        for arm in arms {
            let mut body_patches = Vec::new();
            let mut next_patches = Vec::new();

            for label in &arm.labels {
                match label {
                    fpas_parser::CaseLabel::Value {
                        start,
                        end: Some(end_expr),
                        ..
                    } => {
                        self.emit(Op::GetLocal(case_slot), (line, column));
                        self.compile_expr(start)?;
                        self.emit(ge_op, (line, column));

                        self.emit(Op::GetLocal(case_slot), (line, column));
                        self.compile_expr(end_expr)?;
                        self.emit(le_op, (line, column));

                        self.emit(Op::And, (line, column));
                    }
                    fpas_parser::CaseLabel::Value {
                        start, end: None, ..
                    } => {
                        self.emit(Op::GetLocal(case_slot), (line, column));
                        self.compile_expr(start)?;
                        self.emit(eq_op, (line, column));
                    }
                    fpas_parser::CaseLabel::Destructure {
                        variant, binding, ..
                    } => {
                        self.emit(Op::GetLocal(case_slot), (line, column));
                        match variant {
                            fpas_parser::DestructureVariant::Ok => {
                                self.emit(Op::IsResultOk, (line, column));
                            }
                            fpas_parser::DestructureVariant::Error => {
                                self.emit(Op::IsResultOk, (line, column));
                                self.emit(Op::Not, (line, column));
                            }
                            fpas_parser::DestructureVariant::Some => {
                                self.emit(Op::IsOptionSome, (line, column));
                            }
                            fpas_parser::DestructureVariant::None => {
                                self.emit(Op::IsOptionSome, (line, column));
                                self.emit(Op::Not, (line, column));
                            }
                        }
                        let _ = binding;
                    }
                }

                let jump = self.emit(Op::JumpIfTrue(0), (line, column));
                body_patches.push(jump);
            }

            let skip = self.emit(Op::Jump(0), (line, column));
            next_patches.push(skip);

            let body_addr = self.chunk.len() as u32;
            for patch in body_patches {
                self.patch_jump(patch, body_addr, (line, column))?;
            }

            let has_binding = arm.labels.iter().find_map(|l| {
                if let fpas_parser::CaseLabel::Destructure {
                    variant, binding, ..
                } = l
                {
                    binding.as_ref().map(|b| (*variant, b.clone()))
                } else {
                    None
                }
            });
            let opened_scope = has_binding.is_some();
            if let Some((variant, name)) = &has_binding {
                self.begin_scope();
                self.emit(Op::GetLocal(case_slot), (line, column));
                match variant {
                    fpas_parser::DestructureVariant::Ok | fpas_parser::DestructureVariant::Some => {
                        self.emit(Op::UnwrapOk, (line, column));
                    }
                    fpas_parser::DestructureVariant::Error => {
                        self.emit(Op::UnwrapErr, (line, column));
                    }
                    fpas_parser::DestructureVariant::None => {}
                }
                self.add_local(name);
            }

            let guard_fail = if let Some(guard_expr) = &arm.guard {
                self.compile_expr(guard_expr)?;
                Some(self.emit(Op::JumpIfFalse(0), (line, column)))
            } else {
                None
            };

            self.compile_stmt(&arm.body)?;

            if opened_scope {
                self.end_scope((line, column));
            }

            let end_jump = self.emit(Op::Jump(0), (line, column));
            end_patches.push(end_jump);

            if let Some(guard_patch) = guard_fail {
                let guard_fail_addr = self.chunk.len() as u32;
                self.patch_jump(guard_patch, guard_fail_addr, (line, column))?;
                if has_binding.is_some() {
                    self.emit(Op::Pop, (line, column));
                }
            }

            let next_addr = self.chunk.len() as u32;
            for patch in next_patches {
                self.patch_jump(patch, next_addr, (line, column))?;
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
}
