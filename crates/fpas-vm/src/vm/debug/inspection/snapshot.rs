//! Immutable frame-window capture, lexical visibility, and stable lazy handles.

use std::collections::{HashMap, HashSet};

use fpas_bytecode::{DebugBindingKind, FunctionId, InstructionAddress};

use super::model::{
    DebugFrame, DebugInspectionLimits, DebugScope, DebugScopeKind, DebugVariable, Paginated,
};
use super::render::{self, RetainedValue};
use crate::vm::debug::breakpoints;
use crate::vm::debug::types::{DebugErrorKind, DebugSessionError, SourceLocation};
use crate::vm::worker::Worker;

pub(crate) struct InspectionSnapshot {
    generation: u32,
    frames: Vec<FrameSnapshot>,
    total_frames: usize,
    handles: Vec<HandleEntry>,
    child_handles: HashMap<(u64, usize), u64>,
    limits: DebugInspectionLimits,
}

struct FrameSnapshot {
    frame: DebugFrame,
    scopes: Vec<DebugScope>,
}

struct HandleEntry {
    id: u64,
    values: Vec<RetainedValue>,
}

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
            frames: Vec::new(),
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

    pub(in crate::vm::debug) fn empty(generation: u32, limits: DebugInspectionLimits) -> Self {
        Self {
            generation,
            frames: Vec::new(),
            total_frames: 0,
            handles: Vec::new(),
            child_handles: HashMap::new(),
            limits,
        }
    }

    pub(in crate::vm::debug) fn stack(
        &self,
        start: usize,
        count: usize,
    ) -> Result<Paginated<DebugFrame>, DebugSessionError> {
        self.check_page("stack", count, self.limits.max_frames)?;
        let items = self
            .frames
            .iter()
            .skip(start)
            .take(count)
            .map(|frame| frame.frame.clone())
            .collect();
        Ok(Paginated {
            items,
            total: self.total_frames,
        })
    }

    pub(in crate::vm::debug) fn scopes(
        &self,
        frame_id: u64,
    ) -> Result<Vec<DebugScope>, DebugSessionError> {
        self.frames
            .iter()
            .find(|frame| frame.frame.id == frame_id)
            .map(|frame| frame.scopes.clone())
            .ok_or_else(|| DebugSessionError {
                kind: DebugErrorKind::UnknownFrame,
                message: format!("debug frame {frame_id} is unknown or expired"),
                hint: "Request stack frames again for the current stop.".to_string(),
            })
    }

    pub(in crate::vm::debug) fn variables(
        &mut self,
        reference: u64,
        start: usize,
        count: usize,
    ) -> Result<Paginated<DebugVariable>, DebugSessionError> {
        self.check_page("variables", count, self.limits.max_children)?;
        let Some(handle_index) = self.handles.iter().position(|entry| entry.id == reference) else {
            return Err(DebugSessionError {
                kind: DebugErrorKind::UnknownVariablesReference,
                message: format!("debug variables reference {reference} is unknown or expired"),
                hint: "Request scopes or parent variables again for the current stop.".to_string(),
            });
        };
        let total = self.handles[handle_index].values.len();
        let retained = self.handles[handle_index]
            .values
            .iter()
            .skip(start)
            .take(count)
            .cloned()
            .collect::<Vec<_>>();
        let mut items = Vec::with_capacity(retained.len());
        let mut output_bytes = 0_usize;
        for (relative_index, retained) in retained.into_iter().enumerate() {
            let absolute_index = start.saturating_add(relative_index);
            let rendered = render::render(&retained, self.limits);
            let variables_reference = if rendered.children.is_empty() {
                0
            } else if let Some(existing) = self.child_handles.get(&(reference, absolute_index)) {
                *existing
            } else {
                let handle = self.allocate_handle(rendered.children)?;
                self.child_handles
                    .insert((reference, absolute_index), handle);
                handle
            };
            let variable = DebugVariable {
                name: retained.name,
                value: rendered.summary,
                type_name: rendered.type_name,
                variables_reference,
                named_variables: rendered.named_children,
                indexed_variables: rendered.indexed_children,
                presentation_hint: rendered.presentation_hint,
            };
            let size = variable.name.len()
                + variable.value.len()
                + variable.type_name.len()
                + variable.presentation_hint.as_ref().map_or(0, String::len);
            if output_bytes.saturating_add(size) > self.limits.max_output_bytes {
                break;
            }
            output_bytes += size;
            items.push(variable);
        }
        Ok(Paginated { items, total })
    }

    fn capture_frame(&mut self, worker: &Worker, frame: CapturedFrame, depth: usize) {
        let image = worker.executable.executable();
        let Some(function) = image.functions.get(usize::from(frame.function.get())) else {
            return;
        };
        let point = breakpoints::point_at(&worker.executable, frame.function, frame.instruction);
        let active_scope = point.map_or(0, |point| point.scope);
        let visible_scopes = visible_scope_ids(function, active_scope);
        let mut parameters = Vec::new();
        let mut locals = Vec::new();
        let mut captures = Vec::new();
        for binding in &function.debug.bindings {
            if binding.hidden || !visible_scopes.contains(&binding.scope) {
                continue;
            }
            let name = image.strings.get(binding.name).unwrap_or("<binding>");
            let type_name = image.strings.get(binding.type_name).unwrap_or("dynamic");
            let value = worker
                .registers
                .get(
                    frame
                        .base
                        .saturating_add(usize::from(binding.register.get())),
                )
                .cloned();
            let retained = RetainedValue {
                name: name.to_string(),
                value,
                type_name: type_name.to_string(),
                presentation_hint: binding.cell_backed.then(|| "captured mutable".to_string()),
                depth: 0,
                visited_cells: HashSet::new(),
            };
            match binding.kind {
                DebugBindingKind::Parameter => parameters.push(retained),
                DebugBindingKind::Local => locals.push(retained),
                DebugBindingKind::Capture => captures.push(retained),
            }
        }
        let globals = image
            .globals
            .iter()
            .enumerate()
            .map(|(index, global)| RetainedValue {
                name: image
                    .strings
                    .get(global.name)
                    .unwrap_or("<global>")
                    .to_string(),
                value: worker
                    .globals
                    .read()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .get(index)
                    .cloned()
                    .flatten(),
                type_name: "dynamic".to_string(),
                presentation_hint: None,
                depth: 0,
                visited_cells: HashSet::new(),
            })
            .collect::<Vec<_>>();
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
                .map(|reference| DebugScope {
                    name: name.to_string(),
                    kind,
                    variables_reference: reference,
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
                id: item_id(self.generation, depth),
                name: name.to_string(),
                location,
                depth,
            },
            scopes,
        });
    }

    fn allocate_handle(&mut self, values: Vec<RetainedValue>) -> Result<u64, DebugSessionError> {
        if self.handles.len() >= self.limits.max_handles {
            return Err(DebugSessionError {
                kind: DebugErrorKind::InspectionLimit,
                message: "debug variable handle limit reached".to_string(),
                hint: "Reduce aggregate expansion or increase the configured handle limit."
                    .to_string(),
            });
        }
        let id = item_id(self.generation, self.handles.len());
        self.handles.push(HandleEntry { id, values });
        Ok(id)
    }

    fn check_page(
        &self,
        operation: &'static str,
        count: usize,
        maximum: usize,
    ) -> Result<(), DebugSessionError> {
        if count <= maximum {
            return Ok(());
        }
        Err(DebugSessionError {
            kind: DebugErrorKind::InspectionLimit,
            message: format!("debug {operation} page count {count} exceeds limit {maximum}"),
            hint: format!("Request at most {maximum} items and use pagination."),
        })
    }
}

fn visible_scope_ids(function: &fpas_bytecode::FunctionInfo, mut scope: u32) -> HashSet<u32> {
    let mut visible = HashSet::new();
    loop {
        if !visible.insert(scope) {
            break;
        }
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

fn item_id(generation: u32, index: usize) -> u64 {
    (u64::from(generation) << 32) | u64::try_from(index.saturating_add(1)).unwrap_or(u64::MAX)
}
