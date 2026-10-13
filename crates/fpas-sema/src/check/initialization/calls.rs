//! Propagate callable dependencies through checked call targets and argument order.
//! **Documentation:** `docs/pascal/language/types/declaration-order.md`.

use super::identity;
use super::walker::{Origin, Value, Walker};
use crate::check::MethodCallTarget;
use crate::scope::SymbolKind;
use fpas_lexer::Span;
use fpas_parser::{Designator, Expr};

impl Walker<'_, '_> {
    /// Evaluate receiver/arguments in written order and use the checked call target.
    pub fn call(
        &mut self,
        key: usize,
        designator: &Designator,
        args: &[Expr],
        span: Span,
    ) -> Value {
        let receiver = self.designator(designator, false);
        let mut values: Vec<_> = args
            .iter()
            .map(|arg| self.expr(arg.argument_value()))
            .collect();
        self.order_arguments(args, &mut values);
        self.invalidate_var_arguments(args);
        let previous = self.worker;
        if self.worker_start == Some(key) {
            self.worker = true;
        }
        let result = self.call_target(key, designator, args, span, receiver, values);
        self.worker = previous;
        result
    }

    fn call_target(
        &mut self,
        key: usize,
        designator: &Designator,
        args: &[Expr],
        span: Span,
        receiver: Value,
        mut values: Vec<Value>,
    ) -> Value {
        if self.checker.record_constructions.contains(&key) {
            self.defaults(key, designator, args, span);
            return values.into_iter().flatten().collect();
        }
        if self.checker.enum_constructions.contains(&key)
            || self.checker.distinct_conversions.contains(&key)
            || self.checker.subrange_conversions.contains_key(&key)
        {
            return values.into_iter().flatten().collect();
        }
        if let Some(target) = self.checker.method_calls.get(&key) {
            if matches!(target, MethodCallTarget::Instance { .. }) {
                values.insert(0, receiver);
            }
            return self.method(target.qualified_name(), &values, span);
        }
        if let Some(target) = self.checker.fluent_calls.get(&key) {
            if crate::std_registry::native_operation_by_implementation(&target.name)
                .is_some_and(|operation| operation.receiver_mode() == crate::types::ParamMode::Var)
            {
                if let Some(binding) = self
                    .checker
                    .initialization
                    .references
                    .get(&identity(designator.span))
                    .and_then(|reference| reference.declaration)
                {
                    self.write(binding);
                }
            }
            return self.standard(&target.name, &values, receiver, span);
        }
        if let Some(target) = self.checker.intrinsic_calls.get(&key) {
            return self.standard(target, &values, receiver, span);
        }
        let Some(reference) = self
            .checker
            .initialization
            .references
            .get(&identity(designator.span))
        else {
            self.unknown(span);
            return [Origin::Unknown].into_iter().collect();
        };
        if reference.declaration.is_none()
            && matches!(reference.kind, SymbolKind::Function | SymbolKind::Procedure)
        {
            return self.imported(&reference.name, &values, span);
        }
        if reference.kind == SymbolKind::BuiltinStd {
            return self.standard(&reference.name, &values, receiver, span);
        }
        self.invoke(&receiver, &values, span)
    }

    /// Apply a local or imported record routine's dependency summary.
    pub fn method(&mut self, name: &str, values: &[Value], span: Span) -> Value {
        if let Some(&routine) = self.model.methods.get(&name.to_ascii_lowercase()) {
            self.invoke(
                &[Origin::Routine(routine)].into_iter().collect(),
                values,
                span,
            )
        } else {
            self.imported(name, values, span)
        }
    }

    /// Substitute callable arguments into every possible target's dependencies.
    pub fn invoke(&mut self, origins: &Value, values: &[Value], span: Span) -> Value {
        let mut returned = Value::new();
        if origins.is_empty() {
            self.unknown(span);
            returned.insert(Origin::Unknown);
        }
        for origin in origins {
            if !self.invoking.insert(*origin) {
                self.unknown(span);
                returned.insert(Origin::Unknown);
                continue;
            }
            match *origin {
                Origin::Routine(owner) => {
                    let Some(summary) = self.summaries.get(&owner) else {
                        self.unknown(span);
                        self.invoking.remove(origin);
                        continue;
                    };
                    self.worker_globals(summary.mutable_globals.iter().cloned(), span);
                    for &binding in &summary.started_bindings {
                        self.worker_binding(binding, span);
                    }
                    for &parameter in &summary.started_parameters {
                        self.invoke_worker(
                            values.get(parameter).unwrap_or(&Value::new()),
                            &[],
                            span,
                        );
                    }
                    if summary.unknown_worker {
                        self.worker_unknown(span);
                    }
                    for &binding in &summary.dependencies {
                        self.dependency(binding, span, "routine call");
                    }
                    for &binding in &summary.writes {
                        self.write(binding);
                    }
                    for &binding in &summary.called_bindings {
                        if self
                            .model
                            .bindings
                            .get(&binding)
                            .is_some_and(|binding| binding.owner == self.owner)
                        {
                            let callable = self.binding_value(binding);
                            self.invoke(&callable, &[], span);
                        } else if self.owner == super::collection::Owner::Root {
                            self.unknown(span);
                        } else {
                            self.summary.called_bindings.insert(binding);
                        }
                    }
                    for &parameter in &summary.called_parameters {
                        self.invoke(values.get(parameter).unwrap_or(&Value::new()), &[], span);
                    }
                    if summary.unknown {
                        self.unknown(span);
                    }
                    for &origin in &summary.returned {
                        match origin {
                            Origin::Parameter(index) => returned.extend(
                                values
                                    .get(index)
                                    .cloned()
                                    .unwrap_or_else(|| [Origin::Unknown].into_iter().collect()),
                            ),
                            other => {
                                returned.insert(other);
                            }
                        }
                    }
                }
                Origin::Parameter(index) => {
                    self.summary.called_parameters.insert(index);
                    if self.worker {
                        self.summary.started_parameters.insert(index);
                    }
                    returned.insert(Origin::Unknown);
                }
                Origin::Imported(reference) => {
                    if let Some(reference) = self.checker.initialization.references.get(&reference)
                    {
                        returned.extend(self.imported(&reference.name, values, span));
                    } else {
                        self.unknown(span);
                    }
                }
                Origin::BoundMethod(key, site) => {
                    if let Some(method) = self.checker.bound_methods.get(&key) {
                        let receiver = self
                            .checker
                            .initialization
                            .references
                            .get(&site)
                            .and_then(|reference| reference.declaration)
                            .map(|binding| self.binding_value(binding))
                            .unwrap_or_default();
                        let mut arguments = vec![receiver];
                        arguments.extend_from_slice(values);
                        returned.extend(self.method(&method.qualified_name, &arguments, span));
                    } else {
                        self.unknown(span);
                    }
                }
                Origin::Binding(binding) => {
                    self.dependency(binding, span, "callable access");
                    if self
                        .model
                        .bindings
                        .get(&binding)
                        .is_some_and(|binding| binding.owner == self.owner)
                    {
                        self.unknown(span);
                    } else {
                        self.summary.called_bindings.insert(binding);
                        if self.worker {
                            self.summary.started_bindings.insert(binding);
                        }
                    }
                    returned.insert(Origin::Unknown);
                }
                Origin::Unknown => {
                    self.unknown(span);
                    returned.insert(Origin::Unknown);
                }
            }
            self.invoking.remove(origin);
        }
        returned
    }

    /// Invalidate storage passed by reference after evaluating all call arguments.
    pub fn invalidate_var_arguments(&mut self, args: &[Expr]) {
        for arg in args {
            if let Expr::VarArgument { designator, .. } = arg.argument_value() {
                if let Some(binding) = self
                    .checker
                    .initialization
                    .references
                    .get(&identity(designator.span))
                    .and_then(|reference| reference.declaration)
                {
                    self.write(binding);
                }
            }
        }
    }

    /// Map already evaluated written arguments to their checked parameter positions.
    pub fn order_arguments(&self, args: &[Expr], values: &mut Vec<Value>) {
        if let Some(first) = args.first() {
            if let Some(order) = self
                .checker
                .named_argument_orders
                .get(&crate::expr_lookup_key(first))
            {
                let written = values.clone();
                *values = order
                    .iter()
                    .map(|&index| written.get(index).cloned().unwrap_or_default())
                    .collect();
            }
        }
    }

    fn defaults(&mut self, key: usize, designator: &Designator, args: &[Expr], span: Span) {
        let record = self
            .checker
            .expr_types
            .get(&key)
            .and_then(|ty| match ty {
                crate::types::Ty::Record(record) => Some(record.name.as_str()),
                _ => None,
            })
            .unwrap_or_else(|| {
                self.checker
                    .initialization
                    .references
                    .get(&identity(designator.span))
                    .map(|reference| reference.name.as_str())
                    .unwrap_or("")
            });
        let defaults = self
            .checker
            .record_defaults
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(record))
            .map(|(_, fields)| fields);
        if let Some(defaults) = defaults {
            for (name, value) in defaults {
                if args.iter().any(|arg| {
                    arg.argument_name()
                        .is_some_and(|label| label.eq_ignore_ascii_case(name))
                }) {
                    continue;
                }
                if let Some(value) = value {
                    let before = self.errors.len();
                    self.expr(value);
                    for error in &mut self.errors[before..] {
                        error.span = span;
                    }
                }
            }
        }
    }
}
