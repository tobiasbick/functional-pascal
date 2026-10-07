//! Call argument evaluation for positional and named calls.
//!
//! **Documentation:** `docs/pascal/language/functions/parameters.md`

use fpas_ir::ValueId;
use fpas_parser::Expr;

use crate::CompileError;

use super::super::context::{LoweringContext, unsupported};

impl LoweringContext {
    /// Evaluates call arguments in written order and returns them in parameter order.
    pub(in crate::lowering) fn lower_argument_values(
        &mut self,
        arguments: &[Expr],
        span: fpas_lexer::Span,
    ) -> Result<Vec<ValueId>, CompileError> {
        let values = self.lower_expression_values(arguments, None, span)?;
        let Some(order) = arguments.first().and_then(|first| {
            self.named_argument_orders
                .get(&fpas_sema::expr_lookup_key(first))
        }) else {
            return Ok(values);
        };
        order
            .iter()
            .map(|&written| {
                values
                    .get(written)
                    .copied()
                    .ok_or_else(|| unsupported(span, "named argument order"))
            })
            .collect()
    }

    /// Returns the argument value bound to the parameter at `index`.
    pub(in crate::lowering) fn argument_for_parameter<'a>(
        &self,
        arguments: &'a [Expr],
        index: usize,
    ) -> Option<&'a Expr> {
        let written = match arguments.first().and_then(|first| {
            self.named_argument_orders
                .get(&fpas_sema::expr_lookup_key(first))
        }) {
            Some(order) => *order.get(index)?,
            None => index,
        };
        arguments.get(written).map(Expr::argument_value)
    }
}
