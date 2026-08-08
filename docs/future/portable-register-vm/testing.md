# Test and platform acceptance plan

## Test strategy

The rewrite changes compiler, bytecode, linker, artifacts, VM, hosted runtime plumbing, and tasks while
preserving FPAS behavior. Testing therefore has four layers:

1. focused representation and verifier unit tests;
2. crate integration tests at compiler/linker/program/VM boundaries;
3. existing FPAS regression and standard-library suites;
4. native platform and cross-host artifact execution.

Every new behavior test has a positive case, a negative case where failure is meaningful, and relevant
boundary/edge cases. Tests should describe one behavior and keep action/assertion visible.

## `fpas-ir`

Positive coverage:

- valid scalar, branch, loop, call, closure, aggregate, intrinsic, and task functions;
- block parameters and deterministic block order;
- typed local reads/writes and valid returns.

Negative coverage:

- unknown IDs and use-before-definition;
- duplicate blocks/values/locals/functions;
- missing/multiple terminators;
- branch argument count/type mismatch;
- direct-call signature mismatch;
- return type mismatch;
- invalid capture order/type;
- invalid layout reference.

Edges:

- empty Unit-returning root;
- one-block and maximum-block functions;
- unreachable but unreferenced blocks according to chosen policy;
- ID/count conversion exactly at and one beyond representable limits.

## `fpas-bytecode`

Positive coverage:

- checked constructor/accessor round-trip for every opcode and form;
- destination alias cases explicitly allowed by each opcode;
- valid register windows, jumps, functions, layouts, source runs, and task flags;
- exact eight-byte instruction and compact Value assertions.

Negative coverage:

- every unknown opcode byte;
- wrong form/reserved bits;
- sentinel/out-of-range register;
- bad constant/string/global/function/type/field/variant/intrinsic ID;
- overflowing or out-of-frame argument windows;
- wrong direct-call arity;
- branch outside function or to code boundary;
- overlapping/empty function ranges and fallthrough;
- invalid entry function and feature flags;
- unsorted/out-of-range source runs.

Maintain an exhaustive opcode inventory test so a new opcode cannot omit verifier and execution
coverage silently.

## Compiler and register allocation

Positive coverage:

- every expression and statement family currently covered by compiler tests;
- left-to-right side-effect evaluation;
- nested scopes and shadowing;
- mutable locals through branches/loops;
- block merge values;
- direct calls, recursion, nested routines, closures, properties, events, and intrinsics;
- records with defaults and `with` updates; enums with associated data; generic erased operations;
- deterministic IR and bytecode for identical input.

Negative coverage:

- preserve current diagnostic code/message/help for existing invalid FPAS;
- register, argument, function, global, layout, and instruction limit overflow;
- compiler-internal inconsistent semantic metadata returns a diagnostic rather than panicking.

Edges:

- destination aliases source;
- zero and maximum argument/capture counts;
- last register and first rejected register;
- break/continue through nested scopes;
- early return from nested control flow;
- case/short-circuit paths with effects in only one branch.

For a small curated source corpus, assert the resulting IR/bytecode structure or instruction counts.
Avoid enormous snapshots.

## Linker and unit objects

Positive coverage:

- one program; one/multiple/transitive libraries; workspace references;
- public exports and internal units;
- globals, direct functions, methods, record/enum layouts across objects;
- deterministic object and final image bytes;
- source-map rebasing and dependency initialization order.

Negative coverage:

- duplicate definitions case-insensitively;
- missing/private/wrong-kind imports;
- callable without implementation or incompatible signature;
- incompatible duplicate type layout;
- relocation kind/opcode mismatch;
- every table/address overflow;
- stale/corrupt/old `.fpascu` rebuild path.

## Artifact codec

Implement every positive/negative/edge test listed in
[Bytecode and portability](bytecode-and-portability.md). Retain current bounded-decoder tests and expand
them for sections and packed instructions.

Add a canonical digest test constructed entirely from fixed relative metadata. The test must produce
the same digest on Windows, Linux, macOS, FreeBSD, x86-64, and ARM64.

## VM scalar and control flow

Preserve tests for:

- wrapping integer add/subtract/multiply;
- division/modulo by zero and minimum integer divided by `-1`;
- minimum integer negation;
- shifts below zero and above 63;
- real operations, infinities, NaNs, signed zero;
- boolean coercion where currently supported;
- typed and dynamic comparisons;
- string operations and Unicode behavior;
- truthiness and short-circuit behavior;
- all branch/loop forms and malformed control flow.

Add register alias permutations (`dst == left`, `dst == right`, all distinct) for typed operations.

## Calls, closures, and frames

P4 evidence lives in `fpas-compiler/src/tests/register_subset/{functions,closures}.rs`,
`fpas-vm/src/vm/register/tests/{calls,callbacks}.rs`, and the register-bytecode verifier suite.
Compiler cases are differential against the production stack VM; direct VM cases isolate frame,
capture, callback, and limit behavior without compiler coupling.

Preserve and extend:

- zero/many arguments, functions/procedures/methods, recursion, early return;
- first-class and bound function calls;
- immutable/mutable captures and enclosing-depth behavior;
- callback calls from arrays, Result/Option, Graph/TUI, and hosted operations;
- wrong arity, invalid IDs/windows, call/register stack overflow, and callback unwind;
- multiple VM instances sharing immutable executable bytes without sharing runtime state.

## Aggregates

Preserve and extend:

- arrays/dictionaries/records/enums and nested combinations;
- copy-on-write clones remain unchanged after mutation;
- record layout/field order, defaults, properties, events, equality, formatting, and updates;
- known-field operations use numeric slots;
- missing field, wrong type/layout, invalid variant, and out-of-range associated field diagnostics;
- dictionary insertion order and key equality;
- Result/Option wrapping/unwrapping and errors.

## Intrinsics and hosted runtime

Every intrinsic ID appears in an exhaustive test. For each intrinsic family test:

- valid argument types/count and result register;
- wrong count/type diagnostics;
- borrowed inputs are not accidentally mutated;
- owned results do not alias incorrectly;
- hosted callback behavior and cancellation;
- headless Console/Graph/TUI deterministic state;
- filesystem scratch stays under `.temp-data/`;
- OS-specific tests use platform conditions only where behavior is genuinely host-specific.

P6 evidence is recorded in `p6-intrinsics-hosted-runtimes.md`. The bytecode inventory and compiler
catalog each cover every stable ID independently. Borrowed-runtime tests cover unchanged input and
wrong count/type diagnostics; compiler differential tests cover ordinary and higher-order calls;
direct register-VM tests cover Args, shared Console/Test input and output, headless Graph lifecycle,
and the explicit P7 task boundary. Source-defined TUI code remains a P8 unit-object/linker consumer,
while its Console host surface and the existing production headless suite remain covered in P6.

## Concurrency

Preserve all current task/pool/runtime tests and add:

- suspension with live registers in nested frames;
- resume after call result targets were saved;
- multiple tasks sharing one immutable executable;
- retained/detached success and failure;
- task-bound mutable closure rejection;
- wait-one/wait-all, cooperative sleep, timer wakeup, timeslice yield, shutdown, and panic;
- high-contention repeated runs that assert completion and result correctness;
- task save/load does not retain stale values from a previous task.

P7 evidence is recorded in `p7-tasks-concurrency.md`. Register compiler differential tests cover
retained/detached execution, Wait/WaitAll retention, cooperative sleep, mutable-capture rejection,
and a timeslice taken inside a nested call while the caller retains an aggregate register. Direct
register tests cover main Yield, invalid Wait operands, and cooperative shutdown. The unchanged
218-test `fpas-vm` suite supplies repeated production concurrency, pool, timer, shutdown, panic, and
stress regression coverage; the phase-local `register-p7` benchmark exercises the complete
register spawn/wait path without claiming a production comparison.

Do not weaken scheduler fairness or timeout bounds to improve a benchmark.

## Differential development testing

Before production cutover, use a test-only harness to run a curated deterministic program through old
and new paths and compare:

- exit classification;
- stdout and captured console lines;
- structured screen/pixel snapshots where applicable;
- runtime diagnostic code, location, message, and help;
- deterministic files under `.temp-data/`.

Exclude nondeterministic time/random/task-order observations unless the test supplies deterministic
hosts. Do not expose a user-facing `--backend` flag. Delete the old path and differential harness after
cutover; lasting regression fixtures become explicit expected-output tests.

## Required verification commands

During phases use focused commands. Before each milestone and final completion run:

```text
cargo fmt
cargo build
cargo test --workspace
cargo clippy --all-targets --all-features --locked -- -D warnings
fpas fmt --check examples/ tests/ apps/
fpas test tests/
git diff --check
```

Build release and run the benchmark protocol separately. Interactive demos are checked or exercised
with their existing headless tests; do not launch a blocking UI in batch verification.

## Native platform matrix

Portability is an evidence claim. Use native hosts where available:

| OS | Architecture | Minimum evidence |
|---|---|---|
| Windows | x86-64 | workspace build/test, FPAS suite, canonical digest, direct `.fpascp` run |
| Windows | ARM64 | same when external crates and native runner build |
| Linux | x86-64 | same plus Chromebook run when available |
| Linux | ARM64 | same on native ARM device/runner |
| macOS | x86-64 | same while toolchain/crates support it |
| macOS | ARM64 | same on Apple Silicon |
| FreeBSD | x86-64 | same while toolchain/crates support it |
| FreeBSD | ARM64 | same when a supported native environment exists |

For unavailable hosts:

- `cargo check --target <triple>` may provide compile-only evidence if toolchain and native dependencies
  permit it;
- record `compile-only` or `unverified`, never `supported and tested`;
- do not add GitHub Actions to obtain the matrix;
- failures in external crates are named precisely and separated from FPAS-owned failures.

## Cross-artifact matrix

Use the producer/consumer procedure from [Bytecode and portability](bytecode-and-portability.md).
At least Windows x86-64 -> Linux x86-64 must pass before closing the original user scenario when both
hosts are available.

The host-native `--executable` output is tested only on its producing OS. Do not expect a Windows `.exe`
to execute on Linux.

## Documentation verification

At implementation completion:

- update `docs/pascal/program-structure/compiled-programs.md` with the implemented format properties;
- update CLI/projects docs only where observable behavior changed;
- confirm language and Std API docs remain semantically correct;
- update every Rust `///` link affected by moves;
- remove claims about JSON payloads and stack bytecode;
- remove this future-plan directory only after current docs and tests replace it.
