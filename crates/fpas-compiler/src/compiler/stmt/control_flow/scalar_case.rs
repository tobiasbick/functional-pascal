use super::super::super::Compiler;
use crate::error::CompileError;
use fpas_bytecode::{Op, Value};
use fpas_parser::{CaseArm, CaseLabel, DesignatorPart, Expr, Stmt};
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
            for label in &arm.labels {
                self.emit_case_label_match(label, case_slot, eq_op, ge_op, le_op, line, column)?;
                let fail_patch = self.emit(Op::JumpIfFalse(0), (line, column));

                let scalar_binding = self.scalar_guard_binding_name(label);
                let binding = match label {
                    fpas_parser::CaseLabel::Destructure {
                        variant, binding, ..
                    } => binding.as_ref().map(|name| (*variant, name.clone())),
                    fpas_parser::CaseLabel::Value { .. } => None,
                };

                if let Some(name) = &scalar_binding {
                    self.begin_scope();
                    self.emit(Op::GetLocal(case_slot), (line, column));
                    self.add_local(name);
                } else if let Some((variant, name)) = &binding {
                    self.begin_scope();
                    self.emit(Op::GetLocal(case_slot), (line, column));
                    match variant {
                        fpas_parser::DestructureVariant::Ok
                        | fpas_parser::DestructureVariant::Some => {
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

                if scalar_binding.is_some() || binding.is_some() {
                    self.end_scope((line, column));
                }

                end_patches.push(self.emit(Op::Jump(0), (line, column)));

                if let Some(guard_patch) = guard_fail {
                    let cleanup_addr = self.chunk.len() as u32;
                    self.patch_jump(guard_patch, cleanup_addr, (line, column))?;
                    if scalar_binding.is_some() || binding.is_some() {
                        self.emit(Op::Pop, (line, column));
                    }
                }

                let next_label_addr = self.chunk.len() as u32;
                self.patch_jump(fail_patch, next_label_addr, (line, column))?;
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

    fn emit_case_label_match(
        &mut self,
        label: &fpas_parser::CaseLabel,
        case_slot: u16,
        eq_op: Op,
        ge_op: Op,
        le_op: Op,
        line: u32,
        column: u32,
    ) -> Result<(), CompileError> {
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
                if self.is_scalar_guard_binding_expr(start) {
                    self.emit_constant(Value::Boolean(true), (line, column))?;
                    return Ok(());
                }
                self.emit(Op::GetLocal(case_slot), (line, column));
                self.compile_expr(start)?;
                self.emit(eq_op, (line, column));
            }
            fpas_parser::CaseLabel::Destructure { variant, .. } => {
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
            }
        }

        Ok(())
    }

    fn scalar_guard_binding_name(&self, label: &CaseLabel) -> Option<String> {
        let CaseLabel::Value {
            start, end: None, ..
        } = label
        else {
            return None;
        };
        if !self.is_scalar_guard_binding_expr(start) {
            return None;
        }

        let Expr::Designator(designator) = start else {
            return None;
        };
        let DesignatorPart::Ident(name, _) = &designator.parts[0] else {
            return None;
        };
        Some(name.clone())
    }

    fn is_scalar_guard_binding_expr(&self, expr: &Expr) -> bool {
        self.scalar_case_bindings
            .contains(&fpas_sema::expr_lookup_key(expr))
    }
}
