# Portable register VM

The portable register VM is the only compiler, bytecode, artifact, and VM path. This document
records its measured performance, Windows-native correctness, and implementation evidence. The
architecture changes neither FPAS syntax nor FPAS semantics.

## Acceptance scope

The final acceptance run is intentionally limited to the available Windows 11 x86-64 host. Windows
ARM64 and every Linux, macOS, FreeBSD, and Chromebook row are unverified in this acceptance record;
no support claim is inferred from Rust target availability or from external crate metadata. Native
applications remain host-specific, while `.fpascp` program images remain target-independent data as
documented in [Compiled program images](../pascal/program-structure/compiled-programs.md).

## Settled implementation

The performance work retained the verified register architecture and made focused changes in the
remaining hot paths:

- validated packed instructions expose unchecked-form payload accessors to the already verified VM,
  avoiding repeated instruction-form validation during dispatch;
- register access and scalar operand helpers are inlined at the dispatch boundary;
- direct and first-class calls copy arguments and captures directly from borrowed register windows
  into the callee frame instead of first allocating temporary vectors;
- one-argument calls use the argument's existing register without reserving an unused copy-window
  slot, and single-use producer results followed by a local write are allocated directly to that
  local register;
- local `Items.PushInPlace(Value)` lowering uses a verified internal `ArrayPush` operation that takes
  unique storage when available and otherwise preserves array copy-on-write aliases;
- index-only designator writes into global aggregates use a verified global-index-path instruction
  and mutate a uniquely owned root in place while preserving aliases, bounds checks, and error
  diagnostics;
- non-task intrinsics borrow their argument window, and hosted higher-order array operations reuse a
  callback worker and its register allocation across callback invocations;
- each worker retains its physical register allocation between calls and callbacks while clearing
  the active prefix at frame boundaries, so live values are released without repeatedly growing the
  register vector;
- task-spawn block widths account for the complete argument window, preserving branch addresses in
  loops.

`ArrayPush` is an internal IR and bytecode optimization. Source-visible
`Items.PushInPlace(Value)` behavior, including copy-on-write aliasing, is unchanged.

## Benchmark method

The committed suite and workload arguments were unchanged. The baseline is the complete
`.temp-data/bench/register-vm-before.json` snapshot recorded before the register rewrite. After the
settled release build, the complete suite was compared against that baseline three times on the same
Windows host. Five pre-run processor probes reported 0% to 7% total utilization, with no sustained
competing workload. Each row reports the median of the three elapsed times. Speedup is
`baseline / median`; positive change is the throughput-equivalent improvement
`(speedup - 1) * 100`.

| group | benchmark | baseline ms | run 1 ms | run 2 ms | run 3 ms | median ms | speedup | change |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| vm | `integer_loop` | 12,143 | 5,361 | 5,292 | 5,156 | 5,292 | 2.295x | +129.5% |
| vm | `global_access` | 1,011 | 473 | 475 | 469 | 473 | 2.137x | +113.7% |
| vm | `record_field_access` | 2,807 | 1,611 | 1,568 | 1,582 | 1,582 | 1.774x | +77.4% |
| vm | `closure_call` | 631 | 377 | 372 | 391 | 377 | 1.674x | +67.4% |
| vm | `branch_dispatch` | 4,559 | 2,729 | 2,633 | 2,687 | 2,687 | 1.697x | +69.7% |
| vm | `dynamic_numeric` | 1,199 | 667 | 678 | 666 | 667 | 1.798x | +79.8% |
| vm | `array_push` | 209 | 174 | 173 | 186 | 174 | 1.201x | +20.1% |
| vm | `array_length` | 79 | 80 | 76 | 74 | 76 | 1.039x | +3.9% |
| vm | `string_concat` | 3,612 | 2,178 | 2,170 | 2,147 | 2,170 | 1.665x | +66.5% |
| vm | `string_length` | 77 | 69 | 68 | 68 | 68 | 1.132x | +13.2% |
| vm | `function_call` | 1,095 | 540 | 546 | 545 | 545 | 2.009x | +100.9% |
| vm | `array_callbacks` | 1,249 | 751 | 731 | 722 | 731 | 1.709x | +70.9% |
| vm | `record_update` | 715 | 405 | 402 | 388 | 402 | 1.779x | +77.9% |
| vm | `unicode_char_at` | 1,303 | 1,247 | 1,217 | 1,225 | 1,225 | 1.064x | +6.4% |
| concurrency | `task_spawn_wait` | 1,005 | 584 | 566 | 579 | 579 | 1.736x | +73.6% |
| tui | `tui_headless` | 10,182 | 3,442 | 3,439 | 3,499 | 3,442 | 2.958x | +195.8% |

The VM geometric mean uses the fourteen `vm` group rows only. Concurrency and TUI remain full-suite
regression gates but are not included in that geometric mean.

## Acceptance gates

| gate | required | measured | outcome |
|---|---:|---:|---|
| VM geometric-mean throughput | at least 1.5x | 1.593x | passed |
| `integer_loop` | at least 1.5x | 2.295x | passed |
| `function_call` | at least 1.5x | 2.009x | passed |
| `record_update` | at least 1.25x | 1.779x | passed |
| worst full-suite regression | no worse than -10% | no median regression | passed |
| unchanged workloads and checks | required | unchanged and all completed | passed |

An additional `compare register-vm-before --fail-on-regression --threshold-pct 10` run returned exit
code 0. The quiet three-run medians pass every mandatory numerical gate without an exception or a
reduced threshold.

No benchmark source, iteration count, correctness check, or suite argument was weakened to obtain a
gain.

## Windows verification

| OS | architecture | status | evidence |
|---|---|---|---|
| Windows | x86-64 | verified | workspace build/test, Clippy, FPAS suite, canonical artifact tests, direct `.fpascp`, native application |
| Windows | ARM64 | unverified | native host not used |
| Linux | x86-64 and ARM64 | unverified | outside the final Windows-only scope |
| macOS | x86-64 and ARM64 | unverified | outside the final Windows-only scope |
| FreeBSD | x86-64 and ARM64 | unverified | outside the final Windows-only scope |
| ChromeOS Linux environment | Celeron-class target | unverified | device unavailable for verification |

The Windows gates executed:

- `cargo fmt --all -- --check`;
- `cargo build`;
- `cargo test --workspace`;
- `cargo clippy --all-targets --all-features --locked -- -D warnings`;
- `fpas fmt --check --list examples/ tests/ apps/`;
- `fpas test tests/`: 388 passed, 1 skipped, 0 failed;
- `git diff --check`.

The final `units-basic.fpascp` SHA-256 digest is
`8322BC783A139199164DDAD4F714345A536BA1241F807A82E4E1ABA07BF4E5D1`. Both direct artifact execution
and the Windows native application produced:

```text
Short name: input=135, clamped=100
Qualified name: scaled=42
Clamp direct call: 0
```

The canonical artifact tests cover deterministic bytes and digest, decoder/verifier admission,
source-less `.fpascp` execution, and source-less native Windows application execution. The manual
artifact smoke test uses the checked-in `units-basic` project, runs the emitted `.fpascp` directly,
and records only the artifact digest and output—not a machine path or hostname.

## Documentation classification

The implementation is an internal compiler/bytecode/VM performance change. Current FPAS language,
standard-library, project, CLI, and compiled-program documentation remains semantically correct.
