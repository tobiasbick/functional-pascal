# AP40.2: Ownership and shared state

Package: [AP40](README.md). Status: complete.
Prerequisites: [AP40.1's recorded contract](01-concurrency-contract.md),
implemented AP26 and AP30, and the preserved AP37/AP38 rules.

## Scope and affected areas

Implement owned starts, one go expression form, ordinary discard, foreign
mutable-state rejection, owner teardown, and ordinary returned Result values
together according to AP40.1.

- Require a same-routine lexical scope for go. Give named/anonymous helpers
  the same rule; explicitly passed groups remain the dynamic ownership form.
  Retain scope-handle escape validation and existing group admission/owner checks.
- Remove go statements from parser/AST/lowering and retain one expression spawn
  returning AP30's exact task type. Diagnose standalone starts with concrete
  binding/discard examples; diagnose missing ownership with scope/group repairs.
- Remove discard-specific task, aggregate, channel, generic, and callable-capture
  restrictions. Discard only evaluates and drops its value; owners retain child
  membership/results independently. Preserve no-value operand checks, ordinary
  expression typing, ownership escapes, initialization, and cross-task safety.
- Reject worker access to foreign mutable variables, including global reads and
  writes through helper calls, callbacks, captured callables, and imported units.
  Cover all worker-start paths. Reuse capture/dependency analysis and compiled-unit
  metadata where appropriate; unknown safety cannot silently authorize access.
  Worker-owned local mutation and explicitly shareable resources remain valid.
- Preserve AP37's var/InPlace storage restrictions and AP38 initialization checks.
  Pass values from the starting task and communicate through existing channels;
  no public effect system, general shared-cell API, or implicit atomic mutation.
- Retain scope normal-join/early-cancel-and-join behavior. On creating-task
  termination, cancel and fully close any open groups, including on return,
  failure, cancellation, and main-task teardown. Await children before publishing
  the owner's terminal outcome; keep original diagnostics and avoid duplicate
  reporting. Cooperative cleanup does not promise a hard wall-clock deadline.
- Stop classifying returned Error values as automatic group failures. Retain
  unexpected panic/runtime failures and separate cancellation behavior.
  Preserve explicit supervised retry criteria; the final Result remains data.
  Remove obsolete returned-Error reporting variants/helpers where no longer used.

Inspect parser and grammar consumers; `crates/fpas-sema/src/check/task_scopes/`,
`check/discard/`, closure/capture and initialization analysis; compiler concurrency
lowering; compiled-unit interfaces; and VM worker/global/task/group scheduling,
result retention, failure reporting, and debugger teardown. Keep modules small
and thematic, reuse shared checks, and remove obsolete spawn/discard-only code.

## Delivered implementation

- Go is one expression with exact `task`/`task of T` types and same-routine
  lexical ownership. Missing ownership and unused values give scope/group and
  binding/discard repairs. The statement AST, detached IR operation, opcode, and
  runtime start path are removed; malformed unowned bytecode is rejected too.
- Ordinary discard has no task-freedom proofs. No-value procedure operands,
  scope escapes, task-bound captures, and initialization remain independent checks.
- Private callable summaries track worker starts and mutable globals across
  helpers, patterns, loops, bound methods, callbacks, and reused unit interfaces.
  Known violations report FP3042. Unknown dynamic targets retain mandatory VM
  global/reference checks and cross-task capture checks; they confer no permission
  to access foreign mutable variables. Input snapshots and worker-local mutation
  remain valid.
- Creating-task completion cancels and joins open groups before publishing its
  outcome or beginning a supervised retry. Pool, main-task, and deterministic
  debugger paths share the cleanup continuation. Cancellation cannot hide an
  unexpected child error. Already observed failures are deduplicated by task
  identity; independent equal-source failures remain visible with original locations.
- Result Error values are ordinary data, including non-string payloads. Explicit
  supervision may retry them, but exhaustion creates no automatic failure report.
  TaskFailureKind contains only Panicked, RuntimeError, and Cancelled.
- Bytecode version 21 and unit format 17 invalidate obsolete instructions and
  dependency interfaces. Generated Std.Tasks declarations, scopes/groups in source
  consumers, grammar, editor snippets, and authoring guidance are migrated.

Language, interface, incremental-build, VM, debugger/protocol, and FPAS regressions
cover the delivered boundaries. A pre-fix owner-publication failure was reproduced
against AP39 and passes after AP40. Independent review cleanup/fixture findings
are fixed and rechecked; see [package verification](README.md#delivery-and-verification).

## Migration and documentation

Migrate lib/apps/examples/tests, Rust fixtures, formatter goldens, generated
declarations/templates, benchmarks, skills, and editor snippets/completion.
Replace standalone go with explicit handle binding or discard go expressions.
Supply a lexical scope or explicit group for every former unowned start; choose
the owner that preserves the intended lifetime. Helpers needing caller-owned
dynamic work receive a group rather than opening a shorter-lived scope.

Replace worker global/local shared mutable access with value arguments, worker
local copies, or channel protocols. Retrieve and explicitly handle group Result
values previously exposed only through returned-Error failure reports. Migrate
obsolete TaskFailureKind cases consistently with AP39's closed-enum rules.
Preserve evaluation order and intended completion/error handling within the
explicitly selected semantic changes.

Update concurrency/go/scope/capture/scheduling pages, Std.Tasks, discard rules,
grammar, compact reference, diagnostics, skills, and editor tools in the same
delivery. Name the replaced AP04/AP26 decisions while retaining their completed
status. Leave AP30 type and retrieval rules intact; AP40.3 owns wait priorities.

## Verification

Turn AP40.1's ownership/discard/state/teardown examples into language-boundary
regressions. Cover named/anonymous and nested helpers, function/procedure starts,
group/supervised starts, exact handle types, direct/indirect/imported mutable
access, worker-local mutation, permitted resource sharing, and unknown callables.

Verify ordinary discard for individual/aliased handles, aggregates, channels,
generic values, and captured callables without introducing discard-only safety
proofs. Preserve separate ownership/capture failures and no-value diagnostics.
At runtime, prove that discarded handles do not cancel/detach/consume children,
that aliases retain single retrieval, and that owners retain failures/results.

Use deterministic channels/signals to check normal and early scope exits,
creating-task/main-task group cleanup, cancellation, group Result values,
supervised retries, repeated close, and handle escapes. Test fresh and reused
unit interfaces and debugger parity. No lost-update race or long sleep is proof
of shared-state safety or cleanup ordering.

Run required Rust/FPAS suites, formatting, example/app checks, editor and grammar
checks, documentation links, and shared syntax regressions. Mark complete only
after implementation, consumer migration, documentation, and tooling agree.
