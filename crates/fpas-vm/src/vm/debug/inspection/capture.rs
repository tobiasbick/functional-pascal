//! Frame/global capture and lexical visibility for one stopped generation.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use fpas_bytecode::{DebugBindingKind, Value};

use super::model::{DebugFrame, DebugInspectionLimits, DebugScope, DebugScopeKind};
use super::render::RetainedValue;
use super::snapshot::{FrameSnapshot, InspectionSnapshot, item_id};
use super::targets::{MutationAccess, MutationRoot, MutationTarget};
use crate::vm::debug::initializer_suppression::SourceInitializerTarget;
use crate::vm::debug::routines::display_name;
use crate::vm::debug::types::SourceLocation;
use crate::vm::inspection::{
    bindings,
    frames::{self, Boundary, Frame as CapturedFrame},
    globals, location,
};
use crate::vm::worker::Worker;

impl InspectionSnapshot {
    /// Retains globals after normal completion without manufacturing a live frame.
    pub(in crate::vm::debug) fn completed(
        worker: &Worker,
        generation: u32,
        limits: DebugInspectionLimits,
    ) -> Self {
        Self {
            generation,
            executable: Arc::clone(&worker.executable),
            frames: Vec::new(),
            globals: capture_globals(worker, generation),
            total_frames: 0,
            handles: Vec::new(),
            child_handles: HashMap::new(),
            limits,
        }
    }
    /// Capture debugger-owned handles at the current stopped boundary.
    pub(in crate::vm::debug) fn capture(
        worker: &Worker,
        generation: u32,
        limits: DebugInspectionLimits,
    ) -> Self {
        Self::capture_with_reserved_handles(worker, generation, limits, 0)
    }

    /// Capture a snapshot while leaving bounded capacity for post-commit result handles.
    pub(in crate::vm::debug) fn capture_with_reserved_handles(
        worker: &Worker,
        generation: u32,
        limits: DebugInspectionLimits,
        reserved_handles: usize,
    ) -> Self {
        Self::capture_at(
            worker,
            generation,
            limits,
            reserved_handles,
            Boundary::Stopped,
        )
    }

    /// Captures faulting callback frames for non-executing batch inspection.
    pub(in crate::vm::debug) fn capture_fault(
        worker: &Worker,
        generation: u32,
        limits: DebugInspectionLimits,
    ) -> Self {
        Self::capture_at(worker, generation, limits, 0, Boundary::Fault)
    }

    fn capture_at(
        worker: &Worker,
        generation: u32,
        limits: DebugInspectionLimits,
        reserved_handles: usize,
        boundary: Boundary,
    ) -> Self {
        let mut capture_limits = limits;
        capture_limits.max_handles = capture_limits.max_handles.saturating_sub(reserved_handles);
        let mut snapshot = Self {
            generation,
            executable: Arc::clone(&worker.executable),
            frames: Vec::new(),
            globals: capture_globals(worker, generation),
            total_frames: worker.call_stack.len().saturating_add(1),
            handles: Vec::new(),
            child_handles: HashMap::new(),
            limits: capture_limits,
        };
        let count = frames::visit(
            worker,
            boundary,
            limits.max_frames,
            |owner, frame, depth| {
                snapshot.capture_frame(owner, frame, depth);
            },
        );
        debug_assert!(boundary != Boundary::Stopped || count.unavailable_reason.is_none());
        snapshot.total_frames = count.total.unwrap_or(snapshot.frames.len());
        snapshot.limits = limits;
        snapshot
    }

    fn capture_frame(&mut self, worker: &Worker, frame: CapturedFrame, depth: usize) {
        let image = worker.executable.executable();
        let Some(function) = image.functions.get(usize::from(frame.function.get())) else {
            return;
        };
        let active_scope =
            location::scope_at(image, frame.function, frame.instruction).unwrap_or(0);
        let visible_scopes = location::visible_scope_chain(function, active_scope);
        let visible_scope_set = visible_scopes.iter().copied().collect::<HashSet<_>>();
        let mut parameters = Vec::new();
        let mut locals = Vec::new();
        let mut captures = Vec::new();
        let mut retained_bindings = Vec::new();
        let mut frame_bindings = Vec::with_capacity(function.debug.bindings.len());
        let frame_id = item_id(self.generation, depth);
        for binding in &function.debug.bindings {
            let visible = visible_scope_set.contains(&binding.scope);
            let register = frame
                .base
                .saturating_add(usize::from(binding.register.get()));
            let initialized = worker.register_is_initialized(register);
            let raw_value = initialized
                .then(|| worker.registers.get(register))
                .flatten();
            let referent_type = match image.debug_types.get(binding.ty.get() as usize) {
                Some(fpas_bytecode::DebugType::Reference(inner)) => Some(*inner),
                _ => None,
            };
            // A `var` parameter shows the caller's current value, not its reference.
            let (value, unavailable) = bindings::read(worker, frame.base, binding).into_parts();
            frame_bindings.push(super::snapshot::FrameBinding {
                initialized,
                value: value.clone(),
                ty: binding.ty,
                kind: binding.kind,
                hidden: binding.hidden,
                cell_backed: binding.cell_backed,
                visible,
            });
            if binding.hidden || !visible {
                continue;
            }
            let name = image.strings.get(binding.name).unwrap_or("<binding>");
            let type_name = image.strings.get(binding.type_name).unwrap_or("dynamic");
            let mutation = binding_mutation(
                binding,
                value.as_ref(),
                initialized,
                register,
                self.generation,
                frame_id,
                binding
                    .initializer
                    .filter(|initializer| initializer.get() >= frame.next_instruction.get())
                    .map(|instruction| SourceInitializerTarget {
                        function: frame.function,
                        base: frame.base,
                        instruction,
                    }),
            );
            let mutation = if let Some(expected) = referent_type {
                match raw_value {
                    Some(Value::Reference(reference)) if value.is_some() => {
                        super::references::mutation(
                            image,
                            reference,
                            expected,
                            self.generation,
                            frame_id,
                        )
                    }
                    _ => MutationAccess::Unavailable,
                }
            } else {
                mutation
            };
            let retained = RetainedValue {
                name: name.to_string(),
                value,
                type_name: type_name.to_string(),
                presentation_hint: unavailable
                    .map(|reason| format!("unavailable: {reason}"))
                    .or_else(|| binding.cell_backed.then(|| "captured mutable".to_string())),
                depth: 0,
                visited_cells: HashSet::new(),
                debug_type: Some(bindings::value_type(worker, binding)),
                origin: mutation,
            };
            retained_bindings.push((binding.scope, retained.clone()));
            match binding.kind {
                DebugBindingKind::Parameter => parameters.push(retained),
                DebugBindingKind::Local => locals.push(retained),
                DebugBindingKind::Capture => captures.push(retained),
            }
        }
        let globals = self.globals.clone();
        let evaluation_values = visible_scopes
            .iter()
            .flat_map(|scope| retained_bindings.iter().filter(move |(id, _)| id == scope))
            .map(|(_, value)| value.clone())
            .collect();
        let scopes = [
            ("Parameters", DebugScopeKind::Parameters, parameters, false),
            ("Locals", DebugScopeKind::Locals, locals, false),
            ("Captures", DebugScopeKind::Captures, captures, false),
            ("Globals", DebugScopeKind::Globals, globals, true),
        ]
        .into_iter()
        .filter_map(|(name, kind, values, expensive)| {
            if values.is_empty() {
                return None;
            }
            let named_variables = values.len();
            self.allocate_handle(values)
                .ok()
                .map(|variables_reference| DebugScope {
                    name: name.to_string(),
                    kind,
                    variables_reference,
                    named_variables,
                    expensive,
                })
        })
        .collect();
        let location =
            location::position_at(image, frame.function, frame.instruction).and_then(|position| {
                Some(SourceLocation {
                    source: location::source_path(image, position)?.to_string(),
                    line: position.line,
                    column: position.column,
                })
            });
        let name = image.strings.get(function.name).unwrap_or("<function>");
        self.frames.push(FrameSnapshot {
            frame: DebugFrame {
                id: frame_id,
                name: display_name(name).to_string(),
                location,
                depth,
            },
            function: frame.function,
            scopes,
            evaluation_values,
            bindings: frame_bindings,
        });
    }
}

fn binding_mutation(
    binding: &fpas_bytecode::DebugBinding,
    value: Option<&Value>,
    initialized: bool,
    register: usize,
    generation: u32,
    frame_id: u64,
    initializer: Option<SourceInitializerTarget>,
) -> MutationAccess {
    if !binding.mutable {
        return MutationAccess::NotMutable;
    }
    if matches!(
        binding.kind,
        DebugBindingKind::Parameter | DebugBindingKind::Capture
    ) && !initialized
    {
        return MutationAccess::Unavailable;
    }
    let root = if binding.cell_backed {
        match value {
            Some(Value::Cell(cell)) => MutationRoot::ClosureCell(Arc::clone(cell)),
            _ => return MutationAccess::Unavailable,
        }
    } else {
        MutationRoot::FrameRegister(register)
    };
    MutationAccess::Writable(MutationTarget {
        root,
        path: Vec::new(),
        expected_type: binding.ty,
        generation,
        frame_id: Some(frame_id),
        initialized,
        initializer: if initialized { None } else { initializer },
    })
}

fn capture_globals(worker: &Worker, generation: u32) -> Vec<RetainedValue> {
    let image = worker.executable.executable();
    let slots = globals::slots(worker);
    image
        .globals
        .iter()
        .enumerate()
        .map(|(index, global)| {
            let (value, unavailable) = match &slots {
                Ok(slots) => globals::read_slot(slots, index),
                Err(reason) => bindings::ReadValue::Unavailable(reason),
            }
            .into_parts();
            RetainedValue {
                name: image
                    .strings
                    .get(global.name)
                    .unwrap_or("<global>")
                    .to_string(),
                value: value.clone(),
                type_name: "dynamic".to_string(),
                presentation_hint: unavailable.map(|reason| format!("unavailable: {reason}")),
                depth: 0,
                visited_cells: HashSet::new(),
                debug_type: Some(global.ty),
                origin: if unavailable.is_some() {
                    MutationAccess::Unavailable
                } else if global.mutable {
                    MutationAccess::Writable(MutationTarget {
                        root: MutationRoot::Global(index),
                        path: Vec::new(),
                        expected_type: global.ty,
                        generation,
                        frame_id: None,
                        initialized: value.is_some(),
                        initializer: value
                            .is_none()
                            .then_some(global.initializer)
                            .flatten()
                            .and_then(|initializer| {
                                pending_global_initializer(worker, initializer)
                            }),
                    })
                } else {
                    MutationAccess::NotMutable
                },
            }
        })
        .collect()
}

fn pending_global_initializer(
    worker: &Worker,
    initializer: fpas_bytecode::GlobalInitializer,
) -> Option<SourceInitializerTarget> {
    let mut matches = Vec::new();
    if worker.function == initializer.function
        && initializer.instruction.get() >= u32::try_from(worker.ip).unwrap_or(u32::MAX)
    {
        matches.push(SourceInitializerTarget {
            function: worker.function,
            base: worker.base,
            instruction: initializer.instruction,
        });
    }
    matches.extend(worker.call_stack.iter().filter_map(|frame| {
        (frame.function == initializer.function
            && initializer.instruction.get() >= u32::try_from(frame.ip).unwrap_or(u32::MAX))
        .then_some(SourceInitializerTarget {
            function: frame.function,
            base: frame.base,
            instruction: initializer.instruction,
        })
    }));
    (matches.len() == 1).then(|| matches[0])
}
