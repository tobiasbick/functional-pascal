//! DAP event translation from typed debug engine records.

use serde_json::{Value, json};

use super::DapServer;
use crate::engine::{DebugEvent, DebugRecord};

impl DapServer {
    pub(super) fn translate_events(&mut self, records: Vec<DebugRecord>) -> Vec<Value> {
        records
            .into_iter()
            .flat_map(|record| match record {
                DebugRecord::Event(event) => self.translate_event(event),
                DebugRecord::Response { .. } => Vec::new(),
            })
            .collect()
    }

    pub(super) fn translate_event(&mut self, event: DebugEvent) -> Vec<Value> {
        match event {
            DebugEvent::Initialized => vec![self.event("initialized", json!({}))],
            DebugEvent::Stopped(stop) => {
                self.runtime_failed =
                    stop.reason == fpas_runtime::execution::DebugStopReason::RuntimeError;
                let thread_id = self.threads.thread_id(stop.task_id);
                vec![self.event(
                    "stopped",
                    json!({
                        "reason": dap_stop_reason(stop.reason),
                        "threadId": thread_id,
                        "allThreadsStopped": true
                    }),
                )]
            }
            DebugEvent::Task(change) => {
                let thread_id = self.threads.thread_id(change.task_id);
                let reason = match change.kind {
                    fpas_runtime::execution::DebugTaskEventKind::Started => "started",
                    fpas_runtime::execution::DebugTaskEventKind::Exited => "exited",
                };
                if matches!(
                    change.kind,
                    fpas_runtime::execution::DebugTaskEventKind::Exited
                ) {
                    self.threads.mark_exited(change.task_id);
                }
                vec![self.event("thread", json!({"reason": reason, "threadId": thread_id}))]
            }
            DebugEvent::Output {
                category,
                text,
                location,
                ..
            } => vec![self.event(
                "output",
                json!({
                    "category": category,
                    "output": text,
                    "source": location.as_ref().map(|location| json!({"path": location.source})),
                    "line": location.as_ref().map(|location| location.line),
                    "column": location.as_ref().map(|location| location.column)
                }),
            )],
            DebugEvent::Terminated { exit_code, .. } => vec![
                self.event("exited", json!({"exitCode": exit_code})),
                self.event("terminated", json!({})),
            ],
            DebugEvent::RuntimeError { diagnostic, .. } => vec![self.event(
                "output",
                json!({
                    "category": "stderr",
                    "output": format!("{}\n", diagnostic.message)
                }),
            )],
            DebugEvent::SupplementalDiagnostic(diagnostic) => vec![self.event(
                "output",
                json!({
                    "category": "stderr",
                    "output": format!("{}: {}\n", diagnostic.code, diagnostic.message)
                }),
            )],
            DebugEvent::ProtocolError(error) => vec![self.event(
                "output",
                json!({
                    "category": "stderr",
                    "output": format!("{}\n", error.message)
                }),
            )],
            DebugEvent::SourceBreakpoint(_)
            | DebugEvent::FunctionBreakpoint(_)
            | DebugEvent::DataBreakpoint(_) => Vec::new(),
        }
    }
}

fn dap_stop_reason(reason: fpas_runtime::execution::DebugStopReason) -> &'static str {
    match reason {
        fpas_runtime::execution::DebugStopReason::Entry => "entry",
        fpas_runtime::execution::DebugStopReason::Breakpoint => "breakpoint",
        fpas_runtime::execution::DebugStopReason::DataBreakpoint => "data breakpoint",
        fpas_runtime::execution::DebugStopReason::Pause => "pause",
        fpas_runtime::execution::DebugStopReason::Step => "step",
        fpas_runtime::execution::DebugStopReason::RuntimeError => "exception",
    }
}
