//! Checked targets for ordinary calls of callable values.

use crate::types::Ty;
use fpas_lexer::Span;
use std::collections::HashMap;

/// Signature and result of a callable-value invocation without an implicit receiver.
///
/// **Documentation:** `docs/pascal/language/functions/first-class.md`
#[derive(Debug, Clone, PartialEq)]
pub struct ValueCallTarget {
    /// Checked function or procedure type of the target value.
    pub callable_ty: Ty,
    /// Result type after checking and substituting argument types.
    pub result_ty: Ty,
    /// Call span; expression-target suffixes begin at their opening parenthesis.
    pub call_span: Span,
}

/// Maps expression, statement-designator or postfix-operation identity to a value call.
pub type ValueCallMap = HashMap<usize, ValueCallTarget>;
