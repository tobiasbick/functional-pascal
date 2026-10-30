//! Transfer-capability propagation through typed value expressions.
//! See `docs/pascal/language/functions/closures.md`.

use super::Checker;
use crate::{
    TransferCapability,
    scope::SymbolKind,
    types::{Ty, TypeConstraint},
};
use fpas_parser::{Designator, DesignatorPart, Expr, PostfixOperation};

impl Checker {
    /// Recompute capability after expression typing, including repeated flow analysis.
    pub(in crate::check) fn propagate_task_bound_expr(&mut self, expr: &Expr, key: usize) {
        use TransferCapability::Transferable;
        let capability = match expr {
            Expr::Paren(inner, _)
            | Expr::ResultOk(inner, _)
            | Expr::ResultError(inner, _)
            | Expr::OptionSome(inner, _)
            | Expr::Try(inner, _)
            | Expr::NamedArgument { value: inner, .. } => {
                self.expr_capability(Self::expr_lookup_key(inner))
            }
            Expr::ArrayLiteral(values, _) => self.join_expr_capabilities(values.iter()),
            Expr::DictLiteral(pairs, _) => {
                self.join_expr_capabilities(pairs.iter().flat_map(|(key, value)| [key, value]))
            }
            Expr::If {
                branches,
                else_value,
                ..
            } => self.join_expr_capabilities(
                branches
                    .iter()
                    .map(|branch| &branch.value)
                    .chain(std::iter::once(else_value.as_ref())),
            ),
            Expr::Case { arms, else_arm, .. } => self.join_expr_capabilities(
                arms.iter()
                    .map(|arm| &arm.value)
                    .chain(else_arm.iter().map(|arm| &arm.value)),
            ),
            Expr::Call {
                designator, args, ..
            } if self.is_value_constructor(key, designator) => {
                let mut capability =
                    self.join_expr_capabilities(args.iter().map(Expr::argument_value));
                if let Some(Ty::Record(record)) = self.expr_types.get(&key)
                    && let Some(defaults) = self.record_defaults.get(&record.name)
                {
                    for (name, value) in defaults {
                        if !args.iter().any(|arg| {
                            arg.argument_name()
                                .is_some_and(|provided| provided.eq_ignore_ascii_case(name))
                        }) && let Some(value) = value
                        {
                            capability =
                                capability.join(self.expr_capability(Self::expr_lookup_key(value)));
                        }
                    }
                }
                capability
            }
            Expr::Call { .. } => self
                .native_capabilities
                .get(&key)
                .copied()
                .unwrap_or_else(|| self.call_result_capability(expr)),
            Expr::RecordUpdate { base, fields, .. } => self
                .expr_capability(Self::expr_lookup_key(base))
                .join(self.join_expr_capabilities(fields.iter().map(|field| &field.value))),
            Expr::Postfix {
                base, operations, ..
            } => self.postfix_prefix_capability(base, operations),
            Expr::Designator(designator) => self.designator_capability(designator),
            Expr::Closure(_) => self
                .closure_infos
                .get(&key)
                .map_or(TransferCapability::Unknown, |info| info.capability),
            Expr::OptionNone(_) => Transferable,
            _ => self
                .expr_types
                .get(&key)
                .map_or(TransferCapability::Unknown, |ty| {
                    self.capability_for_type(ty)
                }),
        };
        self.set_expr_capability(key, capability);
    }

    fn join_expr_capabilities<'a>(
        &self,
        values: impl Iterator<Item = &'a Expr>,
    ) -> TransferCapability {
        values.fold(TransferCapability::Transferable, |state, value| {
            state.join(self.expr_capability(Self::expr_lookup_key(value)))
        })
    }

    /// Recognize aggregate construction without treating ordinary call results as arguments.
    pub(in crate::check) fn is_value_constructor(
        &self,
        key: usize,
        designator: &Designator,
    ) -> bool {
        self.record_constructions.contains(&key)
            || self
                .scopes
                .lookup(&Self::resolve_designator_name(designator))
                .is_some_and(|symbol| {
                    matches!(
                        symbol.kind,
                        SymbolKind::EnumVariantConstructor | SymbolKind::EnumMember
                    )
                })
    }

    /// A callable signature alone gives no proof about the captured values.
    pub(crate) fn capability_for_type(&self, ty: &Ty) -> TransferCapability {
        if self.type_can_contain_callable(ty) {
            TransferCapability::Unknown
        } else {
            TransferCapability::Transferable
        }
    }

    /// A named callable, aggregate projection, or bound receiver preserves its source capability.
    pub(crate) fn designator_capability(&self, designator: &Designator) -> TransferCapability {
        if self
            .enum_members
            .contains(&crate::designator_lookup_key(designator))
        {
            return TransferCapability::Transferable;
        }
        let name = Self::resolve_designator_name(designator);
        let symbol = self.scopes.lookup(&name).or_else(|| {
            designator.parts.first().and_then(|part| match part {
                DesignatorPart::Ident(base, _) => self.scopes.lookup(base),
                _ => None,
            })
        });
        let mut capability = symbol.map_or(TransferCapability::Unknown, |symbol| symbol.capability);
        for part in &designator.parts {
            let span = match part {
                DesignatorPart::Ident(_, span) | DesignatorPart::Index(_, span) => span,
            };
            if self
                .path_types
                .get(&(span.source_id, span.offset))
                .is_some_and(|ty| !self.type_can_contain_callable(ty))
            {
                capability = TransferCapability::Transferable;
            }
        }
        capability
    }

    /// Capability of the receiver before a postfix call, independent from the call result.
    pub(crate) fn postfix_prefix_capability(
        &self,
        base: &Expr,
        operations: &[PostfixOperation],
    ) -> TransferCapability {
        let mut capability = self.expr_capability(Self::expr_lookup_key(base));
        for operation in operations {
            let (span, call) = match operation {
                PostfixOperation::Field { span, .. } | PostfixOperation::Index { span, .. } => {
                    (span, false)
                }
                PostfixOperation::MethodCall { span, .. } => (span, true),
            };
            let types = if call {
                &self.postfix_types
            } else {
                &self.path_types
            };
            if call {
                let key = Self::postfix_operation_lookup_key(operation);
                capability = if let PostfixOperation::MethodCall { args, .. } = operation
                    && let Some(native) = self.fluent_calls.get(&key).and_then(|target| {
                        crate::std_registry::native_operation_by_implementation(&target.name)
                    })
                    && let Some(result) = types.get(&(span.source_id, span.offset))
                {
                    self.native_result_capability(native, capability, args, result)
                } else {
                    self.native_capabilities
                        .get(&key)
                        .copied()
                        .or_else(|| self.resolved_call_result_capability(key))
                        .unwrap_or_default()
                };
            }
            if types
                .get(&(span.source_id, span.offset))
                .is_some_and(|ty| !self.type_can_contain_callable(ty))
            {
                capability = TransferCapability::Transferable;
            }
        }
        capability
    }

    /// Whether this value type can retain callable captures, including nested payloads.
    pub(crate) fn type_can_contain_callable(&self, ty: &Ty) -> bool {
        self.type_can_contain_callable_inner(ty, &mut std::collections::HashSet::new())
    }

    fn type_can_contain_callable_inner(
        &self,
        ty: &Ty,
        visited_types: &mut std::collections::HashSet<String>,
    ) -> bool {
        match self.resolve_visible_type(ty) {
            Ty::GenericParam(
                _,
                Some(
                    TypeConstraint::Numeric
                    | TypeConstraint::Comparable
                    | TypeConstraint::Equatable,
                ),
            ) => false,
            Ty::Function(_) | Ty::Procedure(_) | Ty::GenericParam(_, _) | Ty::Named(_) => true,
            Ty::Array(inner) | Ty::Option(inner) => {
                self.type_can_contain_callable_inner(&inner, visited_types)
            }
            Ty::Result(ok, error) | Ty::Dict(ok, error) => {
                self.type_can_contain_callable_inner(&ok, visited_types)
                    || self.type_can_contain_callable_inner(&error, visited_types)
            }
            Ty::Record(record) => {
                let identity = format!(
                    "record:{}",
                    Ty::Record(record.clone()).to_string().to_ascii_lowercase()
                );
                if !visited_types.insert(identity.clone()) {
                    return false;
                }
                let contains_callable = record
                    .fields
                    .iter()
                    .any(|(_, field)| self.type_can_contain_callable_inner(field, visited_types));
                visited_types.remove(&identity);
                contains_callable
            }
            Ty::Enum(enumeration) => {
                let resolved = self.resolve_visible_type(&Ty::Enum(enumeration.clone()));
                let Ty::Enum(enumeration) = resolved else {
                    return false;
                };
                let identity = format!(
                    "enum:{}",
                    Ty::Enum(enumeration.clone())
                        .to_string()
                        .to_ascii_lowercase()
                );
                if !visited_types.insert(identity.clone()) {
                    return false;
                }
                let contains_callable = enumeration.variants.iter().any(|variant| {
                    variant.fields.iter().any(|(_, field)| {
                        self.type_can_contain_callable_inner(field, visited_types)
                    })
                });
                visited_types.remove(&identity);
                contains_callable
            }
            Ty::Integer
            | Ty::Real
            | Ty::Boolean
            | Ty::String
            | Ty::ByteBuffer
            | Ty::Distinct(_)
            | Ty::Subrange(_)
            | Ty::Unit
            | Ty::Channel(_)
            | Ty::Task(_)
            | Ty::Error => false,
        }
    }
}
