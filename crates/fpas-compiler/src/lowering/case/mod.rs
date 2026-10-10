//! Scalar and variant `case` lowering for statements and expressions.
//!
//! **Documentation:** `docs/pascal/language/control-flow/case-of-intro.md`.

mod expression;
mod patterns;
mod scalar;
mod variant;

use fpas_lexer::Span;
use fpas_parser::{CaseLabel, Expr};

use crate::CompileError;

use super::context::LoweringContext;

/// Labels, guard, and span of one arm, independent of its statement or value body.
pub(in crate::lowering) struct CaseArmHead<'a> {
    pub(in crate::lowering) labels: &'a [CaseLabel],
    pub(in crate::lowering) guard: Option<&'a Expr>,
    pub(in crate::lowering) span: Span,
}

/// Lowers the body of arm `Some(index)`, or of the `else` arm for `None`, at the current block.
pub(in crate::lowering) type CaseBody<'a> =
    dyn FnMut(&mut LoweringContext, Option<usize>) -> Result<(), CompileError> + 'a;

/// How the arms of one `case` end.
pub(in crate::lowering) struct CaseEnding {
    /// Whether an `else` arm follows the `when` arms.
    pub(in crate::lowering) has_else: bool,
    /// Whether reaching the end without a matching arm must panic, as for an expression.
    pub(in crate::lowering) require_match: bool,
}
