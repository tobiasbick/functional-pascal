//! Call rules: constructions, standard operations, routines with inferred parameter escapes,
//! and calls through callable values.
//!
//! **Documentation:** `docs/pascal/language/concurrency/task-scopes.md#handle-escapes`.

use std::sync::Arc;

use fpas_lexer::Span;
use fpas_parser::{Designator, DesignatorPart, Expr};

use super::origins::{Owners, merge};
use super::walker::{Resolved, Walker};
use crate::check::{Checker, MethodCallTarget};
use crate::scope::canonical_symbol_name;
use crate::types::ParamMode;
use crate::types::Ty;

/// Parameter escapes of a resolved callee.
enum Callee {
    /// A routine of the analyzed unit.
    Routine(usize),
    /// An imported routine with its recorded parameter escapes.
    Imported(Arc<[bool]>),
    /// A call through a callable value: every argument may outlive the call.
    Value,
}

/// Standard channel operations that hand their value to another task.
const CHANNEL_SENDS: [&str; 5] = [
    "Std.Tasks.Send",
    "Std.Tasks.TrySend",
    "Std.Tasks.SendWithCancellation",
    "Std.Tasks.SendWithTimeout",
    "Std.Tasks.SendCase",
];

impl Walker<'_, '_> {
    /// Origins of a call written as a designator, such as `Helper(A)` or `Items.PushInPlace(A)`.
    pub(super) fn designator_call(
        &mut self,
        key: usize,
        designator: &Designator,
        args: &[Expr],
        span: Span,
    ) -> Owners {
        for part in &designator.parts {
            if let DesignatorPart::Index(index, _) = part {
                let _ = self.expr(index);
            }
        }
        let checker = self.checker;
        let base = designator.parts.first().and_then(|part| match part {
            DesignatorPart::Ident(name, _) => Some(name.as_str()),
            DesignatorPart::Index(..) => None,
        });
        if checker.record_constructions.contains(&key)
            || checker.enum_constructions.contains(&key)
            || checker.distinct_conversions.contains(&key)
            || checker.subrange_conversions.contains_key(&key)
        {
            return self.standard_call("", None, None, args, span);
        }
        if let Some(name) = checker.intrinsic_calls.get(&key) {
            return self.standard_call(name, None, None, args, span);
        }
        if let Some(target) = checker.fluent_calls.get(&key) {
            let receiver = base.map(|name| self.read(name)).unwrap_or_default();
            let writable = (crate::std_registry::intrinsic_std_receiver_mode(&target.name)
                == ParamMode::Var)
                .then_some(base)
                .flatten();
            return self.standard_call(&target.name, Some(receiver), writable, args, span);
        }
        if let Some(target) = checker.method_calls.get(&key) {
            let receiver = match target {
                MethodCallTarget::Instance { .. } => {
                    Some(base.map(|name| self.read(name)).unwrap_or_default())
                }
                MethodCallTarget::Static(_) => None,
            };
            let callee = self.method_callee(target.qualified_name());
            return self.routine_call(callee, target.qualified_name(), receiver, args, span);
        }
        if checker.member_value_calls.contains_key(&key) {
            let captures = base.map(|name| self.read(name));
            return self.routine_call(Callee::Value, "", captures, args, span);
        }
        let name = Checker::resolve_designator_name(designator);
        let callee = match (designator.parts.len(), base) {
            (1, Some(base)) => match self.resolve(base) {
                Resolved::Local(_) | Resolved::Outer => Callee::Value,
                Resolved::Routine(index) => Callee::Routine(index),
                Resolved::Global => self.imported_callee(&name),
            },
            _ => self.imported_callee(&name),
        };
        let captures = matches!(callee, Callee::Value)
            .then(|| base.map(|name| self.read(name)).unwrap_or_default());
        self.routine_call(callee, &name, captures, args, span)
    }

    /// Origins of a postfix `.Method(args)` call on an already computed receiver value.
    pub(super) fn receiver_call(
        &mut self,
        key: usize,
        receiver: Owners,
        args: &[Expr],
        span: Span,
    ) -> Owners {
        let checker = self.checker;
        if let Some(target) = checker.fluent_calls.get(&key) {
            return self.standard_call(&target.name, Some(receiver), None, args, span);
        }
        if let Some(target) = checker.method_calls.get(&key) {
            let callee = self.method_callee(target.qualified_name());
            let receiver = matches!(target, MethodCallTarget::Instance { .. }).then_some(receiver);
            return self.routine_call(callee, target.qualified_name(), receiver, args, span);
        }
        self.routine_call(Callee::Value, "", Some(receiver), args, span)
    }

    /// Constructions, conversions, and standard operations keep the origins of their inputs,
    /// except channel sends, which hand the value to another task.
    fn standard_call(
        &mut self,
        name: &str,
        receiver: Option<Owners>,
        writable_receiver: Option<&str>,
        args: &[Expr],
        span: Span,
    ) -> Owners {
        let mut owners = receiver.unwrap_or_default();
        let mut callback_inputs = owners.clone();
        let mut saw_callback = false;
        let mut safe_callbacks = true;
        let mut targets: Vec<String> = writable_receiver.map(str::to_string).into_iter().collect();
        for arg in args {
            match arg.argument_value() {
                Expr::VarArgument { designator, .. } => {
                    if let Some(DesignatorPart::Ident(name, _)) = designator.parts.first() {
                        let input = self.read(name);
                        merge(&mut callback_inputs, input.iter().copied());
                        merge(&mut owners, input);
                        targets.push(name.clone());
                    }
                }
                value => {
                    let input = self.expr(value);
                    let callable = matches!(
                        self.checker
                            .expr_types
                            .get(&Checker::expr_lookup_key(value)),
                        Some(Ty::Function(_) | Ty::Procedure(_))
                    );
                    if callable {
                        saw_callback = true;
                        let safe = self.callback_is_safe(value);
                        safe_callbacks &= safe;
                    }
                    merge(
                        &mut callback_inputs,
                        input.iter().copied().filter(|origin| {
                            !callable
                                || !matches!(
                                    origin,
                                    super::origins::Origin::Resource { .. }
                                        | super::origins::Origin::Dependency { .. }
                                        | super::origins::Origin::DependencyParameter(_)
                                        | super::origins::Origin::BorrowedCapture(_)
                                        | super::origins::Origin::CallableParameter(_)
                                )
                        }),
                    );
                    if name.starts_with("Std.Tasks.")
                        && matches!(
                            self.checker
                                .expr_types
                                .get(&Checker::expr_lookup_key(value)),
                            Some(Ty::Function(_) | Ty::Procedure(_))
                        )
                    {
                        self.escape_all(&input, span, &format!("a callback passed to `{name}`"));
                    }
                    merge(&mut owners, input);
                }
            }
        }
        if CHANNEL_SENDS
            .iter()
            .any(|send| send.eq_ignore_ascii_case(name))
        {
            self.escape_all(&owners, span, "a channel send");
            return Vec::new();
        }
        if self.spawning {
            let resources: Owners = owners
                .iter()
                .copied()
                .filter(|origin| {
                    matches!(
                        origin,
                        super::origins::Origin::Resource { .. }
                            | super::origins::Origin::Dependency { .. }
                            | super::origins::Origin::DependencyParameter(_)
                            | super::origins::Origin::Parameter(_)
                            | super::origins::Origin::CallableParameter(_)
                            | super::origins::Origin::BorrowedCapture(_)
                    )
                })
                .collect();
            self.escape_all(&resources, span, "a spawned task");
        }
        // Declared callback parameters distinguish invocation from ordinary value storage.
        if !owners.is_empty()
            && !name.starts_with("Std.Tasks.")
            && self.standard_invokes_callback(name)
            && (!saw_callback || !safe_callbacks)
        {
            self.escape_all(
                &callback_inputs,
                span,
                &format!("a callback invoked by `{name}`"),
            );
        }
        for target in targets {
            self.store(&target, owners.clone(), span);
        }
        if name.eq_ignore_ascii_case(fpas_builtins::std_symbols::STD_MCP_PUBLISH) {
            return owners
                .into_iter()
                .map(|origin| match origin {
                    super::origins::Origin::Resource { frame, span } => {
                        super::origins::Origin::Dependency { frame, span }
                    }
                    super::origins::Origin::Parameter(index)
                    | super::origins::Origin::BorrowedCapture(index) => {
                        super::origins::Origin::DependencyParameter(index)
                    }
                    other => other,
                })
                .collect();
        }
        if crate::types::native_resource_fresh_result(name) {
            return Vec::new();
        }
        owners
    }

    fn standard_invokes_callback(&self, name: &str) -> bool {
        let callable = |ty: &Ty| matches!(ty, Ty::Function(_) | Ty::Procedure(_));
        if let Some(operation) = crate::std_registry::native_operation_by_implementation(name) {
            return (operation.signature)()
                .params
                .iter()
                .any(|param| callable(&param.ty));
        }
        self.checker
            .scopes
            .lookup(name)
            .is_some_and(|symbol| match &symbol.ty {
                Ty::Function(function) => function.params.iter().any(|param| callable(&param.ty)),
                Ty::Procedure(procedure) => {
                    procedure.params.iter().any(|param| callable(&param.ty))
                }
                _ => false,
            })
    }

    pub(super) fn callback_is_safe(&self, value: &Expr) -> bool {
        match value {
            Expr::Designator(designator) => {
                if let [DesignatorPart::Ident(name, _)] = designator.parts.as_slice() {
                    match self.resolve(name) {
                        Resolved::Routine(index) => {
                            return self.flags[index].iter().all(|escapes| !escapes);
                        }
                        Resolved::Local(frame) => {
                            return self.frames[frame]
                                .safe_callbacks
                                .contains(&canonical_symbol_name(name));
                        }
                        Resolved::Outer => return false,
                        Resolved::Global => {}
                    }
                }
                let name = Checker::resolve_designator_name(designator);
                self.checker
                    .scopes
                    .param_escapes(&name)
                    .is_some_and(|flags| flags.iter().all(|escapes| !escapes))
            }
            Expr::Closure(closure) => {
                let mut callback =
                    Walker::new(self.checker, self.table, self.flags, &closure.params);
                callback.walk_body(&closure.body, None);
                callback.escapes.iter().all(|escapes| !escapes)
            }
            Expr::Paren(inner, _) => self.callback_is_safe(inner),
            _ => false,
        }
    }

    /// Arguments of a routine call may only carry tracked values to non-escaping parameters.
    fn routine_call(
        &mut self,
        callee: Callee,
        name: &str,
        receiver: Option<Owners>,
        args: &[Expr],
        span: Span,
    ) -> Owners {
        if let Callee::Routine(index) = &callee {
            let routine = &self.table.routines[*index];
            for (capture, name) in routine.captures.iter().enumerate() {
                let owners = self.read(name);
                self.argument_escape(
                    &callee,
                    &routine.name,
                    routine.params.len() + capture,
                    &owners,
                    span,
                );
            }
        }
        let offset = usize::from(receiver.is_some());
        if let Some(receiver) = receiver {
            if self.spawning {
                let resources: Owners = receiver
                    .iter()
                    .copied()
                    .filter(|origin| {
                        matches!(
                            origin,
                            super::origins::Origin::Resource { .. }
                                | super::origins::Origin::Dependency { .. }
                                | super::origins::Origin::DependencyParameter(_)
                                | super::origins::Origin::Parameter(_)
                                | super::origins::Origin::CallableParameter(_)
                                | super::origins::Origin::BorrowedCapture(_)
                        )
                    })
                    .collect();
                self.escape_all(&resources, span, "a spawned task");
            }
            if matches!(callee, Callee::Value) {
                let other: Owners = receiver
                    .iter()
                    .copied()
                    .filter(|origin| {
                        !matches!(
                            origin,
                            super::origins::Origin::Resource { .. }
                                | super::origins::Origin::Dependency { .. }
                                | super::origins::Origin::DependencyParameter(_)
                                | super::origins::Origin::BorrowedCapture(_)
                                | super::origins::Origin::CallableParameter(_)
                        )
                    })
                    .collect();
                self.argument_escape(&callee, name, 0, &other, span);
            } else {
                self.argument_escape(&callee, name, 0, &receiver, span);
            }
        }
        let order = args
            .first()
            .and_then(|first| {
                self.checker
                    .named_argument_orders
                    .get(&Checker::expr_lookup_key(first))
            })
            .cloned();
        for (written, arg) in args.iter().enumerate() {
            let parameter = order
                .as_ref()
                .and_then(|order| order.iter().position(|index| *index == written))
                .unwrap_or(written);
            let owners = match arg.argument_value() {
                Expr::VarArgument { designator, .. } => designator
                    .parts
                    .first()
                    .map(|part| match part {
                        DesignatorPart::Ident(name, _) => self.read(name),
                        DesignatorPart::Index(..) => Vec::new(),
                    })
                    .unwrap_or_default(),
                value => self.expr(value),
            };
            self.argument_escape(&callee, name, parameter + offset, &owners, arg.span());
        }
        Vec::new()
    }

    fn argument_escape(
        &mut self,
        callee: &Callee,
        name: &str,
        parameter: usize,
        owners: &Owners,
        span: Span,
    ) {
        if owners.is_empty() {
            return;
        }
        if self.spawning {
            let resources: Owners = owners
                .iter()
                .copied()
                .filter(|origin| {
                    matches!(
                        origin,
                        super::origins::Origin::Resource { .. }
                            | super::origins::Origin::Dependency { .. }
                            | super::origins::Origin::DependencyParameter(_)
                            | super::origins::Origin::Parameter(_)
                            | super::origins::Origin::CallableParameter(_)
                            | super::origins::Origin::BorrowedCapture(_)
                    )
                })
                .collect();
            self.escape_all(&resources, span, "a spawned task");
        }
        let (escapes, route) = match callee {
            Callee::Routine(index) => (
                self.flags[*index].get(parameter).copied().unwrap_or(true),
                format!(
                    "an argument of `{}` whose parameter can outlive the call",
                    self.table.routines[*index].name
                ),
            ),
            Callee::Imported(flags) => (
                flags.get(parameter).copied().unwrap_or(false),
                format!("an argument of `{name}` whose parameter can outlive the call"),
            ),
            Callee::Value => (
                true,
                "an argument of a call through a callable value".to_string(),
            ),
        };
        if escapes {
            self.escape_all(owners, span, &route);
        }
    }

    fn method_callee(&self, qualified: &str) -> Callee {
        self.table
            .methods
            .get(&canonical_symbol_name(qualified))
            .map_or_else(
                || self.imported_callee(qualified),
                |index| Callee::Routine(*index),
            )
    }

    fn imported_callee(&self, name: &str) -> Callee {
        self.checker
            .scopes
            .param_escapes(name)
            .map_or(Callee::Value, Callee::Imported)
    }
}
