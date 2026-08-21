//! Scalar case lowering with ordered labels, ranges, guards, and bindings.

use fpas_ir::{BinaryOperation, Constant, Operation, Terminator};
use fpas_parser::{CaseArm, CaseLabel, Expr, Stmt};
use fpas_sema::Ty;

use crate::CompileError;

use super::super::context::{LoweringContext, target, unsupported};
use super::super::types;

impl LoweringContext {
    pub(in crate::lowering) fn lower_case(
        &mut self,
        expression: &Expr,
        arms: &[CaseArm],
        else_body: Option<&[Stmt]>,
        span: fpas_lexer::Span,
    ) -> Result<(), CompileError> {
        let mut case_ir_ty = self.expression_ir_type(expression)?;
        let case_ty = self.expression_type(expression).ok();
        if matches!(&case_ty, Some(Ty::Enum(enumeration)) if !enumeration.has_data()) {
            case_ir_ty = types::INTEGER;
        }
        let exhaustive_enum = matches!(&case_ty, Some(Ty::Enum(_)))
            || arms.iter().flat_map(|arm| &arm.labels).any(|label| {
                let CaseLabel::Value { start, .. } = label else {
                    return false;
                };
                matches!(self.expression_type(start), Ok(Ty::Enum(_)))
            });
        let case_value = self.lower_expression(expression)?;
        if matches!(case_ty, Some(Ty::Result(..) | Ty::Option(..)))
            || matches!(&case_ty, Some(Ty::Enum(enumeration)) if enumeration.has_data())
            || matches!(self.type_kind(case_ir_ty), Some(fpas_ir::IrType::Enum(_)))
        {
            return self.lower_variant_case(case_value, case_ir_ty, arms, else_body, span);
        }
        let case_ty = case_ty.ok_or_else(|| unsupported(span, "scalar case type"))?;
        self.begin_scope();
        let case_local = self.declare_hidden_local(case_ir_ty, span)?;
        self.write_local(case_local, case_value, span)?;
        let merge_block = self.new_block(span)?;
        let first_test = self.new_block(span)?;
        self.jump(first_test)?;
        self.switch_to(first_test);
        let mut has_merge_predecessor = false;

        for arm in arms {
            for label in &arm.labels {
                let next_test = self.new_block(arm.span)?;
                let body_block = self.new_block(arm.span)?;
                let is_binding = self.is_scalar_binding(label);
                let matched = self
                    .lower_case_match(label, case_local, case_ir_ty, &case_ty, is_binding, span)?;
                self.terminate(Terminator::Branch {
                    condition: matched,
                    then_target: target(body_block),
                    else_target: target(next_test),
                })?;

                self.switch_to(body_block);
                if is_binding {
                    self.begin_scope();
                    let name =
                        binding_name(label).ok_or_else(|| unsupported(arm.span, "case binding"))?;
                    let value =
                        self.emit_value(Operation::ReadLocal(case_local), case_ir_ty, span)?;
                    let local = self.declare_local(name, case_ir_ty, false, arm.span)?;
                    self.write_local(local, value, arm.span)?;
                }
                if let Some(guard) = &arm.guard {
                    let guard_value = self.lower_expression(guard)?;
                    let guarded_body = self.new_block(arm.span)?;
                    self.terminate(Terminator::Branch {
                        condition: guard_value,
                        then_target: target(guarded_body),
                        else_target: target(next_test),
                    })?;
                    self.switch_to(guarded_body);
                }
                self.lower_statement(&arm.body)?;
                if !self.is_terminated() {
                    self.jump(merge_block)?;
                    has_merge_predecessor = true;
                }
                if is_binding {
                    self.end_scope();
                }
                self.switch_to(next_test);
            }
        }

        if let Some(else_body) = else_body {
            self.lower_statements(else_body)?;
        } else if exhaustive_enum && !self.is_terminated() {
            let message = self.emit_value(
                Operation::Const(Constant::String(
                    "exhaustive case reached no enum member".to_string(),
                )),
                types::STRING,
                span,
            )?;
            self.terminate(Terminator::Panic(message))?;
        }
        if !self.is_terminated() {
            self.jump(merge_block)?;
            has_merge_predecessor = true;
        }
        self.end_scope();
        if has_merge_predecessor {
            self.switch_to(merge_block);
        }
        Ok(())
    }

    fn lower_case_match(
        &mut self,
        label: &CaseLabel,
        case_local: fpas_ir::LocalId,
        case_ir_ty: fpas_ir::TypeId,
        case_ty: &Ty,
        binding: bool,
        span: fpas_lexer::Span,
    ) -> Result<fpas_ir::ValueId, CompileError> {
        if binding {
            return self.emit_value(
                Operation::Const(Constant::Boolean(true)),
                types::BOOLEAN,
                span,
            );
        }
        let CaseLabel::Value { start, end, .. } = label else {
            return Err(unsupported(span, "destructuring case label"));
        };
        let left = self.emit_value(Operation::ReadLocal(case_local), case_ir_ty, span)?;
        let start_value = self.lower_expression(start)?;
        if let Some(end) = end {
            let ge = self.case_ordering(case_ty, false);
            let lower = self.emit_binary(ge, left, start_value, types::BOOLEAN, span)?;
            let upper_left = self.emit_value(Operation::ReadLocal(case_local), case_ir_ty, span)?;
            let end_value = self.lower_expression(end)?;
            let le = self.case_ordering(case_ty, true);
            let upper = self.emit_binary(le, upper_left, end_value, types::BOOLEAN, span)?;
            self.emit_binary(
                BinaryOperation::AndBoolean,
                lower,
                upper,
                types::BOOLEAN,
                span,
            )
        } else {
            self.emit_binary(
                BinaryOperation::Equal,
                left,
                start_value,
                types::BOOLEAN,
                span,
            )
        }
    }

    fn case_ordering(&self, ty: &Ty, upper: bool) -> BinaryOperation {
        match (ty, upper) {
            (Ty::Integer, false) => BinaryOperation::GreaterEqualInteger,
            (Ty::Integer, true) => BinaryOperation::LessEqualInteger,
            (Ty::Real, false) => BinaryOperation::GreaterEqualReal,
            (Ty::Real, true) => BinaryOperation::LessEqualReal,
            (_, false) => BinaryOperation::GreaterEqualDynamic,
            (_, true) => BinaryOperation::LessEqualDynamic,
        }
    }

    fn is_scalar_binding(&self, label: &CaseLabel) -> bool {
        let CaseLabel::Value { start, .. } = label else {
            return false;
        };
        self.scalar_case_bindings
            .contains(&fpas_sema::expr_lookup_key(start))
    }
}

fn binding_name(label: &CaseLabel) -> Option<&str> {
    let CaseLabel::Value {
        start: Expr::Designator(designator),
        ..
    } = label
    else {
        return None;
    };
    let [fpas_parser::DesignatorPart::Ident(name, _)] = designator.parts.as_slice() else {
        return None;
    };
    Some(name)
}
