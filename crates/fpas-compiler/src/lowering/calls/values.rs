//! Invocation of already evaluated callable values.
//!
//! **Documentation:** `docs/pascal/language/functions/first-class.md`

use super::super::context::{LoweringContext, unsupported};
use crate::CompileError;
use fpas_ir::{Operation, TypeId, ValueId};
use fpas_lexer::Span;
use fpas_parser::{Designator, Expr, PostfixOperation};

impl LoweringContext {
    /// Invoke an ordinary value or callable member from a postfix chain.
    pub(in crate::lowering) fn lower_postfix_value_call(
        &mut self,
        value: ValueId,
        operation: &PostfixOperation,
    ) -> Result<Option<(ValueId, TypeId)>, CompileError> {
        let key = fpas_sema::postfix_operation_lookup_key(operation);
        let Some(target) = self.value_calls.get(&key).cloned() else {
            return Ok(None);
        };
        let (callee, args, span) = match operation {
            PostfixOperation::Call { args, span } => (value, args, *span),
            PostfixOperation::MethodCall { args, span, .. } => (
                self.lower_postfix_callable_member(value, operation)?,
                args,
                *span,
            ),
            _ => return Err(unsupported(target.call_span, "value invocation suffix")),
        };
        let result = self
            .type_table
            .id(&target.result_ty, span.line, span.column)?;
        let value = self.lower_value_call(callee, args, result, span)?;
        Ok(Some((value, result)))
    }

    /// Evaluate a callable member before its positional arguments.
    pub(in crate::lowering) fn lower_member_value_call(
        &mut self,
        designator: &Designator,
        arguments: &[Expr],
        result: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let key = fpas_sema::designator_lookup_key(designator);
        let callee = if let Some(reads) = self.property_reads.get(&key).cloned() {
            self.lower_property_read(designator, &reads)?
        } else {
            self.lower_designator_read(designator)?
        };
        self.lower_value_call(callee, arguments, result, span)
    }

    /// Invoke an already evaluated target, then evaluate arguments left to right.
    pub(in crate::lowering) fn lower_value_call(
        &mut self,
        callee: ValueId,
        arguments: &[Expr],
        result: TypeId,
        span: Span,
    ) -> Result<ValueId, CompileError> {
        let callee = self.save_value(callee);
        let values = self.lower_expression_values(arguments, None, span)?;
        let callee = self.restore_value(callee, span)?;
        self.record_call_arguments(values.len(), span)?;
        self.emit_value(
            Operation::CallValue {
                callee,
                arguments: values,
            },
            result,
            span,
        )
    }
}
