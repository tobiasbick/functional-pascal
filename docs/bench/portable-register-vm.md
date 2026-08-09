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

For the call workload, 10,367 main-thread samples taken before reusable register storage showed:

| stack | self | inclusive | interpretation |
|---|---:|---:|---|
| `Worker::dispatch_one` | 32.60% | 94.55% | call and callee instructions share the dispatch cost |
| `Value::clone` | 14.19% | below top inclusive rows | argument and result values still require value semantics |
| `Vec<Value>::extend_with` | 11.10% | 16.18% | repeated callee frame growth was still material |
| `Worker::execute_binary_integer` | 10.08% | below top inclusive rows | the called function performs integer work |
| `Worker::enter_call` | 2.36% | 23.79% | frame preparation is the dominant call-specific stack |
| `Vec<Value>::resize` | 3.30% | 19.52% | frame initialization remained visible after removing temporary argument vectors |

The profiles supported the accepted decode, register-move, argument-window, callback reuse, and
copy-on-write changes. The later reusable-register-storage change addressed the observed frame-growth
cost as a separately measured follow-up.

The one final P11 profile-driven change removed the unused copy-window register from one-argument
call frames. Three loaded-host spot checks moved the `function_call` median from 930 ms to 851 ms, an
8.5% reduction. Those spot checks decided that implementation step only. The quiet full-suite
medians below supersede the earlier contaminated P11 acceptance numbers.

### TUI follow-up profile

The current `tui_headless` workload was sampled again after the global-index-path and reusable
register-storage changes. A process-scoped, main-thread release profile executed the unchanged
`headless_render_benchmark.fpas -- 500` workload in 3,458 ms, or 144 frames/s. Its most useful stacks
were:

| stack | self | inclusive | interpretation |
|---|---:|---:|---|
| `Worker::dispatch_one` | 28.22% | 93.65% | bytecode dispatch remains the broad runtime cost |
| `Value` drop glue | 14.27% | 14.30% | releasing aggregate-rich temporary values is now a primary cost |
| numeric hosted calls | 5.13% | 16.24% | TUI geometry and layout execute many numeric calls |
| synchronous callback calls | 4.55% | 5.27% | higher-order array operations retain visible callback overhead |
| aggregate-window iteration | 4.05% | below top inclusive rows | aggregate traversal remains material |
| aggregate-window collection | 2.93% | 8.81% | building result aggregates remains material |

Optimized symbol ranges merge some neighboring functions, so smaller symbol-labelled rows are not
used as exact source attribution. The important negative result is reliable: neither
`Vec<Value>::resize` nor `Vec<Value>::extend_with` remains in the leading stacks, whereas they were
19.52% and 16.18% inclusive in the earlier call profile. Reusable register storage removed that
identified bottleneck.

A separate public-API microbenchmark rebuilt the same 43-element view tree 500 times after 50
warm-up iterations. Three runs took 12 ms, 12 ms, and 11 ms. View construction is therefore below
0.4% of the complete 3,458 ms render workload and is not the remaining TUI bottleneck. Validation,
arrangement, and paint are internal `Std.Tui` units and are intentionally not exported for direct
source-level timing. The native profile instead points to dispatch, value destruction, numeric and
callback calls, and aggregate traversal as the next areas to investigate; this record makes no new
optimization claim for them.

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
gain. The settled snapshot is also recorded in [Benchmark history](history.md) with the title
`quiet portable register VM acceptance`.

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
