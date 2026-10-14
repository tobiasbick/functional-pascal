# AP45.1: Specify and verify snapshot iteration

Package: [AP45](README.md). Status: planned; not implemented.

## Scope and ownership

Confirm the agreed snapshot contract against
`crates/fpas-compiler/src/lowering/control_flow.rs` and collection value-copy
behavior. Reuse the current lowering. Only repair an implementation path if a
focused test shows that it violates the agreed contract; do not introduce a
separate iterator framework.

Update `docs/pascal/language/control-flow/for-in.md` with array and dictionary
examples, the single-evaluation rule, and the distinction between captured keys
and current dictionary lookups. Confirm `collection-mutation.md` remains
consistent and add concise guidance to `docs/pascal/agent-reference.md` under its
maintenance guide. The grammar needs no new syntax.

## Verification

- An iterable-producing function runs once per loop entry.
- Array append, removal, element replacement and whole-source reassignment in
  the body leave the original visited values and count unchanged.
- Dictionary insertion, deletion, reinsertion and reassignment preserve the
  original key sequence. Existing-key value updates remain visible to explicit
  body lookups; a removed-key lookup retains its normal failure behavior.
- Empty collections, nested loops, `break` and `continue` retain their ordinary
  behavior. Loop bindings remain read-only.
- Use existing compiler execution tests and FPAS regression suites; choose
  bounded fixtures so a live-iteration regression cannot hang the suite.
- Check the documentation examples, compact-reference execution test and the
  applicable [required checks](../development-process.md#required-checks).

No semantic migration of application sources is expected. Any conflicting
consumer assumption must be reported and resolved before completion.
