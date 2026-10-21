# AP48.3: Static capability diagnostics

Package: [AP48](README.md). Status: complete. Effort: medium.
Prerequisite: [AP48.2](02-transfer-rules.md) complete.

## Delivered contract

The checker represents proven transferable, known task-bound and unknown
callable-containing values with one `TransferCapability` type. Joins retain
task-bound evidence before unknown state, and unknown before transferable state.
Scalar-only projections are transferable. Unknown values retain the mandatory
runtime guard; no public language effects system is introduced.

Field/index writes, native value and InPlace operations, aggregate updates, patterns,
nested captures and method receivers preserve the relevant evidence. Control-flow
joins distinguish reachable normal and abrupt exits. Routine analysis isolates its
caller; mutable outer values and possible synchronous writes cannot retain stale
transferability proofs. Full-binding overwrites discard obsolete initializer
provenance. Incompletely resolved mutable callable values remain unknown.

Bounded summaries use resolved local routine, method and immutable-alias metadata,
including explicit returns and implicit Result-error returns through `try`.
Unknown parameters, unresolved calls and imported factory results remain permitted.
Deferred checks retain recursive named-capture diagnostics. FP3016 reports known
invalid task starts, arguments, sends and task results at their transfer boundary.
Ordinary local factories and same-task selection callbacks remain valid.

## Owners and migration

Modify `crates/fpas-sema/src/check/expr/task_bound.rs`, task-call checking, assignment
tracking, declaration metadata, closure pending-capability checks and the task/channel
standard-library checks. Keep representation and joins centralized. Review semantic
metadata and cached-artifact consumers if the representation changes.

## Implementation scope

- Add `crates/fpas-sema/src/capability.rs` and export `TransferCapability` from
  `lib.rs`. Migrate `scope.rs`, `scope/captures.rs`, `check/context.rs`, declaration
  and closure metadata to one transferable/unknown/task-bound representation.
  Update Symbol construction in `check/decl/`, `check/contracts/`, `check/calls/`,
  `std_registry/` and `interface/{install,conversion/from_interface}.rs`.
- Update `check/expr/{mod,task_bound,closure}.rs`, `check/decl/{vars,routine_body}.rs`
  and `check/closures/` for complete value propagation, unknown parameters/results,
  receiver capabilities and deferred recursive capture completion. Keep source
  function-body analysis from mutating its caller's flow facts. At isolated body
  entry, treat outer mutable callable-containing values as unknown so routine
  result summaries do not freeze their current contents. Add
  `check/closures/captured_writes.rs` for conservative synchronous-call invalidation
  of live mutable callable capture roots, using existing capture metadata and exact
  declaration identities. Ordinary source and imported calls can invoke stored
  callbacks even when the routine value is transferable; invalidate captured roots
  and mutable callable globals at these boundaries. Apply the same fallback to
  callback arguments. Retain known task-bound evidence while forgetting unsupported
  transferable proofs. Mutable initializers and assignments of incomplete named captures remain
  unknown rather than reusing provisional transferable facts; whole-binding writes
  discard stale initializer provenance instead of accumulating targets across writes.
- Add focused snapshot/join support in `scope/capabilities.rs`. Modify
  `check/stmt/assignment.rs`, `control_flow/loops.rs`, `control_flow/pattern_capabilities.rs`
  and `control_flow/if_case/` for partial-write joins, branch independence, missing
  alternatives and possible zero loop iterations. Keep scalar projections free.
  Add `control_flow/reachability.rs` and route statement lists in `check/stmt/mod.rs`
  and `check/task_scopes/ownership.rs` through its sequence checker. Distinguish
  normal fallthrough from break, continue and return so only real loop backedges
  propagate capabilities. Continue type-checking unreachable statements while
  discarding their capability writes and abrupt exits. Reuse case coverage to omit
  impossible unmatched exits from exhaustive finite cases.
- Extend `check/expr/calls/{native,fluent}.rs` with a focused
  `native_capabilities.rs` helper for stored native-operation arguments, preserved
  receiver values and InPlace target joins. Do not treat invoked callbacks as stored
  result values. Update `check/calls/mod.rs` to invalidate callable-containing
  `var` targets conservatively after argument checking, retaining known local facts.
  Cover these paths in `tests/expr/native/capabilities.rs`, wired from `native.rs`.
- Extend `check/expr/tasks.rs`, `std_registry/builtins/channel_task.rs` and its
  group/selection checks for all arguments and receivers. Add bounded routine-return
  capability summaries using `check/stmt/control_flow/jump_statements.rs`,
  routine-body checking and focused helpers, to diagnose known invalid task results
  while ordinary closure factories and unknown calls remain valid. Include implicit
  Result-error returns from `check/expr/mod.rs::check_try_expr`; scalar error payloads
  and Option-None returns remain transferable. Discard unreachable return contributions
  while preserving their ordinary type checks.
- Extend existing sema concurrency, pattern, closure, bound-method and record tests
  with focused capability fixtures. Add compiler end-to-end tests under
  `crates/fpas-compiler/src/tests/concurrency/static_capabilities.rs`, wired from
  `concurrency.rs`, for static rejections and dynamic helper boundaries. Add
  `crates/fpas-build/tests/incremental/capabilities.rs`, wired from `incremental.rs`,
  for cold/warm imported factory results; migrate public semantic metadata consumers as needed.
  Preserve existing serialized function flags and artifact representation.
- Repair `crates/fpas-debug/tests/jsonl.rs` to collect execution-limit records from
  both the launch response and the subsequent wait. The engine may report a fast
  failure in either response; no debugger production behavior changes.
- Update `docs/pascal/language/functions/closures.md`, relevant task/channel guidance,
  `docs/pascal/std/concurrency/task.md`, `docs/pascal/tools/diagnostics.md` and
  `docs/pascal/agent-reference.md` for the static evidence/runtime guard boundary.
  No syntax, public effects system or collection activation is introduced.

## Acceptance and verification

- A known local closure remains local through wrappers, patterns, field/index writes
  and branch joins. Overwriting one path does not erase another path's local state.
- Helper-returned closures and callable parameters with unknown capability cannot
  bypass the VM guard, while valid immutable-returning helpers remain usable.
- Recursive captures and bound receivers receive consistent diagnostics.
- Plain scalar values extracted from local aggregates remain transferable.
- Compiler-detected violations and dynamic runtime violations implement the same
  ownership rule; uncertainty alone does not reject otherwise valid programs.

Sema regressions cover flow order, shadowing, partial writes, abrupt exits, scalar
projections, recursive captures, native callbacks, mutable reads/writes and bounded
return summaries. Compiler tests distinguish static diagnostics from mandatory
runtime rejection. Cold/warm unit builds cover both transferable and forbidden
imported factory results. Serialized function flags and artifact representation
are unchanged. The diagnostic, closure, task and compact-reference guidance
describe the delivered contract. No syntax or collection activation is introduced.
