//! Call, callable-value and callback sites use their original checked AST allocations.

use super::*;
use fpas_parser::{Designator, Expr, PostfixOperation};

impl Walker<'_> {
    pub(super) fn ordinary(
        &mut self,
        expression: &Expr,
        owner: &Arc<SourceCallable>,
    ) -> Result<(), SourceQueryError> {
        self.expr(expression, owner, InvocationKind::Ordinary, None)
    }
    pub(super) fn expr(
        &mut self,
        expression: &Expr,
        owner: &Arc<SourceCallable>,
        invocation: InvocationKind,
        callback: Option<&Callback>,
    ) -> Result<(), SourceQueryError> {
        self.cancellation.check()?;
        let key = fpas_sema::expr_lookup_key(expression);
        let ty = self.metadata.expr_types.get(&key);
        // A returned callable has no proven target, even when the producing call is direct.
        if callback.is_some()
            && !matches!(
                expression,
                Expr::Designator(_) | Expr::Closure(_) | Expr::Paren(..) | Expr::Postfix { .. }
            )
            && let Some(resolved) = resolution::value(
                self.index,
                self.metadata,
                self.analysis.snapshot().path(),
                key,
                None,
                ty,
            )
        {
            self.emit(
                owner,
                expression.span(),
                CallRelationKind::CallbackArgument,
                &resolved,
                CallSyntax::value(),
                callback,
            )?;
        }
        match expression {
            Expr::Designator(designator) => {
                self.indices(designator, owner)?;
                if let Some(resolved) = resolution::value(
                    self.index,
                    self.metadata,
                    self.analysis.snapshot().path(),
                    fpas_sema::designator_lookup_key(designator),
                    Some(designator),
                    ty,
                ) {
                    self.emit(
                        owner,
                        designator.span,
                        value_kind(callback),
                        &resolved,
                        CallSyntax::value(),
                        callback,
                    )?;
                }
            }
            Expr::Call {
                designator,
                args,
                span,
            } => {
                self.indices(designator, owner)?;
                self.call(key, Some(designator), args, *span, owner, invocation)?;
            }
            Expr::Go(inner, _) => self.expr(inner, owner, InvocationKind::Spawn, callback)?,
            Expr::Paren(inner, _) => self.expr(inner, owner, invocation, callback)?,
            Expr::UnaryOp { operand, .. }
            | Expr::ResultOk(operand, _)
            | Expr::ResultError(operand, _)
            | Expr::OptionSome(operand, _)
            | Expr::Try(operand, _)
            | Expr::Is { value: operand, .. }
            | Expr::NamedArgument { value: operand, .. } => self.ordinary(operand, owner)?,
            Expr::BinaryOp { left, right, .. } => {
                self.ordinary(left, owner)?;
                self.ordinary(right, owner)?;
            }
            Expr::ArrayLiteral(values, _) => {
                for value in values {
                    self.ordinary(value, owner)?;
                }
            }
            Expr::DictLiteral(entries, _) => {
                for (key, value) in entries {
                    self.ordinary(key, owner)?;
                    self.ordinary(value, owner)?;
                }
            }
            Expr::RecordUpdate { base, fields, .. } => {
                self.ordinary(base, owner)?;
                for field in fields {
                    self.ordinary(&field.value, owner)?;
                }
            }
            Expr::Postfix {
                base, operations, ..
            } => {
                self.ordinary(base, owner)?;
                for (position, operation) in operations.iter().enumerate() {
                    let key = fpas_sema::postfix_operation_lookup_key(operation);
                    match operation {
                        PostfixOperation::Index { index, .. } => self.ordinary(index, owner)?,
                        PostfixOperation::MethodCall { args, span, .. } => self.call(
                            key,
                            None,
                            args,
                            *span,
                            owner,
                            if position + 1 == operations.len() {
                                invocation
                            } else {
                                InvocationKind::Ordinary
                            },
                        )?,
                        PostfixOperation::Field { span, .. } => {
                            let ty = self
                                .metadata
                                .postfix_types
                                .get(&(span.source_id, span.offset));
                            if let Some(resolved) = resolution::value(
                                self.index,
                                self.metadata,
                                self.analysis.snapshot().path(),
                                key,
                                None,
                                ty,
                            ) {
                                let callback = (position + 1 == operations.len())
                                    .then_some(callback)
                                    .flatten();
                                self.emit(
                                    owner,
                                    *span,
                                    value_kind(callback),
                                    &resolved,
                                    CallSyntax::value(),
                                    callback,
                                )?;
                            }
                        }
                    }
                }
            }
            Expr::Closure(closure) => {
                let containing = self.index.anonymous(
                    self.analysis,
                    owner,
                    CallableKind::Closure,
                    closure.span,
                    &format!("<closure@{}>", closure.span.offset),
                );
                (self.owner)(&containing);
                let resolved = resolution::Resolved {
                    target: Some(Arc::clone(&containing)),
                    signature: ty.cloned(),
                    receiver: false,
                    excluded: false,
                    reason: None,
                };
                self.emit(
                    owner,
                    closure.span,
                    value_kind(callback),
                    &resolved,
                    CallSyntax::value(),
                    callback,
                )?;
                self.body(&closure.body, &containing)?;
            }
            Expr::VarArgument { designator, .. } => self.indices(designator, owner)?,
            Expr::If {
                branches,
                else_value,
                ..
            } => {
                for branch in branches {
                    self.ordinary(&branch.condition, owner)?;
                    self.ordinary(&branch.value, owner)?;
                }
                self.ordinary(else_value, owner)?;
            }
            Expr::Case {
                selector,
                arms,
                else_arm,
                ..
            } => {
                self.ordinary(selector, owner)?;
                for arm in arms {
                    self.labels(&arm.labels, owner)?;
                    if let Some(guard) = &arm.guard {
                        self.ordinary(guard, owner)?;
                    }
                    self.ordinary(&arm.value, owner)?;
                }
                if let Some(arm) = else_arm {
                    self.ordinary(&arm.value, owner)?;
                }
            }
            Expr::Error(_) => {
                self.reasons.insert("analysis_errors");
            }
            Expr::Integer(..)
            | Expr::Real(..)
            | Expr::Str(..)
            | Expr::Bool(..)
            | Expr::OptionNone(_) => {}
        }
        Ok(())
    }

    pub(super) fn call(
        &mut self,
        key: usize,
        designator: Option<&Designator>,
        args: &[Expr],
        span: Span,
        owner: &Arc<SourceCallable>,
        invocation: InvocationKind,
    ) -> Result<(), SourceQueryError> {
        let resolved = resolution::call(
            self.index,
            self.metadata,
            self.analysis.snapshot().path(),
            key,
            designator,
        );
        let (arguments, complete) = resolution::arguments(self.metadata, args, &resolved);
        if !resolved.excluded {
            let kind = if resolved.target.is_some() {
                CallRelationKind::DirectCall
            } else {
                CallRelationKind::DynamicCall
            };
            self.emit(
                owner,
                span,
                kind,
                &resolved,
                CallSyntax {
                    invocation,
                    arguments: arguments.clone(),
                    arguments_complete: complete,
                },
                None,
            )?;
        }
        for (argument, mapping) in args.iter().zip(arguments.iter()) {
            let callback = Callback {
                receiving: resolved.target.clone(),
                parameter: mapping.parameter_index,
            };
            self.expr(
                argument.argument_value(),
                owner,
                InvocationKind::Ordinary,
                Some(&callback),
            )?;
        }
        Ok(())
    }
}

fn value_kind(callback: Option<&Callback>) -> CallRelationKind {
    if callback.is_some() {
        CallRelationKind::CallbackArgument
    } else {
        CallRelationKind::CallableValue
    }
}
