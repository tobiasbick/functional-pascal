//! `if`, `elsif`, and `while` conditions that contain `is` pattern tests.
//!
//! The condition's `and` chain is lowered left to right with short-circuit
//! branches. Each `is` test declares its bindings before later conditions run, so
//! they are visible in those conditions and in the guarded body.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/is-test.md`.

use fpas_ir::{BlockId, Operation};
use fpas_parser::Expr;

use crate::CompileError;

use super::super::context::LoweringContext;

impl LoweringContext {
    /// Lowers a condition with `is` tests; falls through on success, branches to `fail` otherwise.
    ///
    /// The caller opens the scope that holds the bindings.
    pub(super) fn lower_pattern_condition(
        &mut self,
        condition: &Expr,
        fail: BlockId,
    ) -> Result<(), CompileError> {
        let mut conjuncts = Vec::new();
        condition.collect_conjuncts(&mut conjuncts);
        for conjunct in conjuncts {
            if let Expr::Is {
                value,
                pattern,
                span,
            } = conjunct
            {
                let ty = self.expression_ir_type(value)?;
                let tested = self.lower_expression(value)?;
                let source = self.declare_hidden_local(ty, *span)?;
                self.write_local(source, tested, *span)?;
                let mut bindings = Vec::new();
                self.lower_pattern_test(source, ty, pattern, fail, &mut bindings)?;
                for (name, binding_ty, local) in bindings {
                    let value = self.emit_value(Operation::ReadLocal(local), binding_ty, *span)?;
                    let named = self.declare_local(&name, binding_ty, false, *span)?;
                    self.write_local(named, value, *span)?;
                }
            } else {
                let matched = self.lower_expression(conjunct)?;
                self.continue_if(matched, fail, conjunct.span())?;
            }
        }
        Ok(())
    }
}

/// True when a top-level `and` condition contains an `is` test.
pub(super) fn has_pattern_test(condition: &Expr) -> bool {
    let mut conjuncts = Vec::new();
    condition.collect_conjuncts(&mut conjuncts);
    conjuncts
        .iter()
        .any(|conjunct| matches!(conjunct, Expr::Is { .. }))
}
