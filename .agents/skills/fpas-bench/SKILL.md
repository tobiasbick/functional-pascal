---
name: fpas-bench
description: >
  Runs and records Functional Pascal performance benchmarks with cargo bench-fpas, including
  before/after compare, committed history.md notes, and adding new suite benches when useful.
  Use when optimizing VM/runtime/stdlib performance, proposing or measuring speedups, running
  benches, or when the user mentions bench, benchmark, history.md, throughput, or regression.
---

# FPAS benchmarks

Normative how-to: [`docs/bench/README.md`](../../../docs/bench/README.md).  
Suite: [`docs/bench/suite.toml`](../../../docs/bench/suite.toml).  
Committed log: [`docs/bench/history.md`](../../../docs/bench/history.md).

Do **not** use Criterion / Rust `cargo bench`. Measure with the FPAS harness only.

## When this skill applies

- Implementing or proposing a performance change (VM, std, TUI, …)
- User asks to bench, baseline, compare, or record results
- Finishing a perf task (recording is part of done)

## Required workflow

Copy and track:

```text
Perf Progress:
- [ ] 1. Baseline: cargo bench-fpas save before [--group vm]
- [ ] 2. Implement the change (proposal first if user asked to propose)
- [ ] 3. cargo build --release -p fpas-cli
- [ ] 4. Compare: cargo bench-fpas compare before [--group vm]
- [ ] 5. Inspect full suite if the change could affect other benches
- [ ] 6. Record: cargo bench-fpas record "<short note of what changed>"
- [ ] 7. Summarize deltas (wins + any regressions) for the user
```

### 1 — Baseline (local, gitignored)

Before editing hot paths:

```sh
cargo bench-fpas save before
# faster while iterating on VM-only work:
cargo bench-fpas save before --group vm
```

Writes `.temp-data/bench/before.json` (not committed).

### 2 — Change

- Prefer a short proposal when the user asked to propose first; implement after “go”.
- Touch only what the speedup needs. Match AGENTS.md / change-checklist for docs/tests.
- **New benchmarks are allowed** when they isolate a hot path the suite does not cover yet (see below).

### New benchmarks

Add a new `*_benchmark.fpas` when it is an advantage — e.g. the existing suite cannot show the win, or a distinct bottleneck needs its own before/after.

When adding one:

1. Put it under `examples/pascal/` by theme (`vm/`, `tui/`, …), same style as existing benches (`MonotonicMillis`, optional `MAX_MILLIS`).
2. Register it in [`docs/bench/suite.toml`](../../../docs/bench/suite.toml) (`id`, `group`, `path`, `args`) when it should run in the harness.
3. Mention it in [`docs/bench/README.md`](../../../docs/bench/README.md) / examples README only if useful; keep suite.toml authoritative.
4. Baseline with `save` **after** the new bench exists (or record a first history row that introduces it) so later compares are meaningful.
5. Do not add benches that hang, sleep long, or need interactive input to the default suite.

### 3 — Rebuild

Rust std/VM changes need a release CLI (stdlib sync / release binary):

```sh
cargo build --release -p fpas-cli
```

### 4 — Compare

```sh
cargo bench-fpas compare before
# or: cargo bench-fpas compare before --group vm
```

Read **every** row. A win on the target bench with a clear loss elsewhere is incomplete — call it out and decide with the user before recording.

Optional gate:

```sh
cargo bench-fpas compare before --fail-on-regression --threshold-pct 10
```

### 5 — Full suite when needed

If the change is not obviously isolated to one group, run without `--group` before recording so TUI/other benches are visible.

### 6 — Record (committed history)

**Required** after a real measured win (or an intentional documented baseline). Title = one short sentence of what changed:

```sh
cargo bench-fpas record "after SharedStr char_len cache"
cargo bench-fpas record "after flat VM Op dispatch" --group vm
```

- Prepends a dated section to `docs/bench/history.md` (date, group, note, timings only — **no host/user/path**)
- Commit `docs/bench/history.md` with the perf change (when the user asks to commit)
- Do not record noisy flailing runs; record the settled after-state

### 7 — Report to the user

Include:

- Target bench before → after (ms and/or throughput)
- Other suite rows that moved meaningfully (regressions too)
- That history was updated (or why not: abandoned change / no measurable win)

## Commands cheat sheet

| Command | Purpose |
|---------|---------|
| `cargo bench-fpas run [--group vm\|tui]` | One-shot suite |
| `cargo bench-fpas save <label>` | Local JSON baseline |
| `cargo bench-fpas compare <label>` | Δ vs local baseline |
| `cargo bench-fpas record "<note>"` | Append to `docs/bench/history.md` |

## Rules

1. Always **save → change → rebuild → compare** for perf work; never claim a speedup without numbers.
2. Always **record** a settled win with a **short note of what changed** so history stays auditable.
3. Watch for **collateral regressions** on other benches in the same compare/record table.
4. Absolute times are machine-specific; compare on the same machine and power settings.
5. **Never** write hostnames, usernames, home paths, or other machine-identifying metadata into `history.md`, skills, docs, commits, or reports.
6. **Create new benchmarks** when they better expose the hotspot; register useful ones in `suite.toml`.
7. Excluded from default suite: `task_memory_benchmark.fpas` (memory/sleep focus).
8. Link detail to `docs/bench/`; do not duplicate suite arg tables in this skill.
9. This is a **hobby project**: Rust/runtime/tooling may be rebuilt freely when it helps. **FPAS language** semantics/syntax change **only after explicit user agreement**.

