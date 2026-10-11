//! Inclusive integer bounds shared by membership tests and checked conversions.
//!
//! **Documentation:** `docs/pascal/language/types/subranges.md`

use super::super::{
    context::LoweringContext,
    types::{BOOLEAN, INTEGER},
};
use crate::CompileError;
use fpas_ir::{BinaryOperation, Constant, Operation, ValueId};
use fpas_lexer::Span;
use fpas_sema::SubrangeTy;

impl LoweringContext {
    /// Tests both inclusive endpoints using comparisons that cannot overflow.
    pub(in crate::lowering) fn lower_subrange_check(
        &mut self,
        value: ValueId,
        range: &SubrangeTy,
        span: Span,
    ) -> Result<ValueId, CompileError> {
        let lower = self.emit_value(
            Operation::Const(Constant::Integer(range.lower)),
            INTEGER,
            span,
        )?;
        let above_lower = self.emit_value(
            Operation::Binary {
                operation: BinaryOperation::GreaterEqualInteger,
                left: value,
                right: lower,
            },
            BOOLEAN,
            span,
        )?;
        let upper = self.emit_value(
            Operation::Const(Constant::Integer(range.upper)),
            INTEGER,
            span,
        )?;
        let below_upper = self.emit_value(
            Operation::Binary {
                operation: BinaryOperation::LessEqualInteger,
                left: value,
                right: upper,
            },
            BOOLEAN,
            span,
        )?;
        self.emit_value(
            Operation::Binary {
                operation: BinaryOperation::AndBoolean,
                left: above_lower,
                right: below_upper,
            },
            BOOLEAN,
            span,
        )
    }
}
