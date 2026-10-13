//! Boolean values produced by nonbinding pattern tests.
//!
//! Documentation: `docs/pascal/language/pattern-matching/is-test.md`.

use fpas_ir::{Constant, Operation, ValueId};
use fpas_lexer::Span;
use fpas_parser::{Expr, Pattern};

use crate::CompileError;
use crate::lowering::{context::LoweringContext, types};

impl LoweringContext {
    /// Evaluates the selector once and joins the matching and failing paths into a Boolean.
    pub(super) fn lower_is_test(
        &mut self,
        value: &Expr,
        pattern: &Pattern,
        span: Span,
    ) -> Result<ValueId, CompileError> {
        let source_ty = self.expression_ir_type(value)?;
        let tested = self.lower_expression(value)?;
        let source = self.declare_hidden_local(source_ty, span)?;
        self.write_local(source, tested, span)?;
        let result = self.declare_hidden_local(types::BOOLEAN, span)?;
        let fail = self.new_block(span)?;
        let merge = self.new_block(span)?;
        self.lower_pattern_test(source, source_ty, pattern, fail, &mut Vec::new())?;
        let matched = self.emit_value(
            Operation::Const(Constant::Boolean(true)),
            types::BOOLEAN,
            span,
        )?;
        self.write_local(result, matched, span)?;
        self.jump(merge)?;
        self.switch_to(fail);
        let unmatched = self.emit_value(
            Operation::Const(Constant::Boolean(false)),
            types::BOOLEAN,
            span,
        )?;
        self.write_local(result, unmatched, span)?;
        self.jump(merge)?;
        self.switch_to(merge);
        self.emit_value(Operation::ReadLocal(result), types::BOOLEAN, span)
    }
}
