//! `if` expressions: conditions in order, and only the selected branch value runs.
//!
//! Documentation: `docs/pascal/language/control-flow/if-then-else.md`.

use fpas_ir::{Operation, ValueId};
use fpas_parser::{Expr, IfExprBranch};

use crate::CompileError;
use crate::lowering::context::LoweringContext;
use crate::lowering::control_flow::conditions;

impl LoweringContext {
    /// Stores the selected branch value in a hidden local and reads it after the merge.
    pub(super) fn lower_if_expression(
        &mut self,
        branches: &[IfExprBranch],
        else_value: &Expr,
        expression: &Expr,
    ) -> Result<ValueId, CompileError> {
        let span = expression.span();
        let result_ty = self.expression_ir_type(expression)?;
        let result = self.declare_hidden_local(result_ty, span)?;
        let merge = self.new_block(span)?;
        for branch in branches {
            let next = self.new_block(span)?;
            // `is` bindings in this condition are visible only in this branch's value.
            self.begin_scope();
            if conditions::has_pattern_test(&branch.condition) {
                self.lower_pattern_condition(&branch.condition, next)?;
            } else {
                let condition = self.lower_expression(&branch.condition)?;
                self.continue_if(condition, next, branch.condition.span())?;
            }
            let value = self.lower_expression_as(&branch.value, result_ty)?;
            self.write_local(result, value, span)?;
            self.end_scope();
            self.jump(merge)?;
            self.switch_to(next);
        }
        let value = self.lower_expression_as(else_value, result_ty)?;
        self.write_local(result, value, span)?;
        self.jump(merge)?;
        self.switch_to(merge);
        self.emit_value(Operation::ReadLocal(result), result_ty, span)
    }
}
