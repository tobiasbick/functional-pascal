//! Constant classification is a language rule, independent of optimizer folding.

use super::Checker;
use crate::scope::SymbolKind;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_NON_CONSTANT_EXPRESSION;
use fpas_parser::{Designator, DesignatorPart, Expr, PostfixOperation};

impl Checker {
    /// Classifies typed aggregate initializers, including omitted nested defaults.
    pub(crate) fn const_initializer_is_compile_time_known(
        &mut self,
        expr: &Expr,
        expected: &Ty,
    ) -> bool {
        if !self.const_expr_is_compile_time_known(expr) {
            return false;
        }
        match (expr, self.resolve_visible_type(expected)) {
            (Expr::Paren(inner, _), ty) => self.const_initializer_is_compile_time_known(inner, &ty),
            (Expr::ArrayLiteral(elements, _), Ty::Array(inner)) => elements
                .iter()
                .all(|value| self.const_initializer_is_compile_time_known(value, &inner)),
            (Expr::DictLiteral(pairs, _), Ty::Dict(key, value)) => pairs.iter().all(|(k, v)| {
                self.const_initializer_is_compile_time_known(k, &key)
                    && self.const_initializer_is_compile_time_known(v, &value)
            }),
            (Expr::OptionSome(value, _), Ty::Option(inner)) => {
                self.const_initializer_is_compile_time_known(value, &inner)
            }
            (Expr::ResultOk(value, _), Ty::Result(inner, _))
            | (Expr::ResultError(value, _), Ty::Result(_, inner)) => {
                self.const_initializer_is_compile_time_known(value, &inner)
            }
            (Expr::RecordUpdate { fields, .. }, Ty::Record(record)) => {
                record.fields.iter().all(|(name, ty)| {
                    fields
                        .iter()
                        .find(|field| field.name.eq_ignore_ascii_case(name))
                        .is_none_or(|field| {
                            self.const_initializer_is_compile_time_known(&field.value, ty)
                        })
                })
            }
            _ => true,
        }
    }

    /// Recognizes static forms only when every dependency is compile-time known.
    pub(crate) fn const_expr_is_compile_time_known(&mut self, expr: &Expr) -> bool {
        self.non_constant_part(expr).is_none()
    }

    /// Requires a static value label or range endpoint, naming its runtime dependency.
    pub(crate) fn require_case_constant(&mut self, expr: &Expr) {
        if let Some((name, span)) = self.non_constant_part(expr) {
            self.error_with_code(SEMA_NON_CONSTANT_EXPRESSION,
                format!("Case label requires a compile-time constant; {name} is computed at runtime"),
                "Use a literal or a compile-time constant for value labels and range endpoints. Put dynamic conditions in a guard, for example `when const Value if Value = ReadValue(): ...`.",
                span);
        }
    }

    /// Rejects a call written as a scalar case label, naming the called routine.
    pub(crate) fn require_case_constant_call(
        &mut self,
        callee: &Designator,
        span: fpas_lexer::Span,
    ) {
        let name = match callee.parts.last() {
            Some(DesignatorPart::Ident(name, _)) => name.clone(),
            _ => Self::resolve_designator_name(callee),
        };
        self.error_with_code(SEMA_NON_CONSTANT_EXPRESSION,
            format!("Case label requires a compile-time constant; call `{name}` is computed at runtime"),
            "Use a literal or a compile-time constant for value labels and range endpoints. Put dynamic conditions in a guard, for example `when const Value if Value = ReadValue(): ...`.",
            span);
    }

    fn non_constant_part(&mut self, expr: &Expr) -> Option<(String, fpas_lexer::Span)> {
        if let Some(known) = self
            .record_default_constants
            .get(&Self::expr_lookup_key(expr))
        {
            return (!known).then(|| ("record field default".into(), expr.span()));
        }
        match expr {
            Expr::Integer(..)
            | Expr::Real(..)
            | Expr::Str(..)
            | Expr::Bool(..)
            | Expr::OptionNone(_) => None,
            Expr::Designator(designator) => {
                let full_name = Self::resolve_designator_name(designator);
                self.ensure_fq_std_unit_loaded(&full_name);
                let known =
                    self.constant_designator_binding(designator)
                        .is_some_and(|(symbol, _)| {
                            symbol.kind == SymbolKind::EnumMember
                                || (symbol.kind == SymbolKind::Const
                                    && symbol
                                        .constant
                                        .as_ref()
                                        .is_none_or(|info| info.compile_time))
                        });
                (!known).then(|| (format!("binding `{full_name}`"), expr.span()))
            }
            Expr::VarArgument { designator, .. } => Some((
                format!(
                    "`var` argument `{}`",
                    Self::resolve_designator_name(designator)
                ),
                expr.span(),
            )),
            Expr::Call { args, .. } if self.constant_record_type(expr).is_some() => {
                if let Some(part) = args
                    .iter()
                    .find_map(|argument| self.non_constant_part(argument.argument_value()))
                {
                    return Some(part);
                }
                let Some(record) = self.constant_record_type(expr) else {
                    return Some(("record construction".into(), expr.span()));
                };
                let defaults = self
                    .record_defaults
                    .get(&record.name)
                    .cloned()
                    .unwrap_or_default();
                defaults.into_iter().find_map(|(name, value)| {
                    if args.iter().any(|argument| {
                        argument
                            .argument_name()
                            .is_some_and(|provided| provided.eq_ignore_ascii_case(&name))
                    }) {
                        None
                    } else {
                        value.and_then(|value| self.non_constant_part(&value))
                    }
                })
            }
            Expr::Call { designator, .. } => Some((
                format!("call `{}`", Self::resolve_designator_name(designator)),
                expr.span(),
            )),
            Expr::Postfix { operations, .. } => {
                let name = operations.iter().find_map(|operation| match operation {
                    PostfixOperation::MethodCall { name, .. } => Some(name),
                    _ => None,
                });
                Some((
                    name.map_or_else(
                        || "postfix expression".to_string(),
                        |name| format!("call `{name}`"),
                    ),
                    expr.span(),
                ))
            }
            Expr::UnaryOp { operand, .. }
            | Expr::Paren(operand, _)
            | Expr::ResultOk(operand, _)
            | Expr::ResultError(operand, _)
            | Expr::OptionSome(operand, _)
            | Expr::NamedArgument { value: operand, .. } => self.non_constant_part(operand),
            Expr::BinaryOp { left, right, .. } => self
                .non_constant_part(left)
                .or_else(|| self.non_constant_part(right)),
            Expr::ArrayLiteral(elements, _) => elements
                .iter()
                .find_map(|element| self.non_constant_part(element)),
            Expr::DictLiteral(pairs, _) => pairs.iter().find_map(|(key, value)| {
                self.non_constant_part(key)
                    .or_else(|| self.non_constant_part(value))
            }),
            Expr::RecordUpdate { base, fields, .. } => self.non_constant_part(base).or_else(|| {
                fields
                    .iter()
                    .find_map(|field| self.non_constant_part(&field.value))
            }),
            Expr::Try(..) | Expr::Go(..) | Expr::Closure(_) | Expr::Is { .. } | Expr::Error(_) => {
                Some(("expression".to_string(), expr.span()))
            }
        }
    }
}
