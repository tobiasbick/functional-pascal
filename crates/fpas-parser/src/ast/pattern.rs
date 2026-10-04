//! Recursive patterns shared by statement and expression case arms.

use super::{Designator, Expr};
use fpas_lexer::Span;

/// A value test, explicit immutable binding, wildcard, or nested variant pattern.
///
/// **Documentation:** `docs/pascal/language/pattern-matching/README.md`.
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// Compare a literal or static constant, optionally over an inclusive range.
    Value {
        /// Single value or lower endpoint.
        start: Expr,
        /// Optional upper endpoint.
        end: Option<Expr>,
        /// Complete source span.
        span: Span,
    },
    /// Bind the matched value with `const Name`.
    Binding {
        /// Name introduced in the arm scope.
        name: String,
        /// Complete source span.
        span: Span,
    },
    /// Match a payload without binding it.
    Wildcard(Span),
    /// Match a qualified variant and recursively match its payloads.
    Variant {
        /// Qualified source name of the variant.
        designator: Designator,
        /// Payload patterns in declaration order.
        arguments: Vec<Pattern>,
        /// Whether a payload list was written, including an empty list.
        parenthesized: bool,
        /// Complete source span.
        span: Span,
    },
}

impl Pattern {
    /// Find an explicit binding by its case-insensitive name.
    #[must_use]
    pub fn binding(&self, name: &str) -> Option<&Self> {
        match self {
            Self::Binding {
                name: candidate, ..
            } if candidate.eq_ignore_ascii_case(name) => Some(self),
            Self::Variant { arguments, .. } => {
                arguments.iter().find_map(|argument| argument.binding(name))
            }
            _ => None,
        }
    }

    /// Visit value expressions in source order, including nested payload tests.
    pub fn visit_expressions<'a>(&'a self, visitor: &mut impl FnMut(&'a Expr)) {
        match self {
            Self::Value { start, end, .. } => {
                visitor(start);
                if let Some(end) = end {
                    visitor(end);
                }
            }
            Self::Variant { arguments, .. } => {
                for argument in arguments {
                    argument.visit_expressions(visitor);
                }
            }
            Self::Binding { .. } | Self::Wildcard(_) => {}
        }
    }

    /// Visit explicit binding names in source order.
    pub fn visit_bindings(&self, visitor: &mut impl FnMut(&str)) {
        match self {
            Self::Binding { name, .. } => visitor(name),
            Self::Variant { arguments, .. } => {
                for argument in arguments {
                    argument.visit_bindings(visitor);
                }
            }
            Self::Value { .. } | Self::Wildcard(_) => {}
        }
    }

    /// Return the complete pattern's source span.
    #[must_use]
    pub fn span(&self) -> Span {
        match self {
            Self::Value { span, .. }
            | Self::Binding { span, .. }
            | Self::Variant { span, .. }
            | Self::Wildcard(span) => *span,
        }
    }
}
