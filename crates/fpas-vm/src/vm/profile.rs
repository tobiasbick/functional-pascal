//! Routine and source-line samples for `fpas run --profile`.
//!
//! Each dispatch batch adds its steps and wall time to the instruction address active when
//! the batch ends (self) and to every distinct function on the call stack (inclusive).
//! Workers keep local buffers and merge them when dropped, so pool threads never contend
//! per batch. Documentation: `docs/pascal/tools/profiler.md`.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use fpas_bytecode::{Executable, InstructionAddress};

use super::worker::Worker;

/// Dispatched steps and wall time attributed to one routine or line.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProfileSample {
    /// Dispatched scheduling steps, mostly instructions.
    pub instructions: u64,
    /// Wall time of the attributed dispatch batches.
    pub nanos: u64,
}

impl ProfileSample {
    fn add(&mut self, other: Self) {
        self.instructions = self.instructions.saturating_add(other.instructions);
        self.nanos = self.nanos.saturating_add(other.nanos);
    }
}

#[derive(Default)]
struct Samples {
    /// Self samples by executing function and instruction address. The function comes from
    /// the worker, so a batch ending on a callee's `Return` counts for the caller consistently
    /// with the inclusive stack.
    addresses: HashMap<(u16, u32), ProfileSample>,
    /// Inclusive samples by function index.
    functions: HashMap<u16, ProfileSample>,
}

impl Samples {
    fn merge(&mut self, other: Self) {
        for (address, sample) in other.addresses {
            self.addresses.entry(address).or_default().add(sample);
        }
        for (function, sample) in other.functions {
            self.functions.entry(function).or_default().add(sample);
        }
    }
}

/// Run-wide sample totals shared by every worker of one VM.
#[derive(Default)]
pub(super) struct ProfileSink(Mutex<Samples>);

/// Worker-local samples, merged into the run's sink when the worker is dropped.
pub(super) struct ProfileBuffer {
    sink: std::sync::Arc<ProfileSink>,
    samples: Samples,
    stack: Vec<u16>,
}

impl Drop for ProfileBuffer {
    fn drop(&mut self) {
        let samples = std::mem::take(&mut self.samples);
        self.sink
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .merge(samples);
    }
}

impl Worker {
    /// Attribute one completed dispatch batch to the current position and call stack.
    pub(super) fn record_profile_batch(&mut self, steps: u32, elapsed: Duration) {
        let Some(sink) = self.hosted.profile.get() else {
            return;
        };
        let buffer = self.profile.get_or_insert_with(|| {
            Box::new(ProfileBuffer {
                sink: std::sync::Arc::clone(sink),
                samples: Samples::default(),
                stack: Vec::new(),
            })
        });
        let sample = ProfileSample {
            instructions: u64::from(steps),
            nanos: u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX),
        };
        buffer
            .samples
            .addresses
            .entry((self.function.get(), self.current_address.get()))
            .or_default()
            .add(sample);
        buffer.stack.clear();
        buffer.stack.push(self.function.get());
        buffer
            .stack
            .extend(self.call_stack.iter().map(|frame| frame.function.get()));
        buffer.stack.sort_unstable();
        buffer.stack.dedup();
        for &function in &buffer.stack {
            buffer
                .samples
                .functions
                .entry(function)
                .or_default()
                .add(sample);
        }
    }
}

/// One routine's self and inclusive totals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutineProfile {
    /// Diagnostic routine name.
    pub name: String,
    /// Source path table index of the routine's first instruction.
    pub source: Option<u32>,
    /// One-based line of the routine's first instruction.
    pub line: Option<u32>,
    /// Samples whose batch ended inside this routine.
    pub self_sample: ProfileSample,
    /// Samples whose call stack contained this routine.
    pub inclusive: ProfileSample,
}

/// One source line's self totals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineProfile {
    /// Diagnostic name of the routine containing the line.
    pub routine: String,
    /// Source path table index.
    pub source: Option<u32>,
    /// One-based line.
    pub line: Option<u32>,
    /// Samples whose batch ended on this line.
    pub sample: ProfileSample,
}

/// Resolved profile of one VM run; rows are sorted by self time, descending.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProfileReport {
    /// Total sampled steps and wall time.
    pub total: ProfileSample,
    /// Every routine with samples.
    pub routines: Vec<RoutineProfile>,
    /// Every source line with samples.
    pub lines: Vec<LineProfile>,
}

impl ProfileSink {
    /// Resolve addresses and function indexes through the executable's metadata.
    pub(super) fn report(&self, executable: &Executable) -> ProfileReport {
        let samples = std::mem::take(
            &mut *self
                .0
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        let name = |function: u16| {
            executable
                .functions
                .get(usize::from(function))
                .and_then(|info| executable.strings.get(info.name))
                .unwrap_or("<unknown>")
                .to_owned()
        };
        let location = |address: u32| {
            executable
                .source_map
                .lookup(InstructionAddress::new(address))
                .map(|run| (run.source.get(), run.line))
        };
        let mut total = ProfileSample::default();
        let mut own: HashMap<u16, ProfileSample> = HashMap::new();
        let mut lines: HashMap<(Option<u32>, Option<u32>, u16), ProfileSample> = HashMap::new();
        for (&(function, address), &sample) in &samples.addresses {
            total.add(sample);
            own.entry(function).or_default().add(sample);
            let (source, line) = location(address).unzip();
            lines
                .entry((source, line, function))
                .or_default()
                .add(sample);
        }
        let mut routines: Vec<RoutineProfile> = samples
            .functions
            .iter()
            .map(|(&function, &inclusive)| {
                // The first sequence point names the declaration body; prologues may map elsewhere.
                let start = executable
                    .functions
                    .get(usize::from(function))
                    .and_then(|info| {
                        info.debug.sequence_points.first().map_or_else(
                            || location(info.code.start.get()),
                            |point| Some((point.location.source.get(), point.location.line)),
                        )
                    });
                RoutineProfile {
                    name: name(function),
                    source: start.map(|(source, _)| source),
                    line: start.map(|(_, line)| line),
                    self_sample: own.get(&function).copied().unwrap_or_default(),
                    inclusive,
                }
            })
            .collect();
        // Time ranks first: one instruction can allocate, validate a graph or call native code.
        routines.sort_by(|a, b| {
            (b.self_sample.nanos, b.self_sample.instructions, &a.name).cmp(&(
                a.self_sample.nanos,
                a.self_sample.instructions,
                &b.name,
            ))
        });
        let mut lines: Vec<LineProfile> = lines
            .into_iter()
            .map(|((source, line, function), sample)| LineProfile {
                routine: name(function),
                source,
                line,
                sample,
            })
            .collect();
        lines.sort_by(|a, b| {
            (b.sample.nanos, a.source, a.line).cmp(&(a.sample.nanos, b.source, b.line))
        });
        ProfileReport {
            total,
            routines,
            lines,
        }
    }
}

impl super::Vm {
    /// Collect routine and line samples during the next run.
    ///
    /// Process-wide event counters are enabled separately through `fpas_bytecode::profile`.
    pub fn enable_profile(&mut self) {
        let _ = self.hosted.profile.set(std::sync::Arc::default());
    }

    /// Pool threads that run spawned tasks; zero when the program spawns none.
    #[must_use]
    pub fn worker_count(&self) -> usize {
        self.pool_size
    }

    /// Resolve the samples of a finished run; `None` unless profiling was enabled.
    #[must_use]
    pub fn profile_report(&self) -> Option<ProfileReport> {
        self.hosted
            .profile
            .get()
            .map(|sink| sink.report(self.executable.executable()))
    }
}

#[cfg(test)]
mod tests;
