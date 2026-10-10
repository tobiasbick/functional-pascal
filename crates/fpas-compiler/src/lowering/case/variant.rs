//! Result, Option, and data-enum variant case lowering.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/README.md`.

use fpas_ir::{Constant, Operation, Terminator};
use fpas_parser::CaseLabel;

use super::{CaseArmHead, CaseBody, CaseEnding};

use crate::CompileError;

use super::super::context::{LoweringContext, target, unsupported};
use super::super::types;

impl LoweringContext {
    /// Lowers variant cases with scoped pattern bindings and catch-all declarations.
    pub(in crate::lowering) fn lower_variant_case(
        &mut self,
        case_value: fpas_ir::ValueId,
        case_ty: fpas_ir::TypeId,
        arms: &[CaseArmHead<'_>],
        ending: &CaseEnding,
        span: fpas_lexer::Span,
        body: &mut CaseBody<'_>,
    ) -> Result<(), CompileError> {
        let case_local = self.declare_hidden_local(case_ty, span)?;
        self.write_local(case_local, case_value, span)?;
        let merge = self.new_block(span)?;
        let first_test = self.new_block(span)?;
        self.jump(first_test)?;
        self.switch_to(first_test);
        let mut has_merge = false;
        for (arm_index, arm) in arms.iter().enumerate() {
            for label in arm.labels {
                let next = self.new_block(arm.span)?;
                let mut bindings = Vec::new();
                match label {
                    CaseLabel::Pattern(pattern) => {
                        self.lower_pattern_test(case_local, case_ty, pattern, next, &mut bindings)?
                    }
                    CaseLabel::Value {
                        start, end: None, ..
                    } => self.lower_value_test(case_local, case_ty, start, next)?,
                    _ => return Err(unsupported(arm.span, "variant case label")),
                }
                self.begin_scope();
                for (name, ty, source) in bindings {
                    let value = self.emit_value(Operation::ReadLocal(source), ty, arm.span)?;
                    let local = self.declare_local(&name, ty, false, arm.span)?;
                    self.write_local(local, value, arm.span)?;
                }
                if let Some(guard) = arm.guard {
                    let condition = self.lower_expression(guard)?;
                    let guarded = self.new_block(arm.span)?;
                    self.terminate(Terminator::Branch {
                        condition,
                        then_target: target(guarded),
                        else_target: target(next),
                    })?;
                    self.switch_to(guarded);
                }
                body(self, Some(arm_index))?;
                if !self.is_terminated() {
                    self.jump(merge)?;
                    has_merge = true;
                }
                self.end_scope();
                self.switch_to(next);
            }
        }
        if ending.has_else {
            self.begin_scope();
            body(self, None)?;
            self.end_scope();
            if !self.is_terminated() {
                self.jump(merge)?;
                has_merge = true;
            }
        } else if !self.is_terminated() {
            let message = self.emit_value(
                Operation::Const(Constant::String(
                    "exhaustive case reached no variant".to_string(),
                )),
                types::STRING,
                span,
            )?;
            self.terminate(Terminator::Panic(message))?;
        }
        if has_merge {
            self.switch_to(merge);
        }
        Ok(())
    }
}
