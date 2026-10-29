//! Live allocation envelopes for mixed task waits, plus exact post-teardown balances.

use crate::measurement::{Footprint, completed_threads};
use crate::workloads::{image, schedule};
use fpas_runtime::CollectionSchedule;
use fpas_runtime::execution::Vm;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

const SOURCE: &str = include_str!("fixtures/mixed.fpas");
const ROUNDS: usize = 512;
const WARMUP: usize = 64;
const BYTE_BOUND: i128 = 1024 * 1024;
const ALLOCATION_BOUND: i128 = 2048;

struct CheckpointWriter(Arc<Mutex<Vec<Footprint>>>);

impl Write for CheckpointWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        assert_eq!(bytes, b".", "unexpected workload output");
        let mut samples = self.0.lock().unwrap();
        assert!(samples.len() < samples.capacity(), "unplanned checkpoint");
        // Capacity is reserved before execution; recording itself allocates no memory.
        samples.push(Footprint::now());
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn run(
    executable: &fpas_runtime::VerifiedExecutable,
    config: CollectionSchedule,
    mode: &str,
    short: bool,
) -> Vec<Footprint> {
    let rounds = if short { 32 } else { ROUNDS };
    let samples = Arc::new(Mutex::new(Vec::with_capacity(rounds)));
    let mut arguments = vec![mode.to_owned()];
    if short {
        arguments.push("short".into());
    }
    let mut vm = Vm::with_writer_and_args(
        executable.clone(),
        Box::new(CheckpointWriter(Arc::clone(&samples))),
        arguments,
    )
    .with_collection_schedule(config);
    let outcome = vm.run();
    if mode == "fail" {
        let error = outcome.unwrap_err();
        assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC);
        assert!(
            error.message.contains("expected mixed selection failure"),
            "{error:?}"
        );
    } else {
        drop(outcome.unwrap());
    }
    drop(vm);
    let samples = Arc::try_unwrap(samples).unwrap().into_inner().unwrap();
    assert_eq!(samples.len(), if mode == "fail" { 15 } else { rounds });
    samples
}

fn growth(samples: &[Footprint]) -> Footprint {
    assert_eq!(samples.len(), ROUNDS);
    let baseline = samples[WARMUP - 1];
    samples[WARMUP..]
        .iter()
        .fold(Footprint::default(), |peak, sample| Footprint {
            bytes: peak.bytes.max(sample.bytes - baseline.bytes),
            allocations: peak
                .allocations
                .max(sample.allocations - baseline.allocations),
        })
}

pub fn live_mixed_waits() {
    let executable = image(SOURCE);
    // Retain reachable payloads on purpose: a checker that only measures after
    // teardown would accept this control, although its live footprint grows.
    let control = growth(&run(
        &executable,
        CollectionSchedule::default(),
        "retain",
        false,
    ));
    assert!(
        control.bytes > BYTE_BOUND,
        "retention control did not exceed the envelope: {control:?}"
    );
    assert!(
        control.allocations > 0,
        "retention must also allocate storage"
    );
    for config in [CollectionSchedule::default(), schedule()] {
        let peak = growth(&run(&executable, config, "collect", false));
        assert!(peak.bytes < BYTE_BOUND, "mixed live byte growth: {peak:?}");
        assert!(
            peak.allocations < ALLOCATION_BOUND,
            "mixed live allocation growth: {peak:?}"
        );
        println!("mixed waits: {ROUNDS} live checkpoints, growth {peak:?}");
    }
}

pub fn mixed_wait_teardown() {
    let executable = image(SOURCE);
    for mode in ["collect", "retain", "fail"] {
        for config in [CollectionSchedule::default(), schedule()] {
            completed_threads("mixed wait teardown", 4, || {
                drop(run(&executable, config, mode, true));
            });
        }
    }
}
