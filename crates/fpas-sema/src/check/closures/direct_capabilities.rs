//! Direct capture facts, excluding named routine dependencies resolved later.
//! See `docs/pascal/language/functions/closures.md`.

use super::pending_capabilities::binding_key;
use crate::{TransferCapability, check::Checker};
use fpas_parser::{DesignatorPart, Expr, PostfixOperation};

impl Checker {
    /// Separate immutable initializer facts from recursive named-target capabilities.
    pub(super) fn direct_expr_capability(&self, expr: &Expr) -> TransferCapability {
        use TransferCapability::{TaskBound, Transferable, Unknown};
        let key = Self::expr_lookup_key(expr);
        if self
            .expr_types
            .get(&key)
            .is_some_and(|ty| !self.type_can_contain_callable(ty))
        {
            return Transferable;
        }
        match expr {
            Expr::Designator(designator) => {
                let full = Self::resolve_designator_name(designator);
                let name = if self.scopes.lookup(&full).is_some() {
                    full.as_str()
                } else if let Some(DesignatorPart::Ident(root, _)) = designator.parts.first() {
                    root.as_str()
                } else {
                    return Unknown;
                };
                if self.scopes.routine_capture_key(name).is_some() {
                    return Transferable;
                }
                if self
                    .scopes
                    .lookup(name)
                    .is_some_and(|symbol| symbol.mutable)
                {
                    // Mutable storage is governed by current flow facts, not its initializer.
                    return self.expr_capability(key);
                }
                self.scopes
                    .lookup_with_scope_and_declaration(name)
                    .and_then(|(_, _, span)| span)
                    .and_then(|span| {
                        self.capture_dependencies
                            .capabilities
                            .direct
                            .get(&binding_key(name, span))
                    })
                    .copied()
                    .unwrap_or_else(|| self.expr_capability(key))
            }
            Expr::Closure(_) => self.closure_infos.get(&key).map_or(Unknown, |info| {
                info.captures.iter().fold(Transferable, |state, capture| {
                    state.join(if capture.mutable {
                        TaskBound
                    } else {
                        self.capture_dependencies
                            .capabilities
                            .direct
                            .get(&binding_key(&capture.name, capture.declaration))
                            .copied()
                            .unwrap_or(capture.capability)
                    })
                })
            }),
            Expr::Paren(inner, _)
            | Expr::ResultOk(inner, _)
            | Expr::ResultError(inner, _)
            | Expr::OptionSome(inner, _)
            | Expr::Try(inner, _)
            | Expr::NamedArgument { value: inner, .. } => self.direct_expr_capability(inner),
            Expr::Postfix {
                base, operations, ..
            } if !operations
                .iter()
                .any(|operation| matches!(operation, PostfixOperation::MethodCall { .. })) =>
            {
                self.direct_expr_capability(base)
            }
            Expr::ArrayLiteral(values, _) => self.join_direct_capabilities(values.iter()),
            Expr::DictLiteral(pairs, _) => {
                self.join_direct_capabilities(pairs.iter().flat_map(|(key, value)| [key, value]))
            }
            Expr::If {
                branches,
                else_value,
                ..
            } => self.join_direct_capabilities(
                branches
                    .iter()
                    .map(|branch| &branch.value)
                    .chain(std::iter::once(else_value.as_ref())),
            ),
            Expr::Case { arms, else_arm, .. } => self.join_direct_capabilities(
                arms.iter()
                    .map(|arm| &arm.value)
                    .chain(else_arm.iter().map(|arm| &arm.value)),
            ),
            Expr::RecordUpdate { base, fields, .. } => self
                .direct_expr_capability(base)
                .join(self.join_direct_capabilities(fields.iter().map(|field| &field.value))),
            Expr::Call {
                designator, args, ..
            } if self.is_value_constructor(key, designator) => {
                let mut result =
                    self.join_direct_capabilities(args.iter().map(Expr::argument_value));
                if let Some(crate::types::Ty::Record(record)) = self.expr_types.get(&key)
                    && let Some(defaults) = self.record_defaults.get(&record.name)
                {
                    for (name, value) in defaults {
                        if !args.iter().any(|arg| {
                            arg.argument_name()
                                .is_some_and(|provided| provided.eq_ignore_ascii_case(name))
                        }) && let Some(value) = value
                        {
                            result = result.join(self.direct_expr_capability(value));
                        }
                    }
                }
                result
            }
            _ => self.expr_capability(key),
        }
    }

    fn join_direct_capabilities<'a>(
        &self,
        values: impl Iterator<Item = &'a Expr>,
    ) -> TransferCapability {
        values.fold(TransferCapability::Transferable, |state, value| {
            state.join(self.direct_expr_capability(value))
        })
    }
}
