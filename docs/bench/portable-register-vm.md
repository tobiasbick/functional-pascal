# Portable register VM acceptance

P11 closes the portable register VM rewrite with measured performance, Windows-native correctness,
and durable implementation evidence. The accepted implementation is the only compiler, bytecode,
artifact, and VM path. This work changes neither FPAS syntax nor FPAS semantics.

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
- local `Std.Array.Push` lowering uses a verified internal `ArrayPush` operation that takes unique
  storage when available and otherwise preserves array copy-on-write aliases;
- non-task intrinsics borrow their argument window, and hosted higher-order array operations reuse a
  callback worker and its register allocation across callback invocations;
- task-spawn block widths account for the complete argument window, preserving branch addresses in
  loops.

`ArrayPush` is an internal IR and bytecode optimization. Source-visible `Std.Array.Push` behavior,
including copy-on-write aliasing, is unchanged.

## Profile evidence

Process-scoped sampling profiles used a release Windows `fpas.exe` with Rust debug information. The
profiled commands executed the production `fpas run` path for
`integer_loop_benchmark.fpas -- 50000000` and `function_call_benchmark.fpas -- 30000000`.
Absolute profiler-run timings were not used for acceptance because other desktop load was present;
the sample proportions below describe only the profiled FPAS process.

For the integer loop, 8,319 main-thread samples showed:

| stack | self | inclusive | interpretation |
|---|---:|---:|---|
| `Worker::dispatch_one` | 41.44% | 93.65% | packed decode and opcode dispatch dominate the loop |
| `Worker::execute_binary_integer` | 20.83% | 22.30% | integer helper and result handling remain material |
| `Worker::execute_divide_integer` | 7.54% | 8.47% | benchmark arithmetic includes division |
| integer comparison | 7.09% | 7.46% | loop control remains visible |
| `Value::clone` | 5.61% | below top inclusive rows | value movement is still measurable |

For the call workload, 10,367 main-thread samples showed:

| stack | self | inclusive | interpretation |
|---|---:|---:|---|
| `Worker::dispatch_one` | 32.60% | 94.55% | call and callee instructions share the dispatch cost |
| `Value::clone` | 14.19% | below top inclusive rows | argument and result values still require value semantics |
| `Vec<Value>::extend_with` | 11.10% | 16.18% | callee frame growth remains a future optimization target |
| `Worker::execute_binary_integer` | 10.08% | below top inclusive rows | the called function performs integer work |
| `Worker::enter_call` | 2.36% | 23.79% | frame preparation is the dominant call-specific stack |
| `Vec<Value>::resize` | 3.30% | 19.52% | frame initialization remains visible after removing temporary argument vectors |

The profiles supported the accepted decode, register-move, argument-window, callback reuse, and
copy-on-write changes. They also show that further call-frame storage work would need a separate
measured change rather than being hidden in P11 acceptance.

The one final profile-driven change removed the unused copy-window register from one-argument call
frames. Three loaded-host spot checks moved the `function_call` median from 930 ms to 851 ms, an
8.5% reduction. Those spot checks decide the implementation step only; the three full-suite medians
below remain the recorded P11 evidence.

## Benchmark method

The committed suite and workload arguments were unchanged. The baseline is the complete
`.temp-data/bench/register-vm-before.json` snapshot recorded before the register rewrite. After the
settled release build, the complete suite was compared against that baseline three times on the same
Windows host. The user explicitly directed P11 to continue while a game was using several CPU cores.
The final runs therefore capture concurrent desktop/game load that was not synchronized with the
earlier baseline. They prove workload correctness and preserve the requested measurement record, but
their absolute deltas mix VM changes with host contention and must not be presented as quiet-machine
throughput. Each row reports the median of the three elapsed times. Speedup is `baseline / median`;
positive change is the throughput-equivalent improvement `(speedup - 1) * 100`.

| group | benchmark | baseline ms | run 1 ms | run 2 ms | run 3 ms | median ms | speedup | change |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| vm | `integer_loop` | 12,143 | 10,132 | 10,928 | 10,091 | 10,132 | 1.199x | +19.8% |
| vm | `global_access` | 1,011 | 703 | 714 | 685 | 703 | 1.438x | +43.8% |
| vm | `record_field_access` | 2,807 | 2,308 | 2,534 | 2,428 | 2,428 | 1.156x | +15.6% |
| vm | `closure_call` | 631 | 640 | 651 | 676 | 651 | 0.969x | -3.1% |
| vm | `branch_dispatch` | 4,559 | 4,127 | 5,336 | 4,213 | 4,213 | 1.082x | +8.2% |
| vm | `dynamic_numeric` | 1,199 | 1,184 | 1,236 | 1,141 | 1,184 | 1.013x | +1.3% |
| vm | `array_push` | 209 | 253 | 290 | 261 | 261 | 0.801x | -19.9% |
| vm | `array_length` | 79 | 115 | 137 | 113 | 115 | 0.687x | -31.3% |
| vm | `string_concat` | 3,612 | 3,441 | 4,933 | 3,577 | 3,577 | 1.010x | +1.0% |
| vm | `string_length` | 77 | 105 | 103 | 118 | 105 | 0.733x | -26.7% |
| vm | `function_call` | 1,095 | 941 | 1,066 | 936 | 941 | 1.164x | +16.4% |
| vm | `array_callbacks` | 1,249 | 1,296 | 1,542 | 1,552 | 1,542 | 0.810x | -19.0% |
| vm | `record_update` | 715 | 520 | 691 | 645 | 645 | 1.109x | +10.9% |
| vm | `unicode_char_at` | 1,303 | 1,692 | 2,145 | 1,949 | 1,949 | 0.669x | -33.1% |
| concurrency | `task_spawn_wait` | 1,005 | 1,038 | 955 | 1,016 | 1,016 | 0.989x | -1.1% |
| tui | `tui_headless` | 10,182 | 14,199 | 15,800 | 13,266 | 14,199 | 0.717x | -28.3% |

The VM geometric mean uses the fourteen `vm` group rows only. Concurrency and TUI remain full-suite
regression gates but are not included in that geometric mean.

## Acceptance gates

| gate | required | measured | outcome |
|---|---:|---:|---|
| VM geometric-mean throughput | at least 1.5x | 0.964x | measured exception accepted for concurrent game load |
| `integer_loop` | at least 1.5x | 1.199x | measured exception accepted for concurrent game load |
| `function_call` | at least 1.5x | 1.164x | measured exception accepted for concurrent game load |
| `record_update` | at least 1.25x | 1.109x | measured exception accepted for concurrent game load |
| worst full-suite regression | no worse than -10% | `unicode_char_at` -33.1% | measured exception accepted for concurrent game load |
| unchanged workloads and checks | required | unchanged and all completed | passed |

All three threshold-enabled comparisons returned exit code 2 because at least one row exceeded the
10% regression threshold. The mandatory numerical gates are therefore not claimed as passing. The
user explicitly instructed P11 to ignore the concurrently running game and continue to completion;
this is the user-accepted measured exception permitted by the P11 contract, not a silent threshold
reduction. A later quiet-machine comparison may replace these contaminated numbers, but it is not a
claim made by this acceptance record.

No benchmark source, iteration count, correctness check, or suite argument was weakened to obtain a
gain. The settled snapshot is also recorded in [Benchmark history](history.md) with the title
`after portable register VM`.

## Rejected experiments

The following measured experiments were removed because they did not produce a repeatable net win or
made the runtime less focused:

- pooled or lazy Unicode character caches;
- dedicated array-length and string-length opcodes;
- hosted intrinsic match routing and broader intrinsic inlining;
- generalized contiguous multi-argument call windows and parameter-prologue folding;
- in-place scalar `Value` writes, which increased generated code and regressed the suite;
- Thin LTO with one code-generation unit, which made this workload slower;
- extra caching or compatibility layers retained only for the completed migration.

## Windows verification

| OS | architecture | P11 status | evidence |
|---|---|---|---|
| Windows | x86-64 | verified | workspace build/test, Clippy, FPAS suite, canonical artifact tests, direct `.fpascp`, native application |
| Windows | ARM64 | unverified | native host not used |
| Linux | x86-64 and ARM64 | unverified | outside the final Windows-only scope |
| macOS | x86-64 and ARM64 | unverified | outside the final Windows-only scope |
| FreeBSD | x86-64 and ARM64 | unverified | outside the final Windows-only scope |
| ChromeOS Linux environment | Celeron-class target | unverified | device unavailable for P11 |

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
The completed future plan was removed after this record, tests, and benchmark history became the
durable source of truth.
