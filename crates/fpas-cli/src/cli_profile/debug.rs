//! Debug-target reports share the run profiler's bounded rows and publication policy.

use std::{path::PathBuf, time::Instant};

use fpas_bytecode::profile::{self, Counter};
use serde_json::json;

use super::{Run, Target, render, round3};
use crate::{CliInput, DebugCliConfig, DebugProtocol};

pub(crate) struct Session {
    destination: PathBuf,
    target: Target,
    sources: Vec<PathBuf>,
    profile: fpas_vm::DebugProfile,
    collection: fpas_bytecode::CollectionSchedule,
    protocol: &'static str,
    counting: profile::Run,
    started: Instant,
}

impl Session {
    pub(crate) fn start(
        target: &mut fpas_debug::PreparedDebugTarget,
        config: &DebugCliConfig,
    ) -> Option<Self> {
        let destination = config.profile.clone()?;
        let profile = target.enable_profile();
        let (kind, path) = match &config.input {
            CliInput::SourceFile(path) => ("source", path),
            CliInput::ProjectFile(path) => ("project", path),
            CliInput::WorkspaceFile(path) => ("workspace", path),
            CliInput::CompiledProgramFile(path) => ("compiled", path),
        };
        Some(Self {
            destination,
            target: Target::new(kind, path, profile.executable().executable()),
            sources: target
                .source_paths()
                .into_iter()
                .map(PathBuf::from)
                .collect(),
            profile,
            collection: config.collection_schedule,
            protocol: match config.protocol {
                DebugProtocol::Batch => "batch",
                DebugProtocol::Dap => "dap",
                DebugProtocol::Jsonl => "jsonl",
            },
            counting: profile::begin(),
            started: Instant::now(),
        })
    }

    /// All protocol owners and their workers must be dropped before finishing.
    pub(crate) fn finish(self, exit_code: i32) -> std::io::Result<()> {
        let wall = self.started.elapsed();
        let counters = self.counting.finish();
        let report = self.profile.report();
        let outcome = self.profile.outcome();
        let target_exit = match outcome {
            "runtime_error" => 2,
            "error" => 3,
            _ => 0,
        };
        let run = Run {
            target: &self.target,
            collection: self.collection,
            workers: 0,
            counters_exclusive: counters.exclusive,
            debug: true,
        };
        let mut document = render(
            &report,
            &counters.counts,
            wall,
            &run,
            if exit_code == 0 {
                target_exit
            } else {
                exit_code
            },
            Some(&self.sources),
        );
        let milliseconds = |counter| {
            let nanos = counters
                .counts
                .iter()
                .find(|(key, _)| *key == counter)
                .map_or(0, |(_, nanos)| *nanos);
            round3(nanos as f64 / 1_000_000.0)
        };
        document["run"]["runtime"]["execution"] = json!("debug_serial");
        document["debug"] = json!({
            "protocol": self.protocol,
            "outcome": outcome,
            "timing": "inclusive_wall",
            "resume_ms": milliseconds(Counter::DebugResumeNanos),
            "schedule_ms": milliseconds(Counter::DebugScheduleNanos),
            "dispatch_ms": milliseconds(Counter::DebugDispatchNanos),
            "root_publication_ms": milliseconds(Counter::RootPublicationNanos),
            "inspection_ms": milliseconds(Counter::DebugInspectionNanos),
            "wait_ms": milliseconds(Counter::DebugWaitNanos),
        });
        let mut bytes = serde_json::to_vec_pretty(&document)?;
        bytes.push(b'\n');
        crate::cli_failure_dump::publish_report(&self.destination, &bytes)
    }
}
