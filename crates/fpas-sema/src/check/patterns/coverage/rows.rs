//! Normalize checked patterns for coverage without changing their AST identities.

use super::{Checker, RowPattern, Tag};
use fpas_ir::Constant;
use fpas_parser::{Expr, Pattern};

impl Checker {
    /// Normalize one checked pattern while retaining its original AST identity.
    pub(super) fn coverage_pattern(&self, pattern: &Pattern) -> RowPattern {
        if let Some(variant) = self
            .pattern_infos
            .get(&crate::pattern_lookup_key(pattern))
            .and_then(|info| info.variant)
        {
            let arguments = match pattern {
                Pattern::Variant { arguments, .. } => arguments
                    .iter()
                    .map(|pattern| self.coverage_pattern(pattern))
                    .collect(),
                _ => Vec::new(),
            };
            return RowPattern::Constructor(Tag::Variant(variant), arguments);
        }
        match pattern {
            Pattern::Binding { .. } | Pattern::Wildcard(_) => RowPattern::Any,
            Pattern::Value { start, end, .. } => self.coverage_value(start, end.as_ref()),
            Pattern::Variant { .. } => RowPattern::Constructor(
                Tag::Opaque(format!("invalid:{}", pattern.span().offset)),
                vec![],
            ),
        }
    }

    fn coverage_value(&self, expression: &Expr, end: Option<&Expr>) -> RowPattern {
        let value = self.evaluate_static_expression(expression);
        let tag = match value {
            Some(Constant::Integer(value)) => {
                let end = end
                    .and_then(|end| self.evaluate_static_expression(end))
                    .and_then(|end| match end {
                        Constant::Integer(value) => Some(value),
                        _ => None,
                    })
                    .unwrap_or(value);
                Tag::Integer(value, end)
            }
            Some(Constant::Boolean(value)) => Tag::Boolean(value),
            Some(Constant::String(value)) => {
                let upper = end
                    .and_then(|end| self.evaluate_static_expression(end))
                    .and_then(|end| match end {
                        Constant::String(value) => Some(value),
                        _ => None,
                    })
                    .unwrap_or_else(|| value.clone());
                Tag::String(value, upper)
            }
            Some(Constant::Real(value)) => {
                Tag::Real(if value == 0.0 { 0 } else { value.to_bits() })
            }
            _ => Tag::Opaque(match expression {
                Expr::Designator(designator) => {
                    Self::designator_name(designator).to_ascii_lowercase()
                }
                _ => format!("expression:{}", expression.span().offset),
            }),
        };
        RowPattern::Constructor(tag, vec![])
    }
}
