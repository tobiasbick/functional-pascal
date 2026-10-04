//! Constant declaration checking.
//!
//! **Documentation:** `docs/pascal/language/basics/constants.md` (from the repository root).

use super::Checker;
use crate::scope::SymbolKind;
use fpas_parser::{DesignatorPart, Expr};

impl Checker {
    /// Classify static expressions separately from binding immutability.
    pub(in crate::check) fn const_expr_is_compile_time_known(&mut self, expr: &Expr) -> bool {
        match expr {
            Expr::If(_) | Expr::Case(_) => false,
            Expr::Integer(..) | Expr::Real(..) | Expr::Str(..) | Expr::Bool(..) => true,
            Expr::Designator(designator) => {
                if !designator
                    .parts
                    .iter()
                    .all(|part| matches!(part, DesignatorPart::Ident(..)))
                {
                    return false;
                }

                let full_name = self.resolve_designator_name(designator);
                self.ensure_fq_std_unit_loaded(&full_name);

                self.scopes
                    .lookup(&full_name)
                    .or_else(|| {
                        self.designator_root_symbol(&designator.parts)
                            .map(|(symbol, _)| symbol)
                    })
                    .is_some_and(|symbol| {
                        matches!(symbol.kind, SymbolKind::Const | SymbolKind::EnumMember)
                    })
            }
            Expr::UnaryOp { operand, .. } | Expr::Paren(operand, _) => {
                self.const_expr_is_compile_time_known(operand)
            }
            Expr::BinaryOp { left, right, .. } => {
                self.const_expr_is_compile_time_known(left)
                    && self.const_expr_is_compile_time_known(right)
            }
            Expr::ArrayLiteral(elements, _) => elements
                .iter()
                .all(|element| self.const_expr_is_compile_time_known(element)),
            Expr::DictLiteral(pairs, _) => pairs.iter().all(|(key, value)| {
                self.const_expr_is_compile_time_known(key)
                    && self.const_expr_is_compile_time_known(value)
            }),
            Expr::RecordConstruction { fields, .. } => {
                self.record_construction_is_static(expr, fields)
            }
            Expr::Call { .. }
                if self
                    .record_constructions
                    .contains(&Self::expr_lookup_key(expr)) =>
            {
                self.record_construction_is_static(expr, &[])
            }
            Expr::ResultOk(inner, _) | Expr::ResultError(inner, _) | Expr::OptionSome(inner, _) => {
                self.const_expr_is_compile_time_known(inner)
            }
            Expr::Try(..) | Expr::Go(..) | Expr::Closure(_) | Expr::VarArgument(..) => false,
            Expr::OptionNone(_) => true,
            Expr::Call { .. } | Expr::Postfix { .. } | Expr::InvalidRecord(..) | Expr::Error(_) => {
                false
            }
            Expr::RecordUpdate { base, fields, .. } => {
                self.const_expr_is_compile_time_known(base)
                    && fields
                        .iter()
                        .all(|f| self.const_expr_is_compile_time_known(&f.value))
            }
        }
    }

    fn record_construction_is_static(
        &mut self,
        expr: &Expr,
        fields: &[fpas_parser::FieldInit],
    ) -> bool {
        if !fields
            .iter()
            .all(|field| self.const_expr_is_compile_time_known(&field.value))
        {
            return false;
        }
        let Some(crate::types::Ty::Record(record)) =
            self.expr_types.get(&Self::expr_lookup_key(expr)).cloned()
        else {
            return false;
        };
        let defaults = self
            .record_defaults
            .get(&record.name)
            .cloned()
            .unwrap_or_default();
        record.fields.iter().all(|(name, _)| {
            fields
                .iter()
                .any(|field| field.name.eq_ignore_ascii_case(name))
                || defaults
                    .iter()
                    .find(|(field, _)| field.eq_ignore_ascii_case(name))
                    .and_then(|(_, default)| default.as_ref()?.expression())
                    .is_some_and(|expression| self.const_expr_is_compile_time_known(expression))
        })
    }
}
