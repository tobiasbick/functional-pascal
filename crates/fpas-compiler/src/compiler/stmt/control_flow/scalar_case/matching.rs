use super::super::super::super::Compiler;
use crate::error::CompileError;
use fpas_bytecode::{Op, SourceLocation, Value};
use fpas_parser::CaseLabel;
use fpas_sema::Ty;

impl Compiler {
    pub(super) fn scalar_case_compare_ops(case_ty: &Ty) -> (Op, Op, Op) {
        match case_ty {
            Ty::String => (Op::EqStr, Op::GeStr, Op::LeStr),
            Ty::Real => (Op::EqReal, Op::GeReal, Op::LeReal),
            Ty::Boolean => (Op::EqBool, Op::GeInt, Op::LeInt),
            _ => (Op::EqInt, Op::GeInt, Op::LeInt),
        }
    }

    pub(super) fn emit_case_label_match(
        &mut self,
        label: &CaseLabel,
        case_slot: u16,
        eq_op: Op,
        ge_op: Op,
        le_op: Op,
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        match label {
            CaseLabel::Value {
                start,
                end: Some(end_expr),
                ..
            } => {
                self.emit(Op::GetLocal(case_slot), location);
                self.compile_expr(start)?;
                self.emit(ge_op, location);

                self.emit(Op::GetLocal(case_slot), location);
                self.compile_expr(end_expr)?;
                self.emit(le_op, location);

                self.emit(Op::And, location);
            }
            CaseLabel::Value {
                start, end: None, ..
            } => {
                if self.is_scalar_guard_binding_expr(start) {
                    self.emit_constant(Value::Boolean(true), location)?;
                    return Ok(());
                }
                self.emit(Op::GetLocal(case_slot), location);
                self.compile_expr(start)?;
                self.emit(eq_op, location);
            }
            CaseLabel::Destructure { variant, .. } => {
                self.emit(Op::GetLocal(case_slot), location);
                match variant {
                    fpas_parser::DestructureVariant::Ok => {
                        self.emit(Op::IsResultOk, location);
                    }
                    fpas_parser::DestructureVariant::Error => {
                        self.emit(Op::IsResultOk, location);
                        self.emit(Op::Not, location);
                    }
                    fpas_parser::DestructureVariant::Some => {
                        self.emit(Op::IsOptionSome, location);
                    }
                    fpas_parser::DestructureVariant::None => {
                        self.emit(Op::IsOptionSome, location);
                        self.emit(Op::Not, location);
                    }
                }
            }
        }

        Ok(())
    }
}
