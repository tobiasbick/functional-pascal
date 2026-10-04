//! Explicit array mutation calls through ordinary var-parameter entries.
//!
//! Documentation: `docs/pascal/std/collections/array/mutating.md`.

use super::*;

impl LoweringContext {
    /// Reserve caller storage before evaluating the appended value.
    pub(super) fn lower_array_push(
        &mut self,
        arguments: &[Expr],
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let [Expr::VarArgument(target, target_span), value] = arguments else {
            return Err(unsupported(span, "Std.Arrays.Push var arguments"));
        };
        let key = super::super::closures::array_mutation::call_key(true, span);
        let callable = self
            .resolve_callable(&key)
            .ok_or_else(|| unsupported(span, "array push entry"))?;
        let reference = self.lower_var_argument(target, *target_span)?;
        let reference = self.save_value(reference);
        let value = self.lower_expression_as(value, callable.parameters[1])?;
        let reference = self.restore_value(reference, span)?;
        self.record_call_arguments(2, span)?;
        self.emit_value(
            Operation::CallDirect {
                function: callable.function,
                arguments: vec![reference, value],
            },
            callable.result,
            span,
        )
    }

    /// Remove one value under the same activation and cleanup as a source var call.
    pub(super) fn lower_array_pop(
        &mut self,
        arguments: &[Expr],
        result: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let [Expr::VarArgument(target, target_span)] = arguments else {
            return Err(unsupported(span, "Std.Arrays.Pop var argument"));
        };
        let key = super::super::closures::array_mutation::call_key(false, span);
        let callable = self
            .resolve_callable(&key)
            .ok_or_else(|| unsupported(span, "array pop entry"))?;
        let reference = self.lower_var_argument(target, *target_span)?;
        self.record_call_arguments(1, span)?;
        self.emit_value(
            Operation::CallDirect {
                function: callable.function,
                arguments: vec![reference],
            },
            result,
            span,
        )
    }
}
