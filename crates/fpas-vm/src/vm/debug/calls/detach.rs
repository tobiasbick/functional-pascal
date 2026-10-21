//! Identity-aware deep detachment of mutable debugger values.

use std::collections::HashMap;
use std::sync::Arc;

use fpas_bytecode::{
    MemoryDomain, OwnedCell, RootKind, RootValues, SharedEnum, SharedFunction, SharedRecord, Value,
};

use crate::vm::debug::types::{DebugErrorKind, DebugSessionError};

pub(in crate::vm::debug) struct ValueDetacher {
    cells: HashMap<usize, OwnedCell>,
    pub(in crate::vm::debug) domain: MemoryDomain,
    sources: Vec<MemoryDomain>,
    values: usize,
    maximum: usize,
    observations: bool,
    depth: usize,
    roots: RootValues,
}

impl ValueDetacher {
    pub(in crate::vm::debug) fn new(maximum: usize) -> Self {
        let domain = crate::vm::domain::new_domain();
        let roots = domain.root_values(RootKind::Evaluation, Vec::new());
        Self {
            cells: HashMap::new(),
            domain,
            sources: Vec::new(),
            values: 0,
            maximum,
            observations: false,
            depth: 0,
            roots,
        }
    }

    /// Detach a non-executing observation, preserving opaque resource identities only as data.
    pub(in crate::vm::debug) fn observation(
        &mut self,
        value: &Value,
    ) -> Result<Value, DebugSessionError> {
        let previous = self.observations;
        self.observations = true;
        let result = self.detach(value);
        self.observations = previous;
        result
    }

    pub(in crate::vm::debug) fn allow_source(&mut self, domain: MemoryDomain) {
        if !self.sources.iter().any(|source| source.id() == domain.id()) {
            self.sources.push(domain);
        }
    }

    pub(in crate::vm::debug) fn detach(
        &mut self,
        value: &Value,
    ) -> Result<Value, DebugSessionError> {
        if self.depth >= 256 {
            return Err(error(
                DebugErrorKind::CallLimit,
                "debug detached graph exceeds nesting limit",
                "Inspect a shallower value.",
            ));
        }
        self.depth += 1;
        let result = self.detach_one(value);
        self.depth -= 1;
        result
    }

    fn detach_one(&mut self, value: &Value) -> Result<Value, DebugSessionError> {
        self.values = self.values.saturating_add(1);
        if self.values > self.maximum {
            return Err(error(
                DebugErrorKind::CallLimit,
                format!(
                    "debug call detached-value count exceeds limit {}",
                    self.maximum
                ),
                "Use smaller arguments or inspect the aggregate without calling code.",
            ));
        }
        match value {
            Value::Integer(value) => Ok(Value::Integer(*value)),
            Value::Real(value) => Ok(Value::Real(*value)),
            Value::Boolean(value) => Ok(Value::Boolean(*value)),
            Value::Str(value) => Ok(Value::Str(value.clone())),
            Value::Unit => Ok(Value::Unit),
            Value::Array(values) => Ok(Value::Array(
                values
                    .iter()
                    .map(|value| self.detach(value))
                    .collect::<Result<Vec<_>, _>>()?
                    .into(),
            )),
            Value::Dict(entries) => Ok(Value::dict(
                entries
                    .iter()
                    .map(|(key, value)| Ok((self.detach(key)?, self.detach(value)?)))
                    .collect::<Result<Vec<_>, DebugSessionError>>()?,
            )),
            Value::Record(record) => Ok(Value::Record(SharedRecord::new(
                Arc::clone(&record.body().layout),
                record
                    .body()
                    .values
                    .iter()
                    .map(|value| self.detach(value))
                    .collect::<Result<Vec<_>, _>>()?,
            ))),
            Value::Enum(enumeration) => Ok(Value::Enum(SharedEnum::new(
                Arc::clone(&enumeration.body().layout),
                enumeration
                    .body()
                    .values
                    .iter()
                    .map(|value| self.detach(value))
                    .collect::<Result<Vec<_>, _>>()?,
            ))),
            Value::ResultOk(value) => Ok(Value::result_ok(self.detach(value)?)),
            Value::ResultError(value) => Ok(Value::result_error(self.detach(value)?)),
            Value::OptionSome(value) => Ok(Value::option_some(self.detach(value)?)),
            Value::OptionNone => Ok(Value::OptionNone),
            Value::Function(function) => {
                if let Some(owner) = &function.owner_domain
                    && owner != self.domain.id()
                    && !self.sources.iter().any(|source| source.id() == owner)
                {
                    return Err(error(
                        DebugErrorKind::UnavailableValue,
                        "debug call cannot detach a function owned by another task",
                        "Select the stopped task that owns the function.",
                    ));
                }
                let bound_receiver = function
                    .bound_receiver
                    .as_ref()
                    .map(|receiver| self.detach(receiver))
                    .transpose()?;
                let captures = function
                    .captures
                    .iter()
                    .map(|value| self.detach(value))
                    .collect::<Result<Vec<_>, _>>()?;
                let detached = if let Some(receiver) = bound_receiver {
                    SharedFunction::bound(function.target().clone(), receiver)
                } else if function.task_bound {
                    SharedFunction::task_owned(
                        function.target().clone(),
                        captures,
                        self.domain.id().clone(),
                    )
                } else {
                    SharedFunction::unbound(function.target().clone(), captures)
                };
                Ok(Value::Function(
                    detached.with_display_name(Arc::clone(&function.name)),
                ))
            }
            Value::Cell(cell) => self.detach_cell(cell),
            Value::Reference(_) => Err(error(
                DebugErrorKind::UnavailableValue,
                "debug calls cannot detach `var` parameter references",
                "Pass the variable's current value instead.",
            )),
            Value::Task(_) | Value::OpaqueHandle(_) if self.observations => Ok(value.clone()),
            Value::Task(_) => Err(error(
                DebugErrorKind::UnavailableValue,
                "debug calls cannot detach task handles",
                "Pass completed task results, not task handles, to debugger calls.",
            )),
            Value::OpaqueHandle(_) => Err(error(
                DebugErrorKind::UnavailableValue,
                "debug calls cannot detach opaque host handles",
                "Inspect the handle without passing it to debugger-side code.",
            )),
        }
    }

    fn detach_cell(&mut self, cell: &OwnedCell) -> Result<Value, DebugSessionError> {
        if cell.owner() == self.domain.id() {
            return Ok(Value::Cell(cell.clone()));
        }
        let identity = cell.identity();
        if let Some(detached) = self.cells.get(&identity) {
            return Ok(Value::Cell(detached.clone()));
        }
        let source = self
            .sources
            .iter()
            .find(|domain| domain.id() == cell.owner())
            .ok_or_else(|| {
                error(
                    DebugErrorKind::UnavailableValue,
                    "debug call cannot access a foreign mutable cell",
                    "Use values owned by the selected stopped task.",
                )
            })?;
        let _access = source.access();
        let source_value = cell.try_read_snapshot(source).map_err(|_| {
            error(
                DebugErrorKind::UnavailableValue,
                "debug call mutable cell is unavailable",
                "Retry at a stable stop with the owning task selected.",
            )
        })?;
        let detached = self.domain.new_cell(Value::Unit).map_err(|_| {
            error(
                DebugErrorKind::UnavailableValue,
                "debug call cannot create detached storage",
                "Retry at a stable stop.",
            )
        })?;
        self.cells.insert(identity, detached.clone());
        self.roots.push(Value::Cell(detached.clone()));
        let inner = self.detach(&source_value)?;
        detached.replace(&self.domain, inner).map_err(|_| {
            error(
                DebugErrorKind::UnavailableValue,
                "debug call detached graph violates ownership",
                "Use values owned by the selected stopped task.",
            )
        })?;
        Ok(Value::Cell(detached))
    }
}

pub(in crate::vm::debug) fn error(
    kind: DebugErrorKind,
    message: impl Into<String>,
    hint: impl Into<String>,
) -> DebugSessionError {
    DebugSessionError {
        kind,
        message: message.into(),
        hint: hint.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_cells_detach_once_without_aliasing_live_state() {
        let source = MemoryDomain::new();
        let live = source.new_cell(Value::Integer(7)).expect("live cell");
        let value = Value::Array(vec![Value::Cell(live.clone()), Value::Cell(live.clone())].into());
        let mut detacher = ValueDetacher::new(16);
        detacher.allow_source(source.clone());
        let Value::Array(detached) = detacher.detach(&value).expect("detach array") else {
            panic!("detached array expected");
        };
        let [Value::Cell(first), Value::Cell(second)] = detached.as_slice() else {
            panic!("detached cells expected");
        };

        assert!(first.ptr_eq(second));
        assert!(!first.ptr_eq(&live));
        first
            .replace(&detacher.domain, Value::Integer(8))
            .expect("detached write");
        assert_eq!(live.read(&source).expect("live read"), Value::Integer(7));
        assert_eq!(
            second.read(&detacher.domain).expect("shared read"),
            Value::Integer(8)
        );
    }

    #[test]
    fn cyclic_cells_retain_the_cycle_inside_the_detached_graph() {
        let source = MemoryDomain::new();
        let live = source.new_cell(Value::Unit).expect("live cell");
        live.replace(&source, Value::Cell(live.clone()))
            .expect("create cycle");
        let mut detacher = ValueDetacher::new(16);
        detacher.allow_source(source.clone());
        let Value::Cell(detached) = detacher
            .detach(&Value::Cell(live.clone()))
            .expect("detach cycle")
        else {
            panic!("detached cell expected");
        };
        assert!(!detached.ptr_eq(&live));
        let inner = detached.read(&detacher.domain).expect("detached read");
        let Value::Cell(back_edge) = &inner else {
            panic!("detached cycle back edge expected");
        };
        assert!(detached.ptr_eq(back_edge));
        live.replace(&source, Value::Unit)
            .expect("break live cycle");
        detached
            .replace(&detacher.domain, Value::Unit)
            .expect("break detached cycle");
    }

    #[test]
    fn task_bound_functions_remain_owned_by_the_detached_sandbox() {
        let source = MemoryDomain::new();
        let live = source.new_cell(Value::Integer(7)).expect("live cell");
        let (program, errors) = fpas_parser::parse("program Work; begin end.");
        assert!(errors.is_empty());
        let executable = fpas_compiler::compile(&program).expect("callback image");
        let target = executable
            .function_target(executable.executable().entry)
            .expect("entry");
        let value = Value::Function(
            SharedFunction::task_owned(
                target.clone(),
                vec![Value::Cell(live.clone())],
                source.id().clone(),
            )
            .with_display_name("Work.Capture"),
        );
        let mut detacher = ValueDetacher::new(16);
        detacher.allow_source(source);
        let Value::Function(detached) =
            detacher.detach(&value).expect("detach task-bound function")
        else {
            panic!("detached function expected");
        };
        assert!(detached.task_bound);
        assert_eq!(detached.owner_domain.as_ref(), Some(detacher.domain.id()));
        assert_eq!(detached.target(), &target);
        assert_eq!(detached.name.as_ref(), "Work.Capture");
        let [Value::Cell(detached_cell)] = detached.captures.as_slice() else {
            panic!("detached cell capture expected");
        };
        assert!(!detached_cell.ptr_eq(&live));
    }

    #[test]
    fn task_bound_function_without_cells_requires_explicit_source_authority() {
        let (program, errors) = fpas_parser::parse("program Work; begin end.");
        assert!(errors.is_empty());
        let executable = fpas_compiler::compile(&program).expect("callback image");
        let target = executable
            .function_target(executable.executable().entry)
            .expect("entry");
        let source = MemoryDomain::new();
        let value = Value::task_owned_function(target, Vec::new(), source.id().clone());
        let mut detacher = ValueDetacher::new(16);
        let error = detacher.detach(&value).expect_err("foreign callable");
        assert_eq!(error.kind, DebugErrorKind::UnavailableValue);
        detacher.allow_source(source);
        let Value::Function(detached) = detacher.detach(&value).expect("authorized callable")
        else {
            panic!("detached function expected");
        };
        assert_eq!(detached.owner_domain.as_ref(), Some(detacher.domain.id()));
    }
}

#[cfg(test)]
mod root_tests {
    use super::*;

    #[test]
    fn abandoned_partial_detachment_releases_registrations_and_copied_cells() {
        let source = MemoryDomain::new();
        let live = source.new_cell(Value::Integer(7)).expect("cell");
        let mut detacher = ValueDetacher::new(16);
        detacher.allow_source(source.clone());
        let domain = detacher.domain.clone();
        let input = Value::Array(vec![Value::Cell(live.clone()), Value::OpaqueHandle(1)].into());
        assert!(detacher.detach(&input).is_err());
        let weak = detacher
            .cells
            .values()
            .next()
            .expect("partial cell")
            .downgrade();
        assert_eq!(domain.root_count(), 1);
        drop(detacher);
        assert_eq!(domain.root_count(), 0);
        assert!(weak.upgrade().is_none());
        assert_eq!(live.read(&source).expect("live cell"), Value::Integer(7));
    }

    #[test]
    fn nesting_limit_is_a_recoverable_error_and_drops_partial_roots() {
        let mut value = Value::Integer(1);
        for _ in 0..1000 {
            value = Value::option_some(value);
        }
        let mut detacher = ValueDetacher::new(4096);
        assert!(detacher.detach(&value).is_err());
        let domain = detacher.domain.clone();
        drop(detacher);
        assert_eq!(domain.root_count(), 0);
        fpas_bytecode::release_values(vec![value]);
    }
}
