# Interpreter and runtime implementation guide

## Dispatch loop

The hot loop performs exactly one exhaustive opcode dispatch:

```text
fetch packed word
decode opcode
advance instruction address
match opcode
execute direct typed handler
apply terminal/suspend/timeslice transition
```

Category functions take decoded typed operands, for example
`execute_add_int(dst, left, right)`. They must not accept `Instruction` and perform another opcode
match. An exhaustive outer match makes an "unhandled opcode" fallback unnecessary.

Keep bounds safety in ordinary indexing/access APIs. The verified executable allows concise code, but
does not authorize `get_unchecked`. Consider unsafe dispatch/indexing only in a later, separate,
profile-backed proposal with explicit safety invariants and user review.

Do not add `#[inline(always)]`, branch prediction tricks, computed goto emulation, or macro-generated
duplicate interpreters until benchmarks isolate a gain.

## Worker state

Replace stack-machine fields with:

- shared `Arc<VerifiedExecutable>`;
- current `FunctionId` and function-relative instruction address;
- `Vec<Value>` containing frame register windows;
- current frame base;
- `Vec<CallFrame>`;
- pending root/test entry as `FunctionId`;
- current task ID, retention state, timeslice counter, suspension flags, and shutdown policy;
- last diagnostic address only when an error is being built.

Preallocate the main register vector from the root function's register count plus a modest measured
reserve. Pool workers may retain vector capacity between tasks, but loading a task must replace logical
contents exactly and must not leak values from a previous task.

## Register access

Centralize relative-to-absolute conversion in small typed methods:

- `read(Register) -> &Value`;
- `write(Register, Value)`;
- `take(Register) -> Value` only for bytecodes whose ownership contract permits it;
- typed scalar reads returning copied `i64`, `f64`, or `bool`;
- checked argument-window slice access.

Methods return structured internal VM errors for impossible malformed state. In production, the
verifier prevents these errors; tests must still exercise them through intentionally constructed
worker state where possible.

For arithmetic, copy scalar operands into Rust locals before writing the destination so operand/dest
aliasing is safe. Avoid cloning `Value` for typed scalar operations.

## Frame entry and return

On direct call:

1. Look up `FunctionInfo` by numeric ID.
2. Confirm runtime arity defensively even though verification checked it.
3. Check call-depth and total-register limits with checked arithmetic.
4. Reserve/resize the register store for the callee.
5. Move or clone arguments according to bytecode semantics into callee parameter registers.
6. Append captures for closure calls.
7. Initialize remaining registers to Unit.
8. Push a frame containing the caller continuation and return destination.
9. Enter instruction zero of the callee's code range.

On return:

1. Obtain the result before truncating the callee window.
2. Restore caller function, address, and base from the frame.
3. Truncate the register store to the caller's window end.
4. Write the result directly into the recorded destination.
5. For a root/task completion with no frame, route the value through the existing execution-context
   policy.

Preserve the current recursion diagnostic contract. Keep call depth and total live register storage as
separate limits so a large function and deep recursion report the correct resource.

## Values and ownership

Retain the current compact tagged `Value` design initially:

- integer, real, boolean, Unit, task ID, and simple sentinels are inline;
- strings and immutable/function metadata use shared storage;
- arrays, dictionaries, records, and enums retain copy-on-write/shared storage;
- Result/Option payloads and mutable capture cells preserve current semantics.

Keep the `Value <= 16 bytes` assertion. Do not force `Copy`; aggregate cloning intentionally increments
shared ownership. Prefer borrowing register values and producing a new destination only when the
operation semantically returns one.

After cutover, profile reference-count operations. If they dominate, evaluate narrow changes such as
last-use moves or a different shared pointer only with complete task-safety and semantic tests.

## Globals

Start with `RwLock<Vec<Option<Value>>>` because tasks share globals and current behavior permits
concurrent access. Resolve `GlobalId` before execution. Reads clone the stored logical value; writes
replace the slot while preserving current synchronization and copy-on-write behavior.

Potential lock striping or immutable-global separation is later work. Numeric IDs alone should be
measured first.

## Records and enums

Record values contain shared immutable layout metadata and positional field values. A field opcode
uses a validated slot. The runtime may retain a defensive layout/type check on error-prone operations,
but the successful path must not search or case-fold strings.

Record mutation through FPAS value semantics performs copy-on-write before changing a positional slot.
Record `with` updates must evaluate overrides in existing order and detach no more than once per
operation.

Enum values carry shared layout metadata, numeric variant ID, and associated values. Variant tests are
integer comparisons. Formatting obtains type, variant, and field names from layout metadata.

## Functions and closures

`FunctionValue` contains numeric `FunctionId`, captures, task-bound flag, and shared diagnostic name
metadata. Invocation does not consult the name. Equality and display behavior remain consistent with
the current language contract; add tests before changing the representation.

Mutable captures remain `Arc<Mutex<Value>>` for the first cutover. The register rewrite must not
weaken `Send`/`Sync` or task-bound enforcement. Optimize cells only after concurrency profiles identify
them.

## Intrinsics

All intrinsics use a uniform register ABI:

```text
Intrinsic(dst, intrinsic_id, arg_base, arg_count)
```

The dispatch validates the ID once when loading the executable, then selects the concrete intrinsic.
Argument helpers borrow a register slice and provide typed extraction with the current diagnostic codes.

Rules:

- borrow strings/arrays/records for read-only operations;
- clone only at an ownership or semantic boundary;
- collect into a temporary vector only when the host API genuinely requires ownership;
- write exactly one result to `dst`, including Unit;
- hosted callbacks invoke `FunctionId` through the common frame machinery;
- blocking host calls retain current shutdown/cancellation behavior;
- OS-specific path/process behavior stays in `fpas-std` or hosted runtime modules.

## Tasks and scheduling

The scheduler remains cooperative at bytecode timeslice boundaries and uses the existing OS-thread pool
for spawned work. Register bytecode must not change FPAS task semantics.

`TaskState` saves all execution-local data necessary to resume deterministically:

- function and instruction address;
- register store and frame bases;
- call frames and return destinations;
- task ID and retained-result flag.

Do not save source locations; resolve from address after resume when needed. Do not copy the executable
per task.

Timeslice accounting remains one decrement per logical bytecode instruction. If a superinstruction
replaces several operations, document and test whether it counts as one scheduling step; preserve
fairness rather than maximizing a benchmark.

## Diagnostics

Runtime helpers receive an `InstructionAddress` or lightweight execution point, not an eagerly loaded
`SourceLocation`. On error:

1. resolve the source run for the failing address;
2. construct the existing diagnostic code/message/help contract;
3. retain fallback behavior when metadata is missing in an internally constructed test executable;
4. render paths through the existing terminal-safe diagnostic layer.

Malformed bytecode is rejected before VM construction. Defensive internal errors inside the VM should
therefore indicate a compiler/verifier/runtime invariant bug, not blame the FPAS user.

## Profiling checkpoints

Profile after each production-capable milestone, focusing on:

- dispatch and operand decoding;
- `Value::clone` and `Arc` reference counting;
- frame resize/initialization and call argument moves;
- global lock acquisition;
- intrinsic argument conversion;
- aggregate copy-on-write detachment;
- scheduler/timeslice overhead;
- source lookup only on diagnostic workloads.

Do not optimize a former hotspot after the rewrite without confirming it remains hot.
