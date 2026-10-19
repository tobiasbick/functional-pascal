//! Consecutive scalar comparison, detached capture and charged event retention.

use super::{binding, model::*};
use crate::vm::inspection::values::{
    ValueLimits, budget::ValueBudget, render::RetainedValue, snapshot,
};
use fpas_bytecode::{Executable, Value};
use std::collections::HashSet;

pub(in crate::vm) struct TraceHistory {
    pub report: DebugTraceReport,
    pub limit: Option<&'static str>,
    event_limit: usize,
    bytes: usize,
    work: usize,
}

impl TraceHistory {
    pub(in crate::vm::debug) fn new(
        image: &Executable,
        selectors: &[String],
        limit: usize,
    ) -> Result<Self, String> {
        if limit == 0 || limit > MAX_TRACE_EVENTS {
            return Err("Trace event limit must be between 1 and 10000.".into());
        }
        let watches = binding::bind(image, selectors)?;
        let report = DebugTraceReport {
            requested: watches.iter().map(|watch| watch.request.clone()).collect(),
            watches,
            complete: true,
            total_count: Some(0),
            ..DebugTraceReport::default()
        };
        let bytes = encoded_size(&report).saturating_add(4096);
        Ok(Self {
            report,
            limit: None,
            event_limit: limit,
            bytes,
            work: MAX_WORK_BYTES,
        })
    }

    pub(super) fn fail(&mut self, reason: &'static str, known: bool) {
        self.limit.get_or_insert(reason);
        self.report.complete = false;
        self.report.reasons.insert(reason);
        if !known {
            self.report.total_count = None;
        }
    }

    pub(super) fn leaves(
        &self,
        global: usize,
        root: Option<&Value>,
    ) -> Result<Vec<(usize, Option<Value>)>, &'static str> {
        self.report
            .watches
            .iter()
            .filter(|watch| watch.identity.global_slot == global)
            .map(|watch| {
                let mut leaf = root;
                for slot in &watch.identity.field_slots {
                    leaf = match leaf {
                        None => None,
                        Some(Value::Record(record)) => {
                            Some(record.body().values.get(*slot).ok_or("trace_value_limit")?)
                        }
                        _ => return Err("trace_value_limit"),
                    };
                }
                let valid = match leaf {
                    None | Some(Value::Integer(_) | Value::Real(_) | Value::Boolean(_)) => true,
                    Some(Value::Str(text)) => text.len() <= MAX_SCALAR_BYTES,
                    Some(Value::Enum(value)) => {
                        value.body().values.is_empty()
                            && value.body().layout.type_name.len() <= 4096
                            && value.body().layout.variant.len() <= 4096
                    }
                    _ => false,
                };
                if !valid {
                    return Err("trace_value_limit");
                }
                Ok((watch.id, leaf.cloned()))
            })
            .collect()
    }

    pub(super) fn commit(
        &mut self,
        image: &Executable,
        old: Vec<(usize, Option<Value>)>,
        new: Vec<(usize, Option<Value>)>,
        task: u64,
        function: Option<&str>,
        location: Option<(&str, u32, u32)>,
    ) {
        for ((id, old), (_, new)) in old.into_iter().zip(new) {
            let cost = string_bytes(old.as_ref()).saturating_add(string_bytes(new.as_ref()));
            if cost > self.work {
                self.fail("trace_work_limit", false);
                return;
            }
            self.work -= cost;
            let equal = match (&old, &new) {
                (Some(Value::Real(left)), Some(Value::Real(right))) => left == right,
                _ => old == new,
            };
            if equal {
                continue;
            }
            self.report.total_count = self.report.total_count.map(|count| count + 1);
            // Finish comparing all affected watches for an exact known total when possible.
            if self.limit.is_some() {
                continue;
            }
            if self.report.events.len() == self.event_limit {
                self.fail("trace_event_limit", true);
                continue;
            }
            let watch = &self.report.watches[id];
            let old_value = capture(image, watch, old);
            let new_value = capture(image, watch, new);
            let (Some(old_value), Some(new_value)) = (old_value, new_value) else {
                self.fail("trace_byte_limit", true);
                continue;
            };
            if function.is_some_and(|name| name.len() > 4096)
                || location.is_some_and(|(source, _, _)| source.len() > 4096)
            {
                self.fail("trace_byte_limit", true);
                continue;
            }
            let event = DebugTraceEvent {
                sequence: self.report.event_count + 1,
                watch_id: id,
                kind: if old_value.status == "uninitialized" {
                    "initialize"
                } else {
                    "change"
                },
                task_id: task,
                function: function.map(str::to_owned),
                location: location.map(|(source, line, column)| DebugTraceLocation {
                    source: source.into(),
                    line,
                    column,
                }),
                old_value,
                new_value,
            };
            let bytes = encoded_size(&event).saturating_add(1);
            if bytes > MAX_TRACE_BYTES.saturating_sub(self.bytes) {
                self.fail("trace_byte_limit", true);
                continue;
            }
            self.bytes += bytes;
            if event.old_value.status == "truncated" || event.new_value.status == "truncated" {
                self.report.reasons.insert("value_truncated");
            }
            self.report.events.push(event);
            self.report.event_count += 1;
        }
    }
}

fn string_bytes(value: Option<&Value>) -> usize {
    match value {
        Some(Value::Str(text)) => text.len(),
        _ => 0,
    }
}

fn capture(
    image: &Executable,
    watch: &DebugTraceWatch,
    value: Option<Value>,
) -> Option<crate::FailureValue> {
    let value = RetainedValue {
        name: String::new(),
        value,
        type_name: watch.type_name.clone(),
        presentation_hint: None,
        depth: 0,
        visited_cells: HashSet::new(),
        debug_type: Some(watch.ty),
        origin: (),
    };
    snapshot::capture(
        &value,
        ValueLimits {
            max_children: 0,
            max_string_chars: 4096,
            max_summary_chars: 4096,
            max_depth: 0,
        },
        Some(image),
        &mut ValueBudget::new(2, 128 * 1024),
    )
}
