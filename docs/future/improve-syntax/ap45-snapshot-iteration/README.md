# AP45: Read-only collection iteration

Status: complete (AP45.1 original delivery; AP45.2 read-only revision).
Effort: medium. Completion is tracked in the [central plan](../README.md).

## Design boundary

The agreed revision replaces AP45.1's permission to change the source during
`for-in`. Arrays and dictionaries have the same read-only source rule. Direct
and indirect writes are compiler errors, including writes through helpers,
closures, callbacks and independently compiled units. Unknown callable effects
must be rejected when they could change a protected source.

## Implemented contract

- Evaluate the iterable expression once on entering each execution of the loop.
- Array iteration visits elements in order; dictionary iteration visits keys
  in insertion order. Ordinary source lookups remain readable and stable.
- Protect the source's writable storage root throughout the loop body, including
  nested loops. Reject reassignment, element/value writes, InPlace operations,
  explicit var arguments and transitive writes. Protect the containing root for
  field/element sources so ancestor replacement cannot bypass the rule.
- Writes to other roots, independent value copies and writes after the loop
  remain legal. Computed iterable values have no protected source root; their
  evaluation finishes before the body's protection starts.
- Preserve immutable loop bindings, empty-loop behavior, lexical scope,
  `break`/`continue`, and shared-resource semantics inside copied values.
  A snapshot of values is not a deep clone of channels or closure environments.
- No live-iteration mode, new keyword, collection API or concurrency guarantee.

This retains AP37's value semantics and its separate callback snapshot contract.
No borrow syntax, mutable iteration mode or iterator framework is added. Shared
resource handles and closure environments are not recursively frozen.

## Dependencies and work packages

AP37's collection semantics are complete. AP44 and AP46 are independent.

- [x] [AP45.1: Specify and verify snapshot iteration](01-iteration-contract.md)
- [x] [AP45.2: Enforce read-only collection iteration](02-read-only-iteration.md)

## Open decisions

No selected AP45 decisions or implementation work remain open.

## Acceptance

Compiler regressions reject direct and indirect mutation with FP3046 and a
concrete hint. Positive regressions retain single evaluation, visit order, reads,
independent copies, nested scopes, post-loop writes and shared resources. Current
guidance and executable examples match the revision. Required Rust/FPAS checks
pass and independent subagent review found no remaining blocking issues.
[AP45.2](02-read-only-iteration.md) records ownership and verification.
