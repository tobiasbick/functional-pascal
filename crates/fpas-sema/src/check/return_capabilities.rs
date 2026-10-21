//! Bounded result summaries for resolved local routines and immutable callable bindings.

use super::Checker;
use crate::{TransferCapability, scope::SymbolKind, types::Ty};
use fpas_parser::{Expr, FuncBody};
use std::collections::HashMap;

type DeclarationKey = (u32, usize);

#[derive(Default)]
pub(crate) struct RoutineResults {
    declarations: HashMap<DeclarationKey, TransferCapability>,
    bodies: HashMap<usize, TransferCapability>,
    active: Vec<ReturnFrame>,
}

struct ReturnFrame {
    declaration: Option<DeclarationKey>,
    body: usize,
    capability: Option<TransferCapability>,
}

impl Checker {
    /// Start a separate summary so nested declarations cannot contribute outer returns.
    pub(crate) fn begin_return_summary(&mut self, name: &str, body: &FuncBody) {
        let declaration = self.routine_result_declaration(name);
        if let Some(key) = declaration {
            // A recursive call does not reuse a summary from a previous flow pass.
            self.routine_results.declarations.remove(&key);
        }
        self.routine_results.active.push(ReturnFrame {
            declaration,
            body: std::ptr::from_ref(body) as usize,
            capability: None,
        });
    }

    /// Join every checked return, including returns revisited during loop analysis.
    pub(crate) fn observe_return_capability(&mut self, value: &Expr) {
        let capability = self.expr_capability(Self::expr_lookup_key(value));
        self.join_return_capability(capability);
    }

    /// A successful try continues locally; only its error payload can leave the routine.
    pub(crate) fn observe_try_return_capability(&mut self, value: &Expr, ty: &Ty) {
        let capability = match ty {
            Ty::Result(ok, error) => self.result_error_capability(value, ok, error),
            Ty::Option(_) => TransferCapability::Transferable,
            _ => return,
        };
        self.join_return_capability(capability);
    }

    fn result_error_capability(&self, value: &Expr, ok: &Ty, error: &Ty) -> TransferCapability {
        if !self.type_can_contain_callable(error) {
            return TransferCapability::Transferable;
        }
        match value {
            Expr::ResultError(payload, _) => self.expr_capability(Self::expr_lookup_key(payload)),
            Expr::ResultOk(_, _) => TransferCapability::Transferable,
            Expr::Paren(inner, _) | Expr::NamedArgument { value: inner, .. } => {
                self.result_error_capability(inner, ok, error)
            }
            Expr::If {
                branches,
                else_value,
                ..
            } => branches
                .iter()
                .map(|branch| &branch.value)
                .chain(std::iter::once(else_value.as_ref()))
                .fold(TransferCapability::Transferable, |state, value| {
                    state.join(self.result_error_capability(value, ok, error))
                }),
            Expr::Case { arms, else_arm, .. } => arms
                .iter()
                .map(|arm| &arm.value)
                .chain(else_arm.iter().map(|arm| &arm.value))
                .fold(TransferCapability::Transferable, |state, value| {
                    state.join(self.result_error_capability(value, ok, error))
                }),
            _ if !self.type_can_contain_callable(ok) => {
                self.expr_capability(Self::expr_lookup_key(value))
            }
            // A local success payload does not prove that the error payload is local.
            _ => TransferCapability::Unknown,
        }
    }

    fn join_return_capability(&mut self, capability: TransferCapability) {
        if let Some(frame) = self.routine_results.active.last_mut() {
            frame.capability = Some(
                frame
                    .capability
                    .map_or(capability, |previous| previous.join(capability)),
            );
        }
    }

    /// Save only the enclosing result accumulator while checking an unreachable tail.
    pub(crate) fn snapshot_return_capability(&self) -> Option<TransferCapability> {
        self.routine_results
            .active
            .last()
            .and_then(|frame| frame.capability)
    }

    /// Discard unreachable returns while preserving independently checked nested bodies.
    pub(crate) fn restore_return_capability(&mut self, capability: Option<TransferCapability>) {
        if let Some(frame) = self.routine_results.active.last_mut() {
            frame.capability = capability;
        }
    }

    pub(crate) fn finish_return_summary(&mut self) {
        let Some(frame) = self.routine_results.active.pop() else {
            return;
        };
        let capability = frame.capability.unwrap_or_default();
        self.routine_results.bodies.insert(frame.body, capability);
        if let Some(key) = frame.declaration {
            self.routine_results.declarations.insert(key, capability);
        }
    }

    /// Retain a known callable result only while the binding cannot be reassigned.
    pub(crate) fn record_callable_result_binding(
        &mut self,
        name: &str,
        value: &Expr,
        mutable: bool,
    ) {
        let Some(key) = self.routine_result_declaration(name) else {
            return;
        };
        if mutable {
            self.routine_results.declarations.remove(&key);
        } else {
            let capability = self.callable_result_capability(value);
            self.routine_results.declarations.insert(key, capability);
        }
    }

    /// Ordinary calls retain known result ownership without forbidding local factories.
    pub(crate) fn call_result_capability(&self, expr: &Expr) -> TransferCapability {
        let key = Self::expr_lookup_key(expr);
        if self
            .expr_types
            .get(&key)
            .is_some_and(|ty| !self.type_can_contain_callable(ty))
        {
            return TransferCapability::Transferable;
        }
        match expr {
            Expr::Call { designator, .. } => self
                .resolved_call_result_capability(key)
                .unwrap_or_else(|| {
                    self.named_result_capability(&Self::resolve_designator_name(designator))
                }),
            Expr::Postfix { operations, .. } => operations
                .last()
                .and_then(|operation| {
                    self.resolved_call_result_capability(Self::postfix_operation_lookup_key(
                        operation,
                    ))
                })
                .unwrap_or_default(),
            _ => TransferCapability::Unknown,
        }
    }

    /// Result evidence for work passed as a value to a group or supervised start.
    pub(crate) fn callable_result_capability(&self, expr: &Expr) -> TransferCapability {
        if let Some(ty) = self.expr_types.get(&Self::expr_lookup_key(expr)) {
            match self.resolve_visible_type(ty) {
                Ty::Procedure(_) => return TransferCapability::Transferable,
                Ty::Function(function)
                    if !self.type_can_contain_callable(&function.return_type) =>
                {
                    return TransferCapability::Transferable;
                }
                _ => {}
            }
        }
        match expr {
            Expr::Paren(inner, _) | Expr::NamedArgument { value: inner, .. } => {
                self.callable_result_capability(inner)
            }
            Expr::Closure(closure) => self
                .routine_results
                .bodies
                .get(&(std::ptr::from_ref(&closure.body) as usize))
                .copied()
                .unwrap_or_default(),
            Expr::Designator(designator) => self
                .bound_methods
                .get(&crate::designator_lookup_key(designator))
                .map_or_else(
                    || self.named_result_capability(&Self::resolve_designator_name(designator)),
                    |method| self.named_result_capability(&method.qualified_name),
                ),
            Expr::Postfix { operations, .. } => operations
                .last()
                .and_then(|operation| {
                    self.bound_methods
                        .get(&Self::postfix_operation_lookup_key(operation))
                })
                .map(|method| self.named_result_capability(&method.qualified_name))
                .unwrap_or_default(),
            _ => TransferCapability::Unknown,
        }
    }

    pub(crate) fn resolved_call_result_capability(&self, key: usize) -> Option<TransferCapability> {
        self.method_calls
            .get(&key)
            .map(|target| self.named_result_capability(target.qualified_name()))
            .or_else(|| {
                self.fluent_calls
                    .get(&key)
                    .map(|target| self.named_result_capability(&target.name))
            })
    }

    fn named_result_capability(&self, name: &str) -> TransferCapability {
        let Some((_, symbol, Some(declaration))) =
            self.scopes.lookup_with_scope_and_declaration(name)
        else {
            return TransferCapability::Unknown;
        };
        if symbol.mutable
            || !matches!(
                symbol.kind,
                SymbolKind::Function | SymbolKind::Procedure | SymbolKind::Const | SymbolKind::Var
            )
        {
            return TransferCapability::Unknown;
        }
        self.routine_results
            .declarations
            .get(&(declaration.source_id, declaration.offset))
            .copied()
            .unwrap_or_default()
    }

    fn routine_result_declaration(&self, name: &str) -> Option<DeclarationKey> {
        self.scopes
            .lookup_with_scope_and_declaration(name)
            .and_then(|(_, _, declaration)| declaration)
            .map(|span| (span.source_id, span.offset))
    }
}
