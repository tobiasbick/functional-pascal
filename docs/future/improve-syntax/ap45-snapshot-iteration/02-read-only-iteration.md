# AP45.2: Enforce read-only collection iteration

Package: [AP45](README.md). Status: complete; implemented, verified and independently reviewed.

## Contract

Replace AP45.1's permission to mutate/reassign an iterated source. The iterable
expression runs once per entry. While the body runs, its writable storage root
is read-only for arrays and dictionaries alike. Parenthesized sources and
field/element sources retain their root. Protect the entire containing root;
independent roots and value copies remain writable. Nested loop protections
accumulate and end with their respective bodies. Computed values such as
`Items.Filter(Predicate)` and `Values.Keys()` are independent iterable values.

Reject assignments to the root or its contents, native InPlace receivers and
explicit var arguments. Resolve transitive writes through named and anonymous
routines, callable values, callback parameters and compiled-unit summaries.
Read-only helpers remain legal. If callable effects cannot be proven safe for
a protected mutable source, reject the call with a compiler diagnostic and a
hint to use an explicit independent copy. Creating a closure is not invoking it.

The rule protects collection storage, not the resources referenced by channel
handles or closure environments stored in elements. AP37 callback operations
retain their separately documented behavior. No new grammar or runtime iterator
mode is introduced.

Var parameters can alias foreign/global/captured roots despite having distinct
declaration identities. Reject writes when disjointness from an active source
cannot be proven, in either alias direction. Independent local value copies
remain writable. Deferred captured callable dependencies must be rejected at
the protected loop call site rather than only propagated to an outer caller.

## Implementation and migration

- Extend the existing declaration-resolved dependency/write analysis; reuse its
  recursive fixed point, callable provenance and native callback roles.
- Persist transitive global writes in compiled-unit interfaces and ensure the
  current compiler fingerprint invalidates older cached artifacts.
- Add one iteration-specific diagnostic with a concrete correction hint.
- Replace source-mutation examples and regressions with protected-source
  rejection and explicit independent-copy examples. Adapt the dictionary
  missing-key regression to an explicit independent source.
- Update for-in, collection mutation, diagnostics and the compact reference;
  retain the existing executable compact-reference test.
- Migrate affected example/app/test sources without weakening their contracts.

## Acceptance gates

- Semantic regressions reject array and dictionary writes, reassignment, var
  forwarding, InPlace calls, transitive/recursive helpers, captured closures,
  callbacks and imported routine writes. Assert the owning diagnostic rather
  than accepting an unrelated parse/type error.
- Positive cases cover source reads, read-only helpers, lexical shadowing,
  independent copies, computed sources, nested loops and writes after the loop.
- End-to-end FPAS coverage retains once-only evaluation including empty sources,
  order, break/continue and shared resource behavior.
- Run cargo fmt, cargo build, cargo test --workspace, FPAS regressions, affected
  source formatting, documentation/example checks and cache/interface checks.
- An independent subagent reviews the final contract and implementation. Resolve
  findings before marking complete and creating the authorized co-authored commit.

## Implementation ownership

- `crates/fpas-sema/src/check/initialization/iteration.rs` owns active source roots,
  symmetric possible var aliases, direct/captured/imported writes and opaque
  callable rejection. Existing walker, statement, call and expression modules
  reuse declaration identities, recursive summaries and callable provenance.
- `crates/fpas-unit/src/interface/initialization.rs` persists `written_globals`;
  semantic interface export qualifies own globals and canonicalization sorts the
  effect set. The existing source-derived compiler fingerprint covers both
  semantic and interface changes, invalidating earlier cached artifacts.
- `crates/fpas-sema/src/tests/stmt/for_in_source_mutation.rs` owns negative and
  positive language-analysis coverage, including a serialized imported interface.
- `tests/stdlib/language/for_in_read_only_test.fpas` retains once-only evaluation,
  empty sources, order, nested control flow, shared resources and explicit copies.
- `crates/fpas-compiler/src/tests/control_flow/iteration.rs` retains ordinary
  dictionary lookup failures when the looked-up dictionary is a separate value.
- For-in, collection mutation, diagnostics and the compact reference document the
  current contract; no application/example sources or grammar required migration.

## Verification

`cargo fmt --all`, `cargo build` and `cargo test --workspace` pass. All 26 focused
for-in semantic tests pass. The full FPAS suite passes 518 tests with one intended
skip and no failures. FPAS source formatting passes for examples, tests and apps.
The 16 example manifest/workspace checks and application-directory check pass.
The existing compact-reference CLI test checks and executes every reference
example; 124 local file references in the changed documentation are valid.

Independent subagent review found var-parameter aliases and deferred captured
callables that could bypass the initial guard. Regression tests reproduced both
before correction; final review confirms all three mutation reproducers fail
with FP3046 and positive controls still check and execute. No blocking findings
or selected work remain open.
