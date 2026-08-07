# Implementation phases

## Execution rules

Implement phases in order. A phase may be split into smaller edits, but the next phase must not begin
until the current phase's exit gate passes. Keep the old production path working until the cutover;
new-path code before cutover is reachable only from focused Rust tests.

Do not combine a structural phase with an unrelated optimization. Do not record benchmark improvements
for code that the production CLI does not yet execute.

Every phase begins with:

1. Re-read the relevant files in the current checkout.
2. Check `git status --short` and preserve unrelated user changes.
3. Show the exact files to create, move, modify, split, and remove.
4. Run the narrow existing tests that cover the touched behavior.
5. Update the phase checkbox and evidence in this plan only after verification.

Every code phase ends with at least `cargo fmt`, a targeted build/test, and `git diff --check`. The full
workspace gate occurs at the marked milestones and at final completion.

## Progress ledger

Implementation agents update this table. `Evidence` must name tests or benchmark snapshots, not say
only "done".

| Phase | State | Evidence |
|---|---|---|
| P0 Contract and baseline | complete | `p0-contract-baseline.md`; full 16-row `register-vm-before` snapshot; two VM repeats; artifact tests (10 + 8 + 18 + 1 + 1); `cargo fmt`/build/test/clippy and FPAS fmt/test gates passed |
| P1 Typed IR | complete | `fpas-ir` validation integration tests: 17 passed, including loop backedges, source spans, maximum IDs, and checked conversions; no unchecked narrowing remains; `cargo fmt --all -- --check`; `cargo build -p fpas-ir`; `cargo clippy -p fpas-ir --all-targets --locked -- -D warnings`; `cargo test --workspace` passed; production path unchanged |
| P2 Register bytecode model and verifier | not started | — |
| P3 Scalar/control-flow compiler and interpreter | not started | — |
| P4 Calls, frames, closures, and callbacks | not started | — |
| P5 Globals, records, enums, arrays, and dictionaries | not started | — |
| P6 Intrinsics and hosted runtimes | not started | — |
| P7 Tasks and concurrency | not started | — |
| P8 Unit objects and linker | not started | — |
| P9 `.fpascp`, `.fpascu`, bundle, and CLI cutover | not started | — |
| P10 Delete stack VM and reconcile documentation | not started | — |
| P11 Final performance and platform acceptance | not started | — |

Valid states are `not started`, `in progress`, `blocked`, and `complete`. A phase is `blocked` only with
a concrete repeated blocker and the evidence collected so far.

## P0 — Contract and baseline

Purpose: freeze observable behavior and collect trustworthy measurements before architecture changes.

Tasks:

- Inventory every current `Op` variant and map it to language/runtime tests. Put the mapping in the
  implementation work notes, then fold the final mapping into [Traceability](traceability.md).
- Inventory every `Chunk` consumer with `rg "\bChunk\b" crates` and every string-resolved operation
  with `rg "functions\(\)|GetGlobal|SetGlobal|FieldGet|FieldSet|IsVariant|MakeEnum" crates`.
- Run the full current test suite and save exact command results.
- Build release `fpas-cli` and save `cargo bench-fpas save register-vm-before` for the full suite.
- Repeat `cargo bench-fpas run --group vm` at least twice to expose noise. Do not average runs from
  different machines or power modes.
- Add missing benchmarks described in [Benchmarks](benchmarks.md), then recreate the baseline so it
  contains the final suite shape.
- Capture deterministic current `.fpascp` behavior: format version rejection, truncation rejection,
  malformed operand rejection, source path behavior, direct `fpas run file.fpascp`, and bundle run.
- Confirm no language document is modified by the rewrite proposal.

Exit gate:

- Full workspace tests pass before the rewrite.
- A complete local baseline exists under `.temp-data/bench/` and is not committed.
- Each current opcode has an owner and successor operation.
- Any pre-existing failure is reported to the user before implementation continues.

## P1 — Typed IR

Purpose: introduce the target-independent contract without changing production execution.

Tasks:

- Add `fpas-ir` to the workspace with the layout in [Target architecture](target-architecture.md).
- Implement ID newtypes, program/function/block structures, typed operations, terminators, and
  deterministic iteration order.
- Encode value types using the existing semantic type vocabulary where practical. Do not clone the
  complete semantic model into a second type system. Define a compact lowered `IrType` only for
  distinctions needed by code generation and validation.
- Implement validation for duplicate IDs, missing blocks, missing definitions, operand type mismatch,
  invalid block arguments, invalid terminators, and unreachable referenced blocks.
- Add builders only where they enforce invariants. Do not add a generic builder framework.
- Add unit tests for positive, negative, and edge cases, including maximum IDs and integer conversion
  failures.

Production compiler and VM remain unchanged in this phase.

Exit gate:

- `cargo test -p fpas-ir` passes.
- `cargo clippy -p fpas-ir --all-targets --locked -- -D warnings` passes.
- Public modules/types/functions have `///` documentation.
- No runtime, CLI, or FPAS behavior changed.

## P2 — Register bytecode model and verifier

Purpose: create a complete, independently testable executable representation.

Tasks:

- Add packed `Instruction(u64)`, explicit `Opcode` values, operand newtypes, function metadata,
  string/constant/global/type tables, and sparse source maps.
- Keep the old `Chunk` and stack `Op` temporarily. Put the new implementation under final concern
  names, not `new.rs`; the old modules are the temporary code to delete.
- Provide checked constructors for every opcode form and checked accessors for every operand.
- Implement a test-only executable builder that emits valid register code without the compiler.
- Implement the complete verifier described in [Bytecode and portability](bytecode-and-portability.md).
- Add exhaustive opcode round-trip tests and malformed cases. A table listing all opcodes in tests
  must fail to compile or fail a count assertion when an opcode is added without coverage.
- Assert `size_of::<Instruction>() == 8` and `size_of::<Value>() <= 16` on all test targets.

Exit gate:

- New bytecode unit and integration tests pass.
- The verifier accepts every valid test fixture and rejects each invalid operand category.
- No `unsafe`, `transmute`, unchecked narrowing, or host-sized serialized field exists.
- Production execution remains on the old VM.

## P3 — Scalar/control-flow compiler and interpreter

Purpose: prove the end-to-end register model for the smallest semantically complete subset.

Initial subset:

- integer, real, boolean, string, and Unit constants;
- immutable and mutable locals;
- assignment and expression temporaries;
- typed unary/binary operations and dynamic generic numeric operations;
- `if`, `case` for scalar values, `while`, `repeat`, `for`, `break`, and `continue`;
- root entry and return without functions or hosted intrinsics.

Tasks:

- Implement AST/sema-to-IR lowering for the subset.
- Implement deterministic block order and register allocation.
- Implement one exhaustive register dispatch loop. Handlers receive decoded operands and never rematch
  a complete instruction.
- Store instruction address in failures; resolve sparse source locations only while constructing a
  diagnostic.
- Port existing scalar/control-flow tests to exercise the new path. During development, keep old and
  new expected outcomes in test code rather than a public CLI backend flag.
- Add instruction-count assertions for small fixed programs to catch accidental push/pop-style
  lowering.

Exit gate:

- The subset has matching output and diagnostic codes on both paths.
- All register operands are validated before execution.
- A local micro-measurement shows the new dispatch path is viable. This is diagnostic evidence, not
  yet a recorded production speedup.

## P4 — Calls, frames, closures, and callbacks

Purpose: replace name-based invocation and stack frames.

Tasks:

- Lower and execute direct functions, procedures, methods, recursion, early returns, and nested
  routines using `FunctionId`.
- Implement frame windows, contiguous argument windows, return destinations, and register limits.
- Implement first-class functions and closures with numeric function IDs, ordered captures, mutable
  cells, and task-bound state.
- Replace callback entry offsets/names in VM tests and hosted callback plumbing with `FunctionId`.
- Preserve callback cancellation, shutdown, and panic behavior.
- Add wrong-arity, invalid-function-ID, invalid-window, recursion-limit, missing-return, capture-order,
  task-bound, and callback-unwind tests.
- Add differential coverage for every existing function and closure regression family.

Exit gate:

- `function_call` and `array_callbacks` can execute entirely on the new path.
- No normal call performs string canonicalization or function-table hashing.
- The verifier rejects call targets outside the function table and jumps across function boundaries.

## P5 — Globals, records, enums, arrays, and dictionaries

Purpose: remove remaining name lookup from language operations while preserving value semantics.

Tasks:

- Assign globals deterministic `GlobalId` slots and use an `RwLock<Vec<Option<Value>>>` initially.
- Build record and enum layouts from semantic metadata. Store names once in shared layout metadata and
  use numeric field/variant IDs in instructions and values.
- Lower constructors, field reads/writes, record `with` updates, properties, events, enum construction,
  variant tests, destructuring, and associated fields.
- Port arrays and dictionaries to register operands without redesigning their language behavior.
- Preserve copy-on-write detachment, insertion/field order, equality, display formatting, and mutation
  visibility.
- Add tests for anonymous record shapes, defaulted fields, qualified names, case-insensitive source
  names, generic records/enums, nested aggregates, shared mutation, missing fields, and invalid layouts.
- Profile record operations. Do not replace existing `Arc` aggregate storage unless direct evidence
  shows it remains material after numeric fields.

Exit gate:

- `record_update`, array, and global-access benchmarks run on the new path.
- Field and variant operations contain no linear name search.
- Existing record/enum/dictionary output and errors remain equivalent.

## P6 — Intrinsics and hosted runtimes

Purpose: route every standard-library and host call through register operands.

Tasks:

- Preserve the existing intrinsic ID hierarchy but validate every ID during executable verification.
- Define one register call convention: destination, intrinsic ID, argument base, argument count.
- Port intrinsic argument decoding from stack pops to borrowed register slices. Clone only when the
  callee requires ownership or FPAS value semantics require an independent logical value.
- Port `Std.Console`, `Std.Graph`, `Std.Tui`, filesystem, process, environment, time, random, parsing,
  JSON, TOML, collection, result/option, testing, and argument handling.
- Keep hosted callbacks on the numeric `FunctionId` path from P4.
- Preserve platform-specific behavior in host crates; no OS logic enters bytecode.
- Add an explicit coverage test equivalent to the existing all-intrinsics list.

Exit gate:

- Every intrinsic is selected, validated, and tested.
- The complete non-concurrency FPAS suite runs on the new path.
- Headless Graph/TUI tests remain deterministic.

## P7 — Tasks and concurrency

Purpose: migrate saved task state and scheduler integration after single-worker semantics are stable.

Tasks:

- Save function ID, instruction address, register store, frames, retain-result state, and task ID in
  `TaskState`.
- Preserve lazy worker-pool creation, timeslicing, detached/retained task behavior, waits, cooperative
  sleep, shutdown, cancellation, and result retention.
- Ensure immutable executable metadata is shared with `Arc`; execution-private mutable state must not
  be placed in the shared executable.
- Keep mutable capture cells `Send + Sync` and enforce task-bound closure rules.
- Add suspend/resume tests in the middle of nested calls and with live aggregate registers.
- Run the existing concurrency, pool, and runtime stress suites repeatedly.

Exit gate:

- `task_spawn_wait` executes on the register VM with no semantic regression.
- Thread sanitizer support may be used when available, but lack of it does not replace deterministic
  stress tests.
- No lock is introduced into the per-instruction scalar fast path.

## P8 — Unit objects and linker

Purpose: make independently compiled units produce relocatable register objects and link into numeric
executable tables.

Tasks:

- Redesign `RelocatableObject` around per-function register code, symbolic definitions/imports,
  constants, source runs, and record/enum layouts.
- Increment the `.fpascu` format/version and reject old sidecars as rebuildable derived artifacts.
- Assign IDs deterministically using dependency order plus canonical symbol order as specified in
  [Compiler and linker](compiler-and-linker.md).
- Relocate functions, globals, constants, strings, types, fields, variants, code ranges, and source
  IDs. Local registers never require link relocation.
- Validate duplicate definitions, visibility, imports, arity, layout compatibility, and overflow
  before producing an executable.
- Port incremental-build identity tests and workspace/library integration tests.

Exit gate:

- Library dependencies and workspaces build and run with the new object/linker path.
- Repeated identical builds produce byte-identical objects and executables.
- Missing, old, corrupt, and incompatible `.fpascu` sidecars rebuild automatically.

## P9 — `.fpascp`, bundle, and CLI cutover

Purpose: make the register executable the sole production artifact and execution path.

Tasks:

- Implement the bounded sectioned binary codec from [Bytecode and portability](bytecode-and-portability.md).
- Increment `PROGRAM_FORMAT_VERSION` and `BYTECODE_VERSION` together for the cutover.
- Port `fpas build`, `run`, `check`, `test`, program artifact reuse, the test child process, bundle
  packaging, and `fpas-runner`.
- Keep `fpas-runner` thin: decoder, verifier, VM, and runtime only.
- Ensure direct `fpas run file.fpascp` never needs sources, manifests, sidecars, parser, sema, compiler,
  linker, or source standard library.
- Reject old stack images with one actionable rebuild diagnostic. Do not implement migration.
- Perform the cross-host artifact procedure in [Testing](testing.md).

Exit gate:

- The CLI has no backend switch and always executes register bytecode.
- Source-less `.fpascp` and native bundles run.
- Artifact corruption and version diagnostics are covered.
- `cargo test --workspace` and the full FPAS suite pass.

## P10 — Delete stack VM and reconcile documentation

Purpose: finish the rewrite rather than retaining two architectures.

Tasks:

- Delete stack `Op`, `Chunk`, stack compiler emission, stack worker fields/helpers, old relocations, old
  JSON payload conversion, and temporary differential adapters.
- Rename register-era types to ordinary names where temporary qualification remains.
- Remove dead dependencies such as `serde_json` from artifact crates if no longer used there.
- Re-run `rg` for old symbols and obsolete terms.
- Split any file that grew beyond the repository's responsibility/size guidance.
- Update current docs under `docs/pascal/` to describe only observable artifact and CLI changes. Do not
  expose internal IR details as language specification.
- Update Rust `///` links and module documentation.

Exit gate:

- No `legacy`, `old`, `stack_vm`, compatibility decoder, or dormant alternate path remains.
- Docs describe implemented behavior only.
- Full format, build, test, and lint gates pass.

## P11 — Final performance and platform acceptance

Purpose: prove the rewrite solved the original problem and retained portability.

Tasks:

- Rebuild release `fpas-cli`.
- Run at least three same-machine comparisons against `register-vm-before`; report median deltas for
  every row and investigate outliers.
- Run the full suite, not only VM benches.
- Use a profiler on the slowest remaining VM rows before adding any final optimization.
- Apply only benchmark-proven peepholes or data-layout changes, each as a separate measured step.
- Record the settled result with `cargo bench-fpas record "after portable register VM"`.
- Execute the platform and cross-artifact matrix. Mark unavailable native hosts unverified.
- When available, repeat the saved workload on the low-end Linux Chromebook and report CPU model only
  at runtime to the user; do not write identifying machine metadata into the repository.

Exit gate:

- Performance gates in [Benchmarks](benchmarks.md) pass or the user explicitly accepts measured lower
  gains.
- Platform gates in [Testing](testing.md) pass for every claimed platform.
- No language syntax or semantic change occurred.
- After current docs and tests are complete, delete this future-plan directory and remove its index
  entry, as required by `docs/future/README.md`.
