//! Frame/global capture and lexical visibility for one stopped generation.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use fpas_bytecode::{DebugBindingKind, FunctionId, InstructionAddress, Value};

use super::model::{DebugFrame, DebugInspectionLimits, DebugScope, DebugScopeKind};
use super::render::RetainedValue;
use super::snapshot::{FrameSnapshot, InspectionSnapshot, item_id};
use super::targets::{MutationAccess, MutationRoot, MutationTarget};
use crate::vm::debug::breakpoints;
use crate::vm::debug::types::SourceLocation;
use crate::vm::worker::Worker;

struct CapturedFrame {
    function: FunctionId,
    instruction: InstructionAddress,
    base: usize,
}

impl InspectionSnapshot {
    pub(in crate::vm::debug) fn capture(
        worker: &Worker,
        generation: u32,
        limits: DebugInspectionLimits,
    ) -> Self {
        let mut snapshot = Self {
            generation,
            executable: Arc::clone(&worker.executable),
            frames: Vec::new(),
            globals: capture_globals(worker, generation),
            total_frames: worker.call_stack.len().saturating_add(1),
            handles: Vec::new(),
            child_handles: HashMap::new(),
            limits,
        };
        let mut captured = vec![CapturedFrame {
            function: worker.function,
            instruction: InstructionAddress::try_from_index(worker.ip)
                .unwrap_or(worker.current_address),
            base: worker.base,
        }];
        captured.extend(worker.call_stack.iter().rev().map(|frame| {
            CapturedFrame {
                function: frame.function,
                instruction: InstructionAddress::try_from_index(frame.ip.saturating_sub(1))
                    .unwrap_or(worker.current_address),
                base: frame.base,
            }
        }));
        captured.truncate(limits.max_frames);
        for (depth, frame) in captured.into_iter().enumerate() {
            snapshot.capture_frame(worker, frame, depth);
        }
        snapshot
    }

    fn capture_frame(&mut self, worker: &Worker, frame: CapturedFrame, depth: usize) {
        let image = worker.executable.executable();
        let Some(function) = image.functions.get(usize::from(frame.function.get())) else {
            return;
        };
        let point = breakpoints::point_at(&worker.executable, frame.function, frame.instruction);
        let active_scope = point.map_or(0, |point| point.scope);
        let visible_scopes = visible_scope_chain(function, active_scope);
        let visible_scope_set = visible_scopes.iter().copied().collect::<HashSet<_>>();
        let mut parameters = Vec::new();
        let mut locals = Vec::new();
        let mut captures = Vec::new();
        let mut retained_bindings = Vec::new();
        let frame_id = item_id(self.generation, depth);
        for binding in &function.debug.bindings {
            if binding.hidden || !visible_scope_set.contains(&binding.scope) {
                continue;
            }
            let name = image.strings.get(binding.name).unwrap_or("<binding>");
            let type_name = image.strings.get(binding.type_name).unwrap_or("dynamic");
            let register = frame
                .base
                .saturating_add(usize::from(binding.register.get()));
            let value = worker
                .registers
                .get(register)
                .cloned()
                .and_then(|value| (!matches!(value, Value::Unit)).then_some(value));
            let mutation =
                binding_mutation(binding, value.as_ref(), register, self.generation, frame_id);
            let retained = RetainedValue {
                name: name.to_string(),
                value,
                type_name: type_name.to_string(),
                presentation_hint: binding.cell_backed.then(|| "captured mutable".to_string()),
                depth: 0,
                visited_cells: HashSet::new(),
                debug_type: Some(binding.ty),
                mutation,
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
        let location = point
            .and_then(|point| breakpoints::source_location(&worker.executable, point))
            .or_else(|| diagnostic_location(worker, frame.instruction));
        let name = image.strings.get(function.name).unwrap_or("<function>");
        self.frames.push(FrameSnapshot {
            frame: DebugFrame {
                id: frame_id,
                name: name.to_string(),
                location,
                depth,
            },
            scopes,
            evaluation_values,
        });
    }
}

fn binding_mutation(
    binding: &fpas_bytecode::DebugBinding,
    value: Option<&Value>,
    register: usize,
    generation: u32,
    frame_id: u64,
) -> MutationAccess {
    if !binding.mutable {
        return MutationAccess::NotMutable;
    }
    let root = if binding.cell_backed {
        match value {
            Some(Value::Cell(cell)) => MutationRoot::ClosureCell(Arc::clone(cell)),
            Some(_) => return MutationAccess::Unavailable,
            None => MutationRoot::FrameRegister(register),
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
    })
}

fn visible_scope_chain(function: &fpas_bytecode::FunctionInfo, mut scope: u32) -> Vec<u32> {
    let mut visible = Vec::new();
    loop {
        if visible.contains(&scope) {
            break;
        }
        visible.push(scope);
        let Some(parent) = function
            .debug
            .scopes
            .get(scope as usize)
            .and_then(|scope| scope.parent)
        else {
            break;
        };
        scope = parent;
    }
    visible
}

fn capture_globals(worker: &Worker, generation: u32) -> Vec<RetainedValue> {
    let image = worker.executable.executable();
    let globals = worker
        .globals
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    image
        .globals
        .iter()
        .enumerate()
        .map(|(index, global)| RetainedValue {
            name: image
                .strings
                .get(global.name)
                .unwrap_or("<global>")
                .to_string(),
            value: globals.get(index).cloned().flatten(),
            type_name: "dynamic".to_string(),
            presentation_hint: None,
            depth: 0,
            visited_cells: HashSet::new(),
            debug_type: Some(global.ty),
            mutation: if global.mutable {
                MutationAccess::Writable(MutationTarget {
                    root: MutationRoot::Global(index),
                    path: Vec::new(),
                    expected_type: global.ty,
                    generation,
                    frame_id: None,
                })
            } else {
                MutationAccess::NotMutable
            },
        })
        .collect()
}

fn diagnostic_location(worker: &Worker, instruction: InstructionAddress) -> Option<SourceLocation> {
    let image = worker.executable.executable();
    let run = image.source_map.lookup(instruction)?;
    let source = image
        .source_map
        .sources
        .get(run.source.get() as usize)
        .and_then(|source| image.strings.get(*source))?;
    Some(SourceLocation {
        source: source.to_string(),
        line: run.line,
        column: run.column,
    })
}
