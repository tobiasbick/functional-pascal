//! Instruction-boundary observation of exact source bindings and live scalar aliases.

use super::{
    DebugTraceIdentity, LocalTraceState,
    local_state::{InheritedLocal, LocalFrame, ParameterValue},
};
use crate::execution::vm::{inspection::location, worker::Worker};
use crate::{DebugBindingKind, FunctionId, InstructionAddress, Value};

impl Worker {
    pub(in crate::execution::vm) fn trace_local_begin(&mut self) {
        self.trace_local_begin_at(None);
    }

    pub(in crate::execution::vm) fn trace_local_resume_begin(&mut self) {
        self.trace_local_begin_at(Some((self.function, self.current_address)));
    }

    fn trace_local_begin_at(&mut self, scope_site: Option<(FunctionId, InstructionAddress)>) {
        let Some(mut state) = self.debug_local_trace.take() else {
            return;
        };
        state.commit = state
            .history
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .commit_id();
        state.sync(self);
        state.prune(self, scope_site);
        state
            .history
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .flush(state.commit, self.executable.executable());
        state.commit = state
            .history
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .commit_id();
        self.debug_local_trace = Some(state);
    }

    pub(in crate::execution::vm) fn trace_local_finish(
        &mut self,
        owner: FunctionId,
        address: InstructionAddress,
        success: bool,
        terminal: bool,
        tail: bool,
    ) {
        let Some(mut state) = self.debug_local_trace.take() else {
            return;
        };
        let depth = self.call_stack.len() + 1;
        if terminal {
            state.retire_from(0);
            state.ended = true;
        } else if tail && state.frames.len() == depth {
            state.retire_from(depth.saturating_sub(1));
        } else if state.frames.len() > depth {
            state.retire_from(depth);
        }
        if success && !tail && !terminal {
            state.initialize(self, owner, address);
        }
        state.observe(self, owner, address);
        state
            .history
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .flush(state.commit, self.executable.executable());
        if terminal {
            state.inherited.clear();
        }
        if !terminal {
            state.commit = state
                .history
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .commit_id();
            state.sync(self);
            state
                .history
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .flush(state.commit, self.executable.executable());
        }
        self.debug_local_trace = Some(state);
    }

    pub(in crate::execution::vm) fn trace_callback_state(&self) -> Option<Box<LocalTraceState>> {
        let parent = self.debug_local_trace.as_ref()?;
        let mut child = parent.for_task();
        child.inherited = parent.inherited.clone();
        for frame in &parent.frames {
            let info = &self.executable.executable().functions[frame.function.get() as usize];
            for live in &frame.live {
                let binding = &info.debug.bindings[live.binding];
                let register = frame.base + usize::from(binding.register.get());
                let source = match &live.parameter {
                    Some(ParameterValue::Alias(value)) => Some(value),
                    Some(ParameterValue::Frozen(_)) => None,
                    None => self.registers.get(register),
                };
                if let Some(storage @ (Value::Cell(_) | Value::Reference(_))) = source
                    && !child
                        .inherited
                        .iter()
                        .any(|prior| prior.instance == live.instance && prior.watch == live.watch)
                {
                    child.inherited.push(InheritedLocal {
                        watch: live.watch,
                        instance: live.instance,
                        storage: storage.clone(),
                    });
                }
            }
        }
        Some(Box::new(child))
    }
}

impl LocalTraceState {
    fn sync(&mut self, worker: &Worker) {
        if self.ended {
            return;
        }
        let actual: Vec<_> = worker
            .call_stack
            .iter()
            .map(|frame| (frame.function, frame.base))
            .chain(std::iter::once((worker.function, worker.base)))
            .collect();
        let common = self
            .frames
            .iter()
            .zip(&actual)
            .take_while(|(old, new)| (old.function, old.base) == **new)
            .count();
        self.retire_from(common);
        for (function, base) in actual.into_iter().skip(common) {
            let mut history = self.history.lock().unwrap_or_else(|p| p.into_inner());
            let Some(activation) = history.activation() else {
                return;
            };
            let mut frame = LocalFrame {
                function,
                base,
                activation,
                live: Vec::new(),
            };
            let info = &worker.executable.executable().functions[function.get() as usize];
            let parameters: Vec<_> = history
                .report
                .watches
                .iter()
                .filter_map(|watch| match watch.identity {
                    DebugTraceIdentity::Local {
                        function_id,
                        binding_id,
                        ..
                    } if function_id == function.get()
                        && info.debug.bindings[binding_id].kind != DebugBindingKind::Local =>
                    {
                        Some((watch.id, binding_id))
                    }
                    _ => None,
                })
                .collect();
            super::local_admission::admit(&mut history, worker, &mut frame, parameters);
            self.frames.push(frame);
        }
        // Parameter/capture admission precedes the callee's first instruction.
        self.observe(
            worker,
            worker.function,
            InstructionAddress::new(worker.ip as u32),
        );
    }

    fn prune(&mut self, worker: &Worker, scope_site: Option<(FunctionId, InstructionAddress)>) {
        let image = worker.executable.executable();
        let Some(frame) = self.frames.last_mut() else {
            return;
        };
        let info = &image.functions[frame.function.get() as usize];
        // The caller's IP already follows the intrinsic while its operation still owns
        // callbacks or a delayed result store. That source scope remains live until commit.
        let address = scope_site
            .or_else(|| {
                worker
                    .callback_origin()
                    .filter(|_| worker.callback_resume_pending())
            })
            .filter(|(owner, _)| *owner == frame.function)
            .map_or(InstructionAddress::new(worker.ip as u32), |(_, site)| site);
        let scope = location::scope_at(image, frame.function, address).unwrap_or(0);
        let visible = location::visible_scope_chain(info, scope);
        let mut history = self.history.lock().unwrap_or_else(|p| p.into_inner());
        frame.live.retain(|live| {
            let keep = visible.contains(&info.debug.bindings[live.binding].scope);
            if !keep {
                history.retire(live.watch, live.instance);
            }
            keep
        });
    }

    fn initialize(&mut self, worker: &Worker, owner: FunctionId, address: InstructionAddress) {
        let Some(frame) = self
            .frames
            .iter_mut()
            .rev()
            .find(|frame| frame.function == owner && frame.base == worker.base)
        else {
            return;
        };
        let image = worker.executable.executable();
        let info = &image.functions[owner.get() as usize];
        let mut history = self.history.lock().unwrap_or_else(|p| p.into_inner());
        let declarations: Vec<_> = history
            .report
            .watches
            .iter()
            .filter_map(|watch| match watch.identity {
                DebugTraceIdentity::Local {
                    function_id,
                    binding_id,
                    ..
                } if function_id == owner.get()
                    && info.debug.bindings[binding_id].kind == DebugBindingKind::Local
                    && info.debug.bindings[binding_id].initializer == Some(address) =>
                {
                    Some((watch.id, binding_id))
                }
                _ => None,
            })
            .collect();
        frame.live.retain(|live| {
            if declarations.iter().any(|(watch, _)| *watch == live.watch) {
                history.retire(live.watch, live.instance);
                false
            } else {
                true
            }
        });
        super::local_admission::admit(&mut history, worker, frame, declarations);
    }

    fn observe(&self, worker: &Worker, owner: FunctionId, address: InstructionAddress) {
        let image = worker.executable.executable();
        let function = image
            .functions
            .get(owner.get() as usize)
            .and_then(|info| image.strings.get(info.name));
        let position = location::position_at(image, owner, address);
        let source = position
            .and_then(|p| location::source_path(image, p).map(|name| (name, p.line, p.column)));
        let mut history = self.history.lock().unwrap_or_else(|p| p.into_inner());
        let site = super::pending::TraceSite {
            task: worker.task_id,
            function,
            location: source,
            commit: Some(self.commit),
        };
        for frame in &self.frames {
            let info = &image.functions[frame.function.get() as usize];
            for live in &frame.live {
                let binding = &info.debug.bindings[live.binding];
                let value = super::local_value::binding_value(
                    worker,
                    frame.base,
                    binding,
                    &history.report.watches[live.watch],
                    live.parameter.as_ref(),
                );
                if let Some(value) = value {
                    history.observe_local(image, live.watch, live.instance, value, site);
                } else {
                    history.fail("trace_value_limit", false);
                    return;
                }
            }
        }
        for inherited in &self.inherited {
            let watch = &history.report.watches[inherited.watch];
            if let Some(value) =
                super::local_value::read(worker, &inherited.storage, &watch.path_types, &watch.path)
            {
                history.observe_local(image, inherited.watch, inherited.instance, value, site);
            } else {
                history.fail("trace_value_limit", false);
                return;
            }
        }
    }
}
