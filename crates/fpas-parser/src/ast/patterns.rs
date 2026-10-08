use super::{Designator, Expr};
use fpas_lexer::Span;

/// Pattern in a case label or in one payload position of another pattern.
///
/// **Documentation:** `docs/pascal/language/pattern-matching/syntax.md`
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// `const Name` binds the matched value for the arm.
    Binding {
        /// Bound name as written.
        name: String,
        /// Source span of the complete binding, including `const`.
        span: Span,
    },
    /// `_` ignores the matched value.
    Wildcard(Span),
    /// Literal, named constant, or fieldless variant compared with the value.
    Value(Expr),
    /// Enum variant with a payload list: `Shape.Circle(const R)`.
    ///
    /// **Documentation:** `docs/pascal/language/pattern-matching/enum-patterns.md`
    Variant {
        /// Variant name, optionally qualified by its enum type.
        constructor: Designator,
        /// Payload fields in declaration order.
        fields: Vec<PatternField>,
        /// Source span of the complete pattern.
        span: Span,
    },
    /// Result or Option pattern: `Ok(...)`, `Error(...)`, `Some(...)`, or `None`.
    ///
    /// **Documentation:** `docs/pascal/language/pattern-matching/result-option-patterns.md`
    Destructure {
        /// Result or option variant matched by the pattern.
        variant: DestructureVariant,
        /// Payload pattern, absent for `None`.
        payload: Option<Box<Pattern>>,
        /// Source span of the complete pattern.
        span: Span,
    },
}

impl Pattern {
    /// Returns the source span of the pattern.
    #[must_use]
    pub fn span(&self) -> Span {
        match self {
            Self::Binding { span, .. }
            | Self::Wildcard(span)
            | Self::Variant { span, .. }
            | Self::Destructure { span, .. } => *span,
            Self::Value(expr) => expr.span(),
        }
    }
}

/// One positional payload field of a variant pattern.
///
/// **Documentation:** `docs/pascal/language/pattern-matching/enum-patterns.md`
#[derive(Debug, Clone, PartialEq)]
pub struct PatternField {
    /// Field name written as `Name := Pattern`, which patterns reject.
    pub label: Option<(String, Span)>,
    /// Pattern for the field value.
    pub pattern: Pattern,
}

/// Result or option variant matched by a destructure pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestructureVariant {
    /// Successful result containing a value.
    Ok,
    /// Failed result containing an error value.
    Error,
    /// Present option containing a value.
    Some,
    /// Empty option without a value.
    None,
}
