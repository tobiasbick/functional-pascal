//! Capture proofs propagated alongside expression types, without changing signatures.
//!
//! **Documentation:** `docs/pascal/language/functions/discard.md`

use super::{Checker, types::TaskSafety};
use crate::types::Ty;
use fpas_parser::{Designator, DesignatorPart, Expr, FieldInit, PostfixOperation};
use fpas_unit::interface::DiscardInfo;

impl Checker {
    /// Reads the static proof for an already checked expression.
    pub(crate) fn discard_info(&self, expr: &Expr) -> DiscardInfo {
        self.discard_exprs
            .get(&Self::expr_lookup_key(expr))
            .copied()
            .unwrap_or_default()
    }

    /// Propagates capture proofs without altering expression types or signatures.
    pub(crate) fn record_discard_info(&mut self, expr: &Expr) {
        let key = Self::expr_lookup_key(expr);
        let ty = self.expr_types.get(&key).cloned().unwrap_or(Ty::Error);
        let mut info = match expr {
            Expr::Paren(inner, _) | Expr::Try(inner, _) => self.discard_info(inner),
            Expr::ResultOk(inner, _) | Expr::ResultError(inner, _) | Expr::OptionSome(inner, _) => {
                DiscardInfo {
                    value: self.discard_info(inner).value,
                    ..Default::default()
                }
            }
            Expr::Designator(designator) => self.designator_discard_info(designator),
            Expr::Call { designator, .. } => {
                let target = self
                    .method_calls
                    .get(&key)
                    .map(|target| target.qualified_name())
                    .or_else(|| {
                        self.fluent_calls
                            .get(&key)
                            .map(|target| target.name.as_str())
                    });
                let result = target
                    .map(|name| self.scopes.discard_info(name).result)
                    .unwrap_or_else(|| self.designator_discard_info(designator).result);
                DiscardInfo {
                    value: result,
                    ..Default::default()
                }
            }
            Expr::Closure(_) => self
                .closure_infos
                .get(&key)
                .map(|closure| DiscardInfo {
                    value: closure.captures.iter().all(|capture| capture.task_free),
                    result: self.discard_exprs.get(&key).is_some_and(|info| info.result),
                })
                .unwrap_or_default(),
            Expr::ArrayLiteral(elements, _) => DiscardInfo {
                value: !elements.is_empty()
                    && elements.iter().all(|expr| self.discard_info(expr).value),
                ..Default::default()
            },
            Expr::DictLiteral(pairs, _) => DiscardInfo {
                value: !pairs.is_empty()
                    && pairs.iter().all(|(key, value)| {
                        self.discard_info(key).value && self.discard_info(value).value
                    }),
                ..Default::default()
            },
            Expr::RecordLiteral { fields, .. } => DiscardInfo {
                value: self.record_literal_is_task_free(fields, &ty),
                ..Default::default()
            },
            Expr::RecordUpdate { base, fields, .. } => DiscardInfo {
                value: self.discard_info(base).value
                    && fields
                        .iter()
                        .all(|field| self.discard_info(&field.value).value),
                ..Default::default()
            },
            Expr::Postfix {
                base, operations, ..
            } => {
                let mut info = self.discard_info(base);
                for operation in operations {
                    let operation_key = Self::postfix_operation_lookup_key(operation);
                    match operation {
                        PostfixOperation::Field { .. } | PostfixOperation::Index { .. } => {
                            info.result = false;
                        }
                        PostfixOperation::MethodCall { .. } => {
                            let target = self
                                .method_calls
                                .get(&operation_key)
                                .map(|target| target.qualified_name())
                                .or_else(|| {
                                    self.fluent_calls
                                        .get(&operation_key)
                                        .map(|target| target.name.as_str())
                                });
                            info = DiscardInfo {
                                value: target
                                    .is_some_and(|name| self.scopes.discard_info(name).result),
                                ..Default::default()
                            };
                        }
                    }
                }
                info
            }
            _ => DiscardInfo::default(),
        };
        info.value = match self.task_safety(&ty) {
            TaskSafety::Safe => true,
            TaskSafety::Captures => {
                !matches!(self.resolve_visible_type(&ty), Ty::Channel(_)) && info.value
            }
            TaskSafety::Forbidden(_) => false,
        };
        self.discard_exprs.insert(key, info);
    }

    fn record_literal_is_task_free(&self, fields: &[FieldInit], ty: &Ty) -> bool {
        let Ty::Record(record) = ty else {
            return false;
        };
        record
            .fields
            .iter()
            .all(|(name, field_ty)| match self.task_safety(field_ty) {
                TaskSafety::Safe => true,
                TaskSafety::Forbidden(_) => false,
                TaskSafety::Captures => fields
                    .iter()
                    .find(|field| field.name.eq_ignore_ascii_case(name))
                    .map(|field| self.discard_info(&field.value).value)
                    .unwrap_or_else(|| {
                        self.record_default_discard
                            .get(&(record.name.to_ascii_lowercase(), name.to_ascii_lowercase()))
                            .copied()
                            .unwrap_or(false)
                    }),
            })
    }

    fn designator_discard_info(&self, designator: &Designator) -> DiscardInfo {
        let full = Self::resolve_designator_name(designator);
        if let Some((scope, symbol)) = self.scopes.lookup_with_scope(&full) {
            let mut info = self.scopes.discard_info(&full);
            if scope == 0
                && matches!(
                    symbol.kind,
                    crate::scope::SymbolKind::Function | crate::scope::SymbolKind::Procedure
                )
            {
                info.value = true;
            }
            return info;
        }
        let Some(DesignatorPart::Ident(base, _)) = designator.parts.first() else {
            return DiscardInfo::default();
        };
        let mut info = self.scopes.discard_info(base);
        if self
            .property_reads
            .contains_key(&crate::designator_lookup_key(designator))
        {
            return DiscardInfo::default();
        }
        info.result = self
            .bound_methods
            .get(&crate::designator_lookup_key(designator))
            .is_some_and(|method| self.scopes.discard_info(&method.qualified_name).result);
        info
    }
}
