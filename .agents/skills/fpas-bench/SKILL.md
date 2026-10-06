---
name: fpas-bench
description: Measure FPAS runtime, compiler, and editor performance with cargo bench-fpas, compare baselines, and record settled results.
---

# FPAS benchmarks

Measure with `cargo bench-fpas`. The [suite](../../../docs/bench/suite.toml)
defines workloads; [benchmark documentation](../../../docs/bench/README.md)
explains drivers and measurement boundaries. Use `cargo bench-fpas --help` for
configured groups and `cargo bench-fpas native --help` for native drivers.

## Before and after

1. Select workloads that exercise the proposed change. Save a baseline before
   editing the measured implementation:

   ```text
   cargo bench-fpas save <label> --group <group>
   ```

   Omit `--group` for a full-suite baseline. Snapshots live under
   `.temp-data/bench/`.
2. Implement the authorized change and verify correctness according to
   [AGENTS.md](../../../AGENTS.md). The harness builds the release CLI before
   each suite command and obtains its executable path from Cargo.
3. Compare using the same group selection, benchmark IDs, and workload arguments:

   ```text
   cargo bench-fpas compare <label> --group <group>
   ```

   Keep machine load and power settings comparable. Inspect every row; repeat
   noisy measurements before drawing a conclusion. Save a fresh baseline when
   the selected workloads change. For changes spanning groups, compare a
   full-suite baseline.
4. Record a settled measured improvement or an intentional baseline in
   [history.md](../../../docs/bench/history.md):

   ```text
   cargo bench-fpas record "<short description>" --group <group>
   ```

   Report before/after measurements, meaningful regressions, and whether history
   was updated. Include history with the performance change when committing is
   requested. Measurements support claims only within their workload boundaries.

## Add a workload when needed

- Place FPAS benchmark sources under `examples/pascal/` by theme; use
  [fpas-authoring](../fpas-authoring/SKILL.md) for source edits.
- Register the workload in `docs/bench/suite.toml` with an explicit timeout and
  deterministic result checks. Choose an FPAS or native driver for the cost being measured.
- Add the workload before saving the implementation baseline. Keep default-suite
  workloads bounded and noninteractive; document useful measurement boundaries
  in `docs/bench/README.md`.
