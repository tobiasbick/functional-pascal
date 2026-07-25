# FPAS benchmarks

End-to-end performance measurements use Functional Pascal programs under `examples/pascal/`, driven by the release `fpas` CLI. There is no Criterion / Rust `cargo bench` suite for these workloads.

The curated suite lives in [`suite.toml`](suite.toml). Run it with the `fpas-bench` harness (cargo alias `bench-fpas`).

Committed progress over time lives in [`history.md`](history.md). Agent workflow: [`.agents/skills/fpas-bench/SKILL.md`](../../.agents/skills/fpas-bench/SKILL.md).

## Prerequisites

- Quiet machine (avoid heavy background load).
- Same power settings when comparing runs.
- Release CLI: the harness builds `fpas-cli` in release mode if `target/release/fpas` is missing.

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
| `cargo bench-fpas run` | Run the full suite (`vm` + `tui`) |
| `cargo bench-fpas run --group vm` | VM microbenches only |
| `cargo bench-fpas run --group tui` | Headless TUI bench only |
| `cargo bench-fpas save <label>` | Run and save JSON under `.temp-data/bench/` |
| `cargo bench-fpas compare <label>` | Re-run and print Δ vs a saved label |
| `cargo bench-fpas record <title…>` | Run and prepend a dated entry to [`history.md`](history.md) |

Compare is **advisory** by default (exit 0 when all benches complete). To fail the process when any bench is slower by more than a percent threshold:

```sh
cargo bench-fpas compare before --fail-on-regression --threshold-pct 10
```

## Editing the suite

Add or adjust entries in [`suite.toml`](suite.toml):

- `id` — short name used in tables and JSON
- `group` — `vm` or `tui` (filter with `--group`)
- `path` — `.fpas` file relative to the repo root
- `args` — arguments after `--` (usually iteration count)

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
