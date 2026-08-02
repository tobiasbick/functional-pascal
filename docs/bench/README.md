# FPAS benchmarks

End-to-end performance measurements use Functional Pascal programs under `examples/pascal/`, driven by the release `fpas` CLI. There is no Criterion / Rust `cargo bench` suite for these workloads.

The curated suite lives in [`suite.toml`](suite.toml). Run it with the `fpas-bench` harness (cargo alias `bench-fpas`).

Committed progress over time lives in [`history.md`](history.md). Agent workflow: [`.agents/skills/fpas-bench/SKILL.md`](../../.agents/skills/fpas-bench/SKILL.md).

## Prerequisites

- Quiet machine (avoid heavy background load).
- Same power settings when comparing runs.
- Release CLI: the harness asks `cargo metadata` for the effective target directory and builds `fpas-cli` in release mode if the corresponding `release/fpas` executable is missing. This honors `CARGO_TARGET_DIR` and Cargo configuration.

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
| `cargo bench-fpas run` | Run the full suite (`vm` + `concurrency` + `tui`) |
| `cargo bench-fpas run --group vm` | VM microbenches only |
| `cargo bench-fpas run --group concurrency` | Task scheduling benchmark only |
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
- `group` — `vm`, `concurrency`, or `tui` (filter with `--group`)
- `path` — `.fpas` file relative to the repo root
- `args` — arguments after `--` (usually iteration count)
- `timeout_ms` — required wall-clock limit for the spawned benchmark process; expiry terminates and reaps the process while retaining captured stdout/stderr in the diagnostic

The curated suite currently uses 120 seconds per entry. Give intentionally longer workloads an explicit larger value instead of disabling the bound.

Excluded from the suite: `examples/pascal/concurrency/task_memory_benchmark.fpas` (long sleep / memory focus).

## Manual single-bench runs

You can still invoke one program directly:

```sh
cargo build --release -p fpas-cli
target/release/fpas run examples/pascal/vm/string_length_benchmark.fpas -- 500000
```

On Windows use `target/release/fpas.exe`. Optional second argument `MAX_MILLIS` turns a slowdown into a panic (see comments at the top of each `*_benchmark.fpas`).

## See also

- [Benchmark history](history.md)
- [Examples README — Performance](../../examples/README.md#performance-benchmarks)
