# VM performance plan

Proposed optimizations to make the register VM (`crates/fpas-vm`) execute FPAS programs faster.
None of these change the FPAS language. Bytecode, compiler output, and VM internals may change
freely. Speedup figures are estimates, not measurements.

Out of scope: JIT and AOT compilation. FPAS stays an interpreted bytecode VM.

- **Part 1** (steps 1–7): incremental work on the current VM.
- **Part 2** (steps 8–16): deeper changes, up to a rewrite of the value model and the VM core.

## Starting point

- `integer_loop` runs about 7.7 M iterations/s (see [`docs/bench/history.md`](../bench/history.md)).
  At roughly 6–8 instructions per iteration, that is about 50 M instructions/s — well below what
  fast register interpreters (e.g. Lua) reach.
- Other slow spots from the history: `string_concat` (~2.3 M/s), `record_update` (~2.7 M/s),
  `unicode_char_at` (~2.4 M/s), `function_call` (~8.8 M/s).
- The main overhead is per-instruction bookkeeping in the dispatch loop, not the operations
  themselves.

## 1. Release build profile — done

The root `Cargo.toml` now uses:

```toml
[profile.release]
lto = "fat"
codegen-units = 1
```

`panic = "abort"` remains optional and was not enabled. The full-suite before/after
measurements are in [`docs/bench/history.md`](../bench/history.md): several VM workloads improved,
while some call and dynamic-numeric workloads slowed slightly. The separate Mandelbrot paint
measurements varied too much to establish a direction. PGO remains a possible later experiment.

## 2. Lower per-instruction overhead in the dispatch loop — done

Implemented in `crates/fpas-vm/src/vm/dispatch.rs`, `dispatch/opcodes.rs`, and
`worker/run.rs`:

- `dispatch_batch` executes up to 256 scheduling steps per call instead of returning a
  `DispatchStep` per instruction. The scheduler abort check and the task yield budget are
  applied once per batch; the yield budget stays exact because the batch size is capped by the
  remaining timeslice.
- Suspension and hosted-callback readiness are checked only after the opcodes that can change
  them: `Return`, `Intrinsic`, `SpawnTask`, `SpawnDetachedTask`, and `Yield`. A batch ends early
  when a callback continuation is ready, so the next batch resumes it first.
- Verified bytecode is pre-decoded into `DecodedInstruction` (`fpas-bytecode`), which keeps the
  8-byte packed size: ABC operands `b`/`c` share the ABx `bx` slot. The hot loop no longer
  decodes opcodes fallibly.
- The instruction counter uses a plain wrapping increment instead of a checked add with an error
  path. Instruction-pointer conversion is no longer checked per instruction.
- Diagnostic construction (`diagnostics::at_address`, `diagnostics::internal`, register bound
  errors) is `#[cold]` / `#[inline(never)]`.

Kept on purpose: `current_address` is still stored once per instruction (a single 4-byte store).
Computing it lazily from `ip` would give wrong addresses for errors raised after a call or jump
has already moved `ip`, and the debugger reads it as the stopped instruction.

## 3. Remove duplicate runtime checks on verified code — done

- Branches set `ip` directly; the verifier keeps targets inside the current function.
- The register vectors now hold exactly the active window (`registers.len()` is the active
  register count; releasing a frame truncates and keeps the allocation). A register read or write
  therefore needs one length check instead of an active-prefix check plus an index check.
- Hot handlers (`Move`, `LoadConstant`, `LoadUnit`, branches, `Return`, `Panic`, typed integer
  and value operations) read and write raw verified operands and skip the redundant
  `Register::new` sentinel check.
- The proposed "integer overwrites integer" fast path was dropped: dropping a scalar `Value` is
  already a single discriminant check, so a separate fast path only adds a branch.

Measured against the `dispatch-before` baseline (full suite, local runs; final numbers in the
history): `integer_loop` 5949 -> ~3860 ms, `branch_dispatch` 2918 -> ~1870 ms,
`function_call` 695 -> ~550 ms, `closure_call` 481 -> ~355 ms, `intrinsic_dispatch`
1395 -> ~1020 ms. `array_callbacks` regressed: an alternating A/B run against the previous build
gave ~865 ms before and ~900 ms after (see step 3.1).

## 3.1 Follow-ups to steps 2 and 3 — done

Work found while implementing and measuring steps 2 and 3 that no later step covers:

- **`array_callbacks` regression.** Synchronous root callbacks run only 2–3 instructions per
  invocation, so fixed per-run costs dominate. Removing the per-batch `Arc` clone, clearing
  callback registers in place, and the two items below brought it back to the previous level
  (alternating A/B runs: 858–869 ms before, 839–856 ms after).
- **Boxed `VmError`.** `VmError` is now `Box<Diagnostic>`, so `Result<_, VmError>` stays two
  words wide. The diagnostic constructors in `vm/diagnostics.rs` return the boxed form, and
  `fpas_std` diagnostics are boxed where they enter the VM. The public `fpas_vm::VmError` alias is
  boxed as well; all workspace consumers compile unchanged through `Deref`.
- **Prepared string constants.** `VerifiedExecutable::string_constant` holds one shared
  `SharedStr` per string constant, built during verification. `LoadConstant` clones it instead
  of allocating; copy-on-write keeps the prepared value unchanged. Function-name allocation stays
  with step 8.
- **Flaky CLI test.** `source_review_redirects_preserve_exact_request_paths` read from an accepted
  socket that inherited the listener's nonblocking mode on Windows, so a request that had not
  arrived yet failed with `WouldBlock`. The fixture now switches accepted streams to blocking.
- **Bench baseline workflow.** The `fpas-bench` skill and `docs/bench/README.md` state that
  `compare` must use the same group as `save`.
- **Results** are recorded in [`docs/bench/history.md`](../bench/history.md).

Not part of this step: function-name allocation (step 8), merged register value and
initialization storage (step 16), and lazy `current_address` (kept on purpose, see step 2).

## 4. Superinstructions — done

- Integer counting loops use `ForLoop` for the inclusive terminal check, counter update, and
  branch. The counter is not advanced at the bound, including at the integer extremes. Boolean
  counting loops retain their existing lowering. The opcode consumes two verified jump payload
  words but takes one dispatch step.
- Adjacent typed integer comparisons and conditional branches use `BranchIf*Integer` opcodes.
  The comparison still writes its Boolean destination for debugger and register semantics; the
  following verified branch word supplies the target and is consumed in the same dispatch step.
- Adjacent integer literals select `AddIntegerImm` or `DivideIntegerImm` when they fit
  a signed 16-bit operand. The literal load remains available as a separate debugger step.
- `fpas-bytecode` validates register operands, payload shape, and both control-flow targets.
  Compiler, verifier, and VM tests cover emitted opcodes, malformed payloads, signed immediates,
  division errors, and inclusive loop boundaries.

Alternating three-run comparison against the step-5-only executable: `integer_loop` median
3198 -> 1718 ms, `branch_dispatch` 1502 -> 1136 ms, `dynamic_numeric` 585 -> 454 ms, and
`function_call` 498 -> 426 ms. The original full-VM snapshot showed broad unrelated variation,
so these paired runs are the attribution evidence.

## 5. Typed fast paths for real and boolean operations — done

Real arithmetic, real and string comparisons, and Boolean equality/AND/OR have direct typed
handlers. Unexpected operand types use the existing generic operations. Division by zero and NaN
ordering retain their diagnostics. String ordering now lowers to the existing typed string
bytecode instead of an invalid integer comparison IR operation.

Alternating three-run comparison against the pre-change executable on the new
`typed_scalar_benchmark.fpas`: `typed_real` median 687 -> 607 ms and `typed_string` 582 -> 493 ms;
`typed_boolean` was effectively unchanged at 519 -> 515 ms. All runs retained the same checksums.
The existing `dynamic_numeric` benchmark uses generic numeric operations, so it does not isolate
the step-5 handlers.

## 6. Targeted hotspots — done

- **`record_update`:** `UpdateRecord` validates its override slots, then moves the record out of
  its register, so a uniquely owned body is updated in place instead of cloned; the override
  window is read without an intermediate vector. Record operations moved to
  `vm/execute/aggregates/records.rs`.
- **`string_concat`:** `SharedStr::append` grows a uniquely owned buffer in place. The compiler
  sets `ConcatString` auxiliary 1 when the left operand is a temporary read for the last time
  (from the register allocator's last-use data; parameters, locals, and values stored directly
  into locals never qualify). The VM then releases the destination's previous value and moves the
  left buffer out, so `S := S + X` and chained concatenations append without a copy. The verifier
  accepts auxiliary 0 or 1.
- **`unicode_char_at`:** `SharedStr::byte_offset` / `char_at` answer ASCII strings in O(1) from the
  cached scalar count and use a sampled offset index (every 64 scalars, built once per string) for
  longer non-ASCII strings. `Std.Str.CharAt`, string indexing, and `Std.Str.Substring` use it.
- **`Value::Cell`:** not changed. Values cross task and pool-thread boundaries, so `Value` must stay
  `Send`; a lock-free cell in safe Rust needs worker-local value storage first (steps 9 and 10).
- **Calls:** not changed. No profiler is available in this environment, and the step requires
  profiling before touching frame setup. Argument copying is addressed by step 13.

## 7. Global allocator — done

The `fpas` and `fpas-bench` binaries use `mimalloc` as the global allocator. Measurements are in
[`docs/bench/history.md`](../bench/history.md): allocation-heavy VM and TUI workloads improved by
8–23 %, `string_search` regressed by about 10 %, and the in-process tooling benches showed no
clear direction (`project_queries` tended to be slower).

# Part 2 — deeper changes

## 8. No per-closure name allocation — done

`make_closure` and function constants no longer copy the function name into a new `String`.
`VerifiedExecutable::function_name` holds one shared `Arc<str>` per function, and function values
clone that pointer. The name stays in the value because `WriteLn` of a function value prints
`<function Name>` and `Value` formatting has no access to the executable.

## 9. Non-atomic reference counting per worker

All shared values (`SharedArray`, `SharedStr`, records, enums, functions) are backed by `Arc`, so
every clone and drop is an atomic operation — even for values that never leave one task.

- Use `Rc`-based storage inside a worker.
- Convert values into a sendable form only at task boundaries: spawn arguments, channel send,
  task results, shared globals.
- FPAS value semantics with copy-on-write make this invisible to programs.

Expected to help strings, arrays, records, and callbacks noticeably.

**Deferred to the end of the plan.** An analysis before implementation showed that this step needs
a redesign of task execution, not only a storage swap:

- Task state, including registers, moves between pool threads through the shared scheduler while
  the root worker runs concurrently. `Rc` values cannot cross threads without `unsafe`, which the
  workspace forbids. Converting at every suspension would deep-copy all live task data every
  timeslice.
- Tasks would have to be pinned to one pool thread with a per-thread scheduler, and only sendable
  data (spawn arguments, results, channel messages) could cross threads.
- Globals are shared across threads. In multi-task programs every `LoadGlobal` of an aggregate
  would deep-copy it, which is a regression for global arrays and records.
- Channels, network/HTTP threads, and the debugger actor also hold values across threads, and
  `fpas-std` uses `Value` throughout.

The expected gain is limited to replacing uncontended atomic reference counts. Revisit this step,
together with the lock-free rest of step 10, after steps 11–13 and 16.

## 10. Lock-free capture cells — partly done, rest deferred with step 9

Done: `CellRead` and `CellWrite` borrow the cell from its register instead of cloning its `Arc`,
so an access is one uncontended lock without a reference-count round trip. The new
`mutable_capture` bench measures this path.

Remaining: removing the lock itself. `Value` must stay `Send` because values cross task and
pool-thread boundaries, so `Rc<RefCell<Value>>` is not possible yet, and a worker-owned cell table
would need its own reference counting shared across callback and debugger workers. Both become
straightforward once values are worker-local (step 9), so this rest moves behind step 9.

## 11. Compiler optimization passes on `fpas-ir` — done except inlining

Bytecode emission now selects every instruction first and derives block addresses from the
actual word counts, so passes may drop code without a separate width table
(`fpas-compiler/src/bytecode/function.rs`). Implemented:

- **Constant folding** (`optimize/constant_folding.rs`): operations on constant operands become
  constants with the VM's exact semantics (wrapping integer `+ - *`); operations that would raise
  a runtime error stay in place.
- **Copy propagation** (`bytecode/allocation/aliases.rs`): a local read reuses the local's
  register when the local is not written before the read's last use, so `B := A + B` is one
  `AddInteger` without `Move`s. The read's sequence point moves to the word that consumes it.
- **Dead constants** (`bytecode/allocation/dead_values.rs`): constants that no emitted word reads
  (folded operands, immediate operands) get no register and no code.
- **Loop-invariant constants** (`optimize/loop_constants.rs`): constants inside a loop move to its
  single outside predecessor. Register allocation keeps values that enter a loop alive until the
  latest back edge (`bytecode/allocation/loop_liveness.rs`). Every loop keeps at least one sequence
  point, so the debugger can still pause a running loop.
- **Jumps to the next block** are not emitted.
- **Register allocation** needs fewer live registers through the passes above; call arguments use
  the top of the call window (step 13).

Not implemented: **inlining**. The debugger has no representation for inlined frames, so
breakpoints and step-into inside an inlined routine would stop working, and running and debugging
share one compiled artifact. Inlining needs inlined-frame debug information first (see step 16).

## 12. Tail calls — done

`TailCall` replaces a direct call whose result the block returns, when caller and callee use the
same return convention. The following `Return` word stays and the verifier requires it. Outside
the debugger the callee reuses the current frame, so tail recursion no longer grows the call
stack. Debugger-owned workers execute `TailCall` as `CallDirect` followed by that `Return`, so
debug stacks, stepping, and frame operations keep every frame. `TailCall` is relocated like
`CallDirect` in unit objects and the linker. Calls through function values use `TailCallValue`
(step 17).

## 13. Overlapping register windows for calls — done

Multi-argument calls place their arguments at the top of the caller's frame. When the arguments
end the caller's active registers, the callee frame starts on them, so they become the callee's
parameters without copying. `CallFrame::frame_end` records the caller's register count; `Return`
and debugger forced returns restore it. Calls with one argument taken from an ordinary register,
bound receivers, and hosted callbacks keep the copying path.

## 14. Consistent in-place updates for unique values — done

- Record updates, string append, array push, and array/dictionary `IndexSet` update uniquely owned
  values in place (steps 6 and earlier).
- The compiler sets `Move` auxiliary 1 when the source is a temporary read exactly once and for
  the last time by the current IR instruction (register allocator last-use data; parameters and
  locals never qualify). The VM then moves the value and leaves the source uninitialized, so
  copies through temporaries no longer keep a second reference. This makes `X := X with ...`,
  assignments, and aggregate/call argument windows keep copy-on-write values unique. The verifier
  accepts auxiliary 0 or 1 on `Move`.
- Arguments copied from the call window into the callee frame are still cloned; step 13 removes
  that copy.

## 15. Strings and intrinsics

- Small-string optimization for short strings (no heap allocation).
- Builder-style growth for repeated `S := S + X` on a uniquely owned string.
- Resolve intrinsic IDs to a function-pointer table at load time instead of nested `match`
  cascades.

## 16. Rewrite: statically typed, untagged registers — deferred

FPAS is statically typed; `fpas-sema` knows the type of every expression. The VM does not need a
tag check per operation.

- Registers become raw 8-byte slots. `integer`, `real`, `boolean`, enum ordinals, and task handles
  are stored unboxed without a tag.
- Reference values (strings, arrays, records, dictionaries, functions, payload boxes) live in
  separate reference slots, or in slots whose kind is known per function from verified metadata.
- Every opcode is typed. All `*Dynamic` paths leave the hot path; generic code is specialized or
  lowered to typed operations by the compiler.
- Writing a scalar slot needs no drop handling; only reference slots are reference counted.
- The verifier checks slot kinds once per function, so the dispatcher never re-checks them.

Likely the largest single win for an interpreter: roughly 2–3× on numeric code on top of Part 1.

Constraints:

- The debugger (live image, frame restart, mutation, forced return, inspection) must read and
  write typed frames. Per-function slot-kind metadata replaces runtime tags for inspection. This
  is the main cost driver of the rewrite.
- Keep `unsafe_code = "forbid"`: slots are safe Rust types (e.g. `u64` bit patterns plus a
  separate `Vec` of reference values), not unions.

**Deferred to the end of the plan.** A scope analysis before implementation showed that this is a
re-architecture of several crates, not one rewrite:

- The register allocator reuses registers for values of different types, so typed slots need
  separate scalar and reference register files in the compiler, plus slot-kind rules in the
  verifier.
- About 150 register access sites in the VM core, every opcode handler, and the `*Dynamic` paths
  change.
- The debugger (about 17,800 lines; inspection, mutation, forced return, frame restart, task
  state, live image) must read and write typed frames through slot-kind metadata.
- `fpas-std` works on `Value` throughout, so every intrinsic call needs a slot/`Value`
  conversion.

After steps 2–13, an integer loop iteration is three or four dispatches; what remains per
operation is one tag check and one drop check. The expected gain on numeric code is therefore much
smaller than the original 2–3× estimate (not measured). Revisit this step together with step 9.

## 17. Follow-ups to steps 11–13

- **Tail calls through function values — done.** `TailCallValue` replaces a block-ending
  `CallValue` whose result the block returns (a unit call returns no value). The target is known
  only at run time, so the VM reuses the frame only when the callee's return convention matches
  the current function's; otherwise, and in debugger-owned execution, it runs as `CallValue`
  followed by the required `Return` word.
- **`task_spawn_wait` regression — not reproducible.** Alternating runs of the builds before and
  after steps 11–13 gave 462–510 ms and 458–513 ms. The benchmark's trivial task body makes it a
  scheduling measurement; with restricted CPU affinity it varies by up to 4× between runs.
- **Inlining prerequisites — deferred to the end of the plan.** Debug information for inlined
  frames, so step 11 can inline small routines without breaking breakpoints, stepping, and
  stacks. This is a debugger feature of its own; revisit it together with step 16.

## Order and verification

1. Build profile (step 1) as a quick win.
2. Dispatch loop and duplicate checks together (steps 2 and 3), then their follow-ups (step 3.1).
3. Superinstructions and typed fast paths (steps 4 and 5).
4. Targeted hotspots and allocator (steps 6 and 7).
5. Cheap Part 2 items: closure names, cell access, in-place updates (steps 8, 10, 14) — done;
   the lock-free part of step 10 follows step 9.
6. Compiler optimization passes, tail calls, and register windows (steps 11–13) — done except
   inlining.
7. Follow-ups to steps 11–13 (step 17) — done except the inlining prerequisites.
8. Revisit the typed register rewrite (step 16) with the debugger frame model, debug information
   for inlined frames and inlining (steps 17 and 11), non-atomic reference counting (step 9), and
   the lock-free rest of step 10; see the deferral notes in steps 9, 16, and 17.

Measure before and after each step with `cargo bench-fpas` and record results in
[`docs/bench/history.md`](../bench/history.md) (see the `fpas-bench` skill).

Expected results (estimates):

- `integer_loop` after Part 1: roughly 2–4× faster.
- Typical programs after Parts 1 and 2: roughly 3–6× faster than today.
