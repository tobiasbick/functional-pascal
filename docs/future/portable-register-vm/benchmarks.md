# Benchmark and profiling protocol

## Principle

Performance claims require measured release execution of the workload affected by the change. The
canonical harness is `cargo bench-fpas`; do not introduce Criterion or use Rust `cargo bench` for the
FPAS end-to-end suite.

Absolute timings from `docs/bench/history.md` are historical context, not the baseline for a new
machine or checkout.

## Required baseline

Before changing a production hot path:

```sh
cargo build --release -p fpas-cli
cargo bench-fpas save register-vm-before
cargo bench-fpas run --group vm
cargo bench-fpas run --group vm
```

If new benchmark programs are added, add and register them first, then recreate
`register-vm-before`. A compare must use exactly the same suite membership and arguments.

Record the current revision, Rust/FPAS versions, OS family, architecture, and power-mode assumptions in
the local work report. Do not write hostname, username, home directory, or absolute machine paths into
the repository.

## Existing required workloads

All current rows in [`docs/bench/suite.toml`](../../bench/suite.toml) remain required:

| Workload | Main signal |
|---|---|
| `integer_loop` | dispatch, integer operands, locals, branches |
| `array_push` | local aggregate mutation and growth |
| `array_length` | borrowed aggregate intrinsic access |
| `string_concat` | allocation and shared string behavior |
| `string_length` | borrowed string intrinsic access |
| `function_call` | direct function lookup, frames, arguments, returns |
| `array_callbacks` | first-class calls and callback frames |
| `record_update` | field identification and copy-on-write update |
| `unicode_char_at` | string representation and Unicode traversal |
| `task_spawn_wait` | task state, scheduler, frames, synchronization |
| `tui_headless` | whole-runtime collateral regression signal |

Read every row after every comparison. A target win does not hide a regression elsewhere.

## New isolating workloads

Add these only if equivalent coverage does not already exist when implementation starts:

```text
examples/pascal/vm/
  global_access_benchmark.fpas       dense repeated global reads/writes
  record_field_access_benchmark.fpas repeated known-field get/set without record construction
  closure_call_benchmark.fpas        captured and non-capturing first-class calls
  branch_dispatch_benchmark.fpas     predictable and alternating branches
  dynamic_numeric_benchmark.fpas     erased generic numeric operations
```

Each benchmark:

- accepts iteration count and optional `MAX_MILLIS` like neighboring benchmarks;
- starts timing inside the FPAS program so compiler/process startup is excluded;
- computes and checks a result so work cannot be optimized away;
- does not sleep, await interactive input, access the network, or depend on wall-clock ordering;
- runs long enough to avoid zero-millisecond and timer-resolution noise;
- is registered under group `vm` in `docs/bench/suite.toml`;
- has a focused name and measures one dominant mechanism.

Do not add all five mechanically. First inspect the current suite and add only missing signals.

## Measurement sequence per optimization

Use the project skill sequence:

1. `cargo bench-fpas save <phase-label>` before the individual optimization.
2. Apply only that optimization.
3. `cargo build --release -p fpas-cli`.
4. `cargo bench-fpas compare <phase-label> --group vm` while iterating.
5. Repeat the compare at least twice more on a quiet machine.
6. Use the median direction/magnitude and inspect variance.
7. Run a full-suite compare before accepting a cross-cutting change.
8. Revert an optimization that has no repeatable material gain or makes the design worse.

Do not record intermediate noisy experiments in `docs/bench/history.md`.

## Final acceptance thresholds

Compare the settled production register VM with `register-vm-before` on the same machine and power
configuration.

Mandatory unless the user explicitly accepts a measured exception:

- VM-suite geometric-mean throughput: at least 1.5x baseline.
- `integer_loop`: at least 1.5x baseline.
- `function_call`: at least 1.5x baseline.
- `record_update` or its more focused successor: at least 1.25x baseline.
- No individual full-suite row slower by more than 10% after repeated measurement.
- No benchmark loses correctness checks, iterations, or work to manufacture a gain.

Stretch targets:

- at least 2x for integer loops and direct function calls;
- at least 1.5x for known record field access;
- a visible reduction in executable byte size and decode time from replacing JSON payloads.

If mandatory thresholds fail:

1. profile the relevant workload;
2. identify the new dominant functions and allocation/lock sites;
3. implement one narrow measured improvement;
4. repeat comparison;
5. if the architecture still cannot meet the gate, stop and present evidence instead of lowering the
   threshold silently.

## Low-end Chromebook acceptance

The original target is an older Linux Chromebook with a Celeron-class CPU. When that device is
available:

- build or install a release Linux `fpas` appropriate for its architecture;
- use the same source revision and benchmark arguments as the development baseline;
- run the full VM group at least three times before and after;
- report median timings and thermal/power caveats;
- run a Windows-produced compatible `.fpascp` fixture directly;
- do not write device name, username, hostname, or local paths into committed files.

If the Chromebook is unavailable, mark this gate `unverified`; desktop results must not be described as
Chromebook results.

## Profiling

Use a native release profiler appropriate to the current host. Profile the executed FPAS benchmark,
not parser/compiler startup when investigating VM runtime.

Required report fields:

- exact benchmark and arguments;
- release command/binary;
- top inclusive and self-time stacks;
- allocation or lock evidence when relevant;
- hypothesis derived from the profile;
- before/after comparison after the change.

Likely probes, not assumed conclusions:

- dispatch and packed operand decoding;
- repeated helper matches;
- `Value` cloning/reference counts;
- call frame initialization and argument shuffles;
- global locks;
- record copy-on-write detachment;
- intrinsic conversion;
- task queue synchronization.

## Startup and artifact measurements

FPAS in-program timers intentionally exclude process startup and compiler work. Measure artifact
concerns separately:

- `.fpascp` byte size for the same fixture;
- decoder+verifier time over repeated in-process loads in a focused Rust test tool;
- `fpas run fixture.fpascp` process wall time only as a coarse end-to-end observation;
- peak allocation if a supported profiler can measure it.

Never mix these numbers into VM throughput claims.

## Final record

After all correctness and full-suite gates pass:

```sh
cargo bench-fpas record "after portable register VM"
```

Commit `docs/bench/history.md` only when the user authorizes a commit. The closing report includes every
meaningful win and regression, not only the best row.
