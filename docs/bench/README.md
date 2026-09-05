# FPAS benchmarks

End-to-end performance measurements use Functional Pascal programs under `examples/pascal/`, driven by the release `fpas` CLI. There is no Criterion / Rust `cargo bench` suite for these workloads.

The curated suite lives in [`suite.toml`](suite.toml). Run it with the `fpas-bench` harness (cargo alias `bench-fpas`).

The `tooling` group measures native editor and compiler workloads in release harness child
processes, with the same timeout, save, compare, and record handling. The
`analysis_queries` workload parses one generated editor buffer, warms its semantic
analysis, then creates a fresh query service for each measured analysis request.
Its arguments select the number of queries and declared functions. Parsing and
warmup are excluded from the elapsed time.

The loose-buffer fixture directory exists before timing so manifest discovery stays
inside it. Earlier records used a missing directory and also scanned the changing
parent scratch directory; those absolute timings are not comparable to the isolated
fixture. This harness correction is not a language-service runtime speedup.

`compiler_lowering` parses a generated program with sequential branches, then
measures repeated semantic analysis and IR lowering of the same AST. Its arguments
select the iteration and branch counts. Parsing, warmup, bytecode emission, and
VM execution are excluded, so it measures compiler work directly.

The project query workloads generate a manifest and twenty unopened sibling units,
open the main buffer, and load the real source standard library. One complete analysis
warms each process before timing. `project_queries` measures unchanged queries;
`project_edits` applies three editor revisions and analyzes each; and
`project_overlapping_queries` runs pairs of concurrent warm requests, including
thread creation. They check semantic success and snapshot/cache identity. An untimed
cancelled-refresh guard verifies rejection before work. They do not measure cancellation
of a running semantic analysis or the LSP transport/queue.

```sh
cargo bench-fpas save analysis-before --group tooling
cargo bench-fpas compare analysis-before --group tooling
cargo bench-fpas native --help
```

Committed progress over time lives in [`history.md`](history.md). Agent workflow: [`.agents/skills/fpas-bench/SKILL.md`](../../.agents/skills/fpas-bench/SKILL.md).

`substring_ascii` and `substring_unicode` measure beginning, middle, end, empty,
and full-range slices of a 32,768-scalar string. Input construction and warmup
are excluded; each iteration checks all five results through production FPAS calls.

The `startup` group runs complete release CLI project builds. Each iteration copies
the real source standard library and the headless TUI benchmark program into an
owned scratch project, excluding compiled artifacts. `project_build_cold` times its
first build; `project_build_warm` performs an untimed first build and then times a
second build, checking that the CLI reused the program image. Fixture copying,
warmup, and cleanup are outside the timer. Process startup, project loading, source
validation, unit builds or sidecar loading, and program artifact admission/publication
are included. The resulting program is not executed. These are artifact-cold and
artifact-warm measurements; neither flushes operating-system filesystem caches.

```sh
cargo bench-fpas save startup-before --group startup
cargo bench-fpas compare startup-before --group startup
cargo bench-fpas native project-build 3 warm
```

## Prerequisites

- Quiet machine (avoid heavy background load).
- Same power settings when comparing runs.
- Release CLI: before every benchmark command, the harness runs `cargo build --release -p fpas-cli` and reads the resulting executable path from Cargo's artifact messages. Cargo decides incrementally whether work is required, so an existing but stale release executable is never reused unchecked. This honors `CARGO_TARGET_DIR`, configured target triples, and other Cargo configuration.

Do not treat absolute times as portable across machines. Compare before/after on one machine.

Never record hostnames, usernames, paths, or other machine-identifying metadata in [`history.md`](history.md).

## Track progress (committed history)

After a meaningful performance change, rebuild release `fpas-cli`, then **record** a snapshot into [`history.md`](history.md) (this file is meant to be committed):

```sh
cargo build --release -p fpas-cli
cargo bench-fpas record "after SharedStr char_len cache"
```

VM-only:

```sh
cargo bench-fpas record "after array Length COW" --group vm
```

Newest entries are prepended. Include a short note naming the change so later diffs show what moved which bench.

## Before / after workflow (local, gitignored)

Use this while iterating on a change. JSON under `.temp-data/bench/` is **not** committed.

1. On a known-good checkout (before your change):

   ```sh
   cargo bench-fpas save before
   ```

   Optional: VM microbenches only (faster):

   ```sh
   cargo bench-fpas save before --group vm
   ```

2. Apply your change and rebuild so the release CLI picks up Rust std/VM edits:

   ```sh
   cargo build --release -p fpas-cli
   ```

3. Re-run and compare against the saved baseline:

   ```sh
   cargo bench-fpas compare before
   # or: cargo bench-fpas compare before --group vm
   ```

4. When the win is real, record it into history (see above) and commit `docs/bench/history.md` with the change.

## Commands

| Command | Meaning |
|---------|---------|
| `cargo bench-fpas --help` | Print usage, configured groups, and copyable examples |
| `cargo bench-fpas run` | Run the full configured suite |
| `cargo bench-fpas run --group vm` | VM microbenches only |
| `cargo bench-fpas run --group concurrency` | Task scheduling and resumable callback benchmarks |
| `cargo bench-fpas run --group tui` | Headless TUI bench only |
| `cargo bench-fpas save <label>` | Run and save JSON under `.temp-data/bench/` |
| `cargo bench-fpas compare <label>` | Re-run and print Δ vs a saved label |
| `cargo bench-fpas record <title…>` | Run and prepend a dated entry to [`history.md`](history.md) |

Saved snapshots record whether they contain the complete suite or one `--group`. A comparison must use the same group selection and must find exactly one baseline result for every current benchmark. Re-run `save` for snapshots created before group metadata was introduced or after the selected suite changes.

Compare is **advisory** by default (exit 0 when all benches complete). To fail the process when any bench is slower by more than a percent threshold:

```sh
cargo bench-fpas compare before --fail-on-regression --threshold-pct 10
```

The threshold is a finite, non-negative percentage. Values such as `NaN` or infinity are rejected so the regression gate cannot silently become ineffective.

A baseline and current duration of 0 ms compare as no change. A 0 ms baseline followed by a positive duration is rejected as an invalid measurement; increase that benchmark's workload and save a new baseline.

Snapshot JSON and committed history are written to a flushed same-directory staging file and atomically committed. A failed write or commit leaves the previous file intact.

## Editing the suite

Add or adjust entries in [`suite.toml`](suite.toml):

- `id` — short name used in tables and JSON
- `group` — `vm`, `concurrency`, `tui`, `tooling`, or `startup` (filter with `--group`)
- `path` — `.fpas` program or `.fpasprj` project relative to the repo root
- `driver` — defaults to `fpas`; `language-service`, `language-service-project`, `compiler-lowering`, and `project-build` run native workloads and omit `path`
- `args` — arguments after `--` (usually iteration count)
- `timeout_ms` — required wall-clock limit for the spawned benchmark process; expiry terminates and reaps its entire process tree while retaining captured stdout/stderr in the diagnostic

The curated suite currently uses 120 seconds per entry. Give intentionally longer workloads an explicit larger value instead of disabling the bound.

Excluded from the suite: `examples/pascal/concurrency/task_memory_benchmark.fpas` (long sleep / memory focus).

## Manual single-bench runs

You can still invoke one program directly:

```sh
cargo build --release -p fpas-cli
target/release/fpas run examples/pascal/vm/string_length_benchmark.fpas -- 500000
target/release/fpas run examples/pascal/tui/notes-headless/notes-headless-benchmark.fpasprj -- 250
```

On Windows use `target/release/fpas.exe`. Optional second argument `MAX_MILLIS` turns a slowdown into a panic (see comments at the top of each `*_benchmark.fpas`).

## See also

- [Benchmark history](history.md)
- [Portable register VM acceptance](portable-register-vm.md)
- [Examples README — Performance](../../examples/README.md#performance-benchmarks)
