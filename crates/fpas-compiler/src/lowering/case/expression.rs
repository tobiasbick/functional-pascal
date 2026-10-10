//! `case` expressions: the shared arm lowering writes the selected arm value.
//!
//! Documentation: `docs/pascal/language/control-flow/case-of-intro.md`.

use fpas_ir::{Operation, ValueId};
use fpas_parser::{CaseExprArm, CaseExprElse, Expr};

use crate::CompileError;

use super::super::context::LoweringContext;
use super::{CaseArmHead, CaseEnding};

impl LoweringContext {
    /// Stores the value of the matching arm in a hidden local and reads it after the merge.
    pub(in crate::lowering) fn lower_case_expression(
        &mut self,
        selector: &Expr,
        arms: &[CaseExprArm],
        else_arm: Option<&CaseExprElse>,
        expression: &Expr,
    ) -> Result<ValueId, CompileError> {
        let span = expression.span();
        let result_ty = self.expression_ir_type(expression)?;
        let result = self.declare_hidden_local(result_ty, span)?;
        let heads = arms
            .iter()
            .map(|arm| CaseArmHead {
                labels: &arm.labels,
                guard: arm.guard.as_ref(),
                span: arm.span,
            })
            .collect::<Vec<_>>();
        let ending = CaseEnding {
            has_else: else_arm.is_some(),
            require_match: true,
        };
        self.lower_case_arms(selector, &heads, &ending, span, &mut |context, index| {
            let value = match (index, else_arm) {
                (Some(index), _) => &arms[index].value,
                (None, Some(else_arm)) => &else_arm.value,
                (None, None) => return Ok(()),
            };
            let value = context.lower_expression_as(value, result_ty)?;
            context.write_local(result, value, span)
        })?;
        self.emit_value(Operation::ReadLocal(result), result_ty, span)
    }
}
