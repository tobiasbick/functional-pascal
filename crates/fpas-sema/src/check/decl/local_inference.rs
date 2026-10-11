//! Limited statement-local inference from literals, constructors, and type conversions.
//!
//! **Documentation:** `docs/pascal/language/basics/local-variables.md`

use fpas_diagnostics::codes::SEMA_TYPE_ANNOTATION_REQUIRED;
use fpas_lexer::Span;
use fpas_parser::{Expr, UnaryOp};

use super::Checker;
use crate::types::{Ty, needs_expected_type};

impl Checker {
    /// Checks an unannotated local initializer without inferring ordinary call results.
    pub(super) fn infer_local_binding(
        &mut self,
        name: &str,
        value: &Expr,
        span: Span,
        mutable: bool,
    ) -> Ty {
        let diagnostic_count = self.errors.len();
        let ty = self.check_expr(value);
        if ty.is_error() {
            return ty;
        }
        let initializer = unparenthesized(value);
        if self.errors.len() > diagnostic_count
            && !matches!(initializer, Expr::ArrayLiteral(..) | Expr::DictLiteral(..))
        {
            return Ty::Error;
        }
        let allowed = match initializer {
            Expr::Integer(..) | Expr::Real(..) | Expr::Str(..) | Expr::Bool(..) => true,
            Expr::UnaryOp {
                op: UnaryOp::Negate,
                operand,
                ..
            } => {
                matches!(unparenthesized(operand), Expr::Integer(..) | Expr::Real(..))
            }
            Expr::ArrayLiteral(..) | Expr::DictLiteral(..) => self.collection_evidence(initializer),
            Expr::Call { .. } => {
                let key = Self::expr_lookup_key(initializer);
                self.distinct_conversions.contains(&key)
                    || self.subrange_conversions.contains_key(&key)
                    || self.record_constructions.contains(&key)
                    || self.enum_constructions.contains(&key)
            }
            Expr::Designator(designator) => self
                .enum_members
                .contains(&crate::designator_lookup_key(designator)),
            Expr::OptionSome(..) => true,
            _ => false,
        };
        if allowed && complete_type(&ty) {
            return ty;
        }
        self.error_with_code(
            SEMA_TYPE_ANNOTATION_REQUIRED,
            format!("Local binding `{name}` requires a type annotation"),
            format!(
                "Use `{keyword} {name}: Type := ...;`, for example `{keyword} {name}: integer := 0;`. Omit the annotation only for literals with complete types, record or enum constructions with all type arguments determined, or explicit type conversions.",
                keyword = if mutable { "var" } else { "const" }
            ),
            span,
        );
        Ty::Error
    }

    fn collection_evidence(&self, expr: &Expr) -> bool {
        match unparenthesized(expr) {
            Expr::ArrayLiteral(values, _) => self.matching_evidence(values.iter()),
            Expr::DictLiteral(entries, _) => {
                self.matching_evidence(entries.iter().map(|(key, _)| key))
                    && self.matching_evidence(entries.iter().map(|(_, value)| value))
            }
            _ => self.checked_type(expr).is_some_and(complete_type),
        }
    }

    fn matching_evidence<'a>(&self, mut values: impl Iterator<Item = &'a Expr>) -> bool {
        let Some(first) = values.next() else {
            return false;
        };
        let Some(expected) = self.checked_type(first) else {
            return false;
        };
        self.collection_evidence(first)
            && values.all(|value| {
                self.collection_evidence(value)
                    && self.checked_type(value).is_some_and(|actual| {
                        expected.assignment_compatible_with(actual)
                            && actual.assignment_compatible_with(expected)
                    })
            })
    }

    fn checked_type(&self, expr: &Expr) -> Option<&Ty> {
        self.expr_types.get(&Self::expr_lookup_key(expr))
    }
}

fn unparenthesized(mut expr: &Expr) -> &Expr {
    while let Expr::Paren(inner, _) = expr {
        expr = inner;
    }
    expr
}

fn complete_type(ty: &Ty) -> bool {
    if needs_expected_type(ty) {
        return false;
    }
    match ty {
        Ty::Array(inner) | Ty::Channel(inner) | Ty::Option(inner) | Ty::Task(inner) => {
            complete_type(inner)
        }
        Ty::Result(left, right) | Ty::Dict(left, right) => {
            complete_type(left) && complete_type(right)
        }
        Ty::Record(record) => record.type_args.iter().all(complete_type),
        Ty::Enum(enumeration) => enumeration.type_args.iter().all(complete_type),
        _ => true,
    }
}
