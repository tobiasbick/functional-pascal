//! Array, dictionary, Result, Option, and `try` expression lowering.

use crate::CompileError;
use fpas_ir::{IrType, Operation, TypeId, ValueId};
use fpas_parser::Expr;

use super::super::context::target;
use super::super::context::{LoweringContext, unsupported};

impl LoweringContext {
    pub(in crate::lowering) fn lower_array_literal(
        &mut self,
        values: &[Expr],
        expression: &Expr,
    ) -> Result<ValueId, CompileError> {
        let ty = self.expression_ir_type(expression)?;
        let values = values
            .iter()
            .map(|value| self.lower_expression(value))
            .collect::<Result<Vec<_>, _>>()?;
        self.emit_value(Operation::MakeArray(values), ty, expression.span())
    }

    pub(in crate::lowering) fn lower_array_literal_as(
        &mut self,
        values: &[Expr],
        ty: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let element_ty = match self.type_kind(ty) {
            Some(IrType::Array(element)) => element,
            _ => return Err(unsupported(span, "expected array type")),
        };
        let values = values
            .iter()
            .map(|value| self.lower_expression_as(value, element_ty))
            .collect::<Result<Vec<_>, _>>()?;
        self.emit_value(Operation::MakeArray(values), ty, span)
    }

    pub(in crate::lowering) fn lower_dictionary_literal(
        &mut self,
        pairs: &[(Expr, Expr)],
        expression: &Expr,
    ) -> Result<ValueId, CompileError> {
        let ty = self.expression_ir_type(expression)?;
        let pairs = pairs
            .iter()
            .map(|(key, value)| Ok((self.lower_expression(key)?, self.lower_expression(value)?)))
            .collect::<Result<Vec<_>, CompileError>>()?;
        self.emit_value(Operation::MakeDictionary(pairs), ty, expression.span())
    }

    pub(in crate::lowering) fn lower_wrapper(
        &mut self,
        operand: Option<&Expr>,
        expression: &Expr,
        kind: u8,
    ) -> Result<ValueId, CompileError> {
        let ty = self.expression_ir_type(expression)?;
        self.lower_wrapper_as(operand, kind, ty, expression.span())
    }

    pub(in crate::lowering) fn lower_wrapper_as(
        &mut self,
        operand: Option<&Expr>,
        kind: u8,
        ty: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let expected_payload = match (kind, self.type_kind(ty)) {
            (0, Some(IrType::Result { ok, .. })) => Some(ok),
            (1, Some(IrType::Result { error, .. })) => Some(error),
            (2, Some(IrType::Option(value))) => Some(value),
            _ => None,
        };
        let lower_operand = |context: &mut Self, value: &Expr| match expected_payload {
            Some(expected) => context.lower_expression_as(value, expected),
            None => context.lower_expression(value),
        };
        let operation = match (kind, operand) {
            (0, Some(value)) => Operation::MakeOk(lower_operand(self, value)?),
            (1, Some(value)) => Operation::MakeError(lower_operand(self, value)?),
            (2, Some(value)) => Operation::MakeSome(lower_operand(self, value)?),
            (3, None) => Operation::MakeNone,
            _ => return Err(unsupported(span, "wrapper construction")),
        };
        self.emit_value(operation, ty, span)
    }

    pub(in crate::lowering) fn lower_expression_as(
        &mut self,
        expression: &Expr,
        expected: TypeId,
    ) -> Result<ValueId, CompileError> {
        match expression {
            Expr::RecordLiteral { fields, span } => {
                self.lower_record_literal_as(fields, expected, *span)
            }
            Expr::ArrayLiteral(values, span) => {
                self.lower_array_literal_as(values, expected, *span)
            }
            Expr::ResultOk(value, span) => self.lower_wrapper_as(Some(value), 0, expected, *span),
            Expr::ResultError(value, span) => {
                self.lower_wrapper_as(Some(value), 1, expected, *span)
            }
            Expr::OptionSome(value, span) => self.lower_wrapper_as(Some(value), 2, expected, *span),
            Expr::OptionNone(span) => self.lower_wrapper_as(None, 3, expected, *span),
            Expr::Call {
                designator,
                args,
                span,
            } if self
                .intrinsic_calls
                .get(&fpas_sema::expr_lookup_key(expression))
                .is_some_and(|name| {
                    name.eq_ignore_ascii_case(fpas_std::std_symbols::STD_TASK_CREATE_CHANNEL)
                }) =>
            {
                self.lower_call(
                    designator,
                    args,
                    expected,
                    *span,
                    fpas_sema::expr_lookup_key(expression),
                )
            }
            _ => self.lower_expression(expression),
        }
    }

    pub(in crate::lowering) fn lower_try(
        &mut self,
        inner: &Expr,
        expression: &Expr,
    ) -> Result<ValueId, CompileError> {
        let wrapper_ty = self.expression_ir_type(inner)?;
        let result_ty = self.expression_ir_type(expression)?;
        let wrapper = self.lower_expression(inner)?;
        let wrapper_local = self.declare_hidden_local(wrapper_ty, expression.span())?;
        self.write_local(wrapper_local, wrapper, expression.span())?;
        let test = match self.type_kind(wrapper_ty) {
            Some(IrType::Result { .. }) => Operation::IsResultOk(wrapper),
            Some(IrType::Option(_)) => Operation::IsOptionSome(wrapper),
            _ => return Err(unsupported(expression.span(), "try operand")),
        };
        let condition = self.emit_value(test, super::super::types::BOOLEAN, expression.span())?;
        let success = self.new_block(expression.span())?;
        let failure = self.new_block(expression.span())?;
        let merge = self.new_block(expression.span())?;
        self.terminate(fpas_ir::Terminator::Branch {
            condition,
            then_target: target(success),
            else_target: target(failure),
        })?;
        self.switch_to(failure);
        let failure_value = self.emit_value(
            Operation::ReadLocal(wrapper_local),
            wrapper_ty,
            expression.span(),
        )?;
        let function_result = self.current_result_type();
        let propagated = match self.type_kind(wrapper_ty) {
            Some(IrType::Result { error, .. }) => {
                let error = self.emit_value(
                    Operation::UnwrapError(failure_value),
                    error,
                    expression.span(),
                )?;
                self.emit_value(
                    Operation::MakeError(error),
                    function_result,
                    expression.span(),
                )?
            }
            Some(IrType::Option(_)) => {
                self.emit_value(Operation::MakeNone, function_result, expression.span())?
            }
            _ => return Err(unsupported(expression.span(), "try operand")),
        };
        self.terminate(fpas_ir::Terminator::Return(Some(propagated)))?;
        self.switch_to(success);
        let success_wrapper = self.emit_value(
            Operation::ReadLocal(wrapper_local),
            wrapper_ty,
            expression.span(),
        )?;
        let operation = match self.type_kind(wrapper_ty) {
            Some(IrType::Result { .. }) => Operation::UnwrapOk(success_wrapper),
            Some(IrType::Option(_)) => Operation::UnwrapSome(success_wrapper),
            _ => return Err(unsupported(expression.span(), "try operand")),
        };
        let payload = self.emit_value(operation, result_ty, expression.span())?;
        let payload_local = self.declare_hidden_local(result_ty, expression.span())?;
        self.write_local(payload_local, payload, expression.span())?;
        self.jump(merge)?;
        self.switch_to(merge);
        self.emit_value(
            Operation::ReadLocal(payload_local),
            result_ty,
            expression.span(),
        )
    }
}
