//! Range-checked conversions using integer representation and the ordinary panic path.
//!
//! **Documentation:** `docs/pascal/language/types/subranges.md`

use super::super::{
    context::LoweringContext,
    types::{INTEGER, STRING},
};
use crate::CompileError;
use fpas_ir::{BinaryOperation, Constant, IntrinsicId, Operation, Terminator, ValueId};
use fpas_lexer::Span;
use fpas_parser::Expr;
use fpas_sema::SubrangeTy;

impl LoweringContext {
    /// Evaluates the input once, rejects either invalid endpoint, and returns its integer value.
    pub(super) fn lower_subrange_conversion(
        &mut self,
        arguments: &[Expr],
        range: &SubrangeTy,
        span: Span,
    ) -> Result<ValueId, CompileError> {
        let [argument] = arguments else {
            return Err(super::super::context::unsupported(
                span,
                "subrange conversion without one argument",
            ));
        };
        let value = self.lower_expression(argument)?;
        let local = self.declare_hidden_local(INTEGER, span)?;
        self.write_local(local, value, span)?;
        let failure = self.new_block(span)?;
        let valid = self.lower_subrange_check(value, range, span)?;
        self.continue_if(valid, failure, span)?;
        let success = self.new_block(span)?;
        self.jump(success)?;
        self.switch_to(failure);
        let value = self.emit_value(Operation::ReadLocal(local), INTEGER, span)?;
        let text = self.emit_value(
            Operation::Intrinsic {
                intrinsic: IntrinsicId::new(u32::from(u16::from(fpas_bytecode::Intrinsic::Conv(
                    fpas_bytecode::ConvIntrinsic::IntToStr,
                )))),
                arguments: vec![value],
            },
            STRING,
            span,
        )?;
        let prefix = self.emit_value(
            Operation::Const(Constant::String(format!(
                "Subrange `{}`: value ",
                range.name
            ))),
            STRING,
            span,
        )?;
        let message = self.emit_value(
            Operation::Binary {
                operation: BinaryOperation::ConcatString,
                left: prefix,
                right: text,
            },
            STRING,
            span,
        )?;
        let suffix = self.emit_value(
            Operation::Const(Constant::String(format!(
                " is outside {}..{}",
                range.lower, range.upper
            ))),
            STRING,
            span,
        )?;
        let message = self.emit_value(
            Operation::Binary {
                operation: BinaryOperation::ConcatString,
                left: message,
                right: suffix,
            },
            STRING,
            span,
        )?;
        self.set_last_instruction_source(span)?;
        self.terminate(Terminator::Panic(message))?;
        self.switch_to(success);
        self.emit_value(Operation::ReadLocal(local), INTEGER, span)
    }
}
