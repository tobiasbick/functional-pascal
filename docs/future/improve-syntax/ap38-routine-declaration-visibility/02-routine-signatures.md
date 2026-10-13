# AP38.2: Routine signatures

Package: [AP38](README.md). Status: Complete.
Prerequisite: [AP38.1's recorded contract](01-visibility-decision.md).

## Delivered behavior

Named free routines, record methods, and nested helpers have scope-wide
signatures. Calls may precede declarations, and siblings may be mutually
recursive. Nearest lexical declarations win independently of source order;
same-scope duplicates remain errors. Callable bindings initialize as values.

Value initializers retain source order and their existing effects. Bodies may
resolve later enclosing bindings, but calls, capture creation, and construction
must have initialized dependencies. Omitted record defaults retain their
declaring environment and are checked when evaluated; provided fields skip them.
Unsafe or unprovable startup uses produce FP3041 with a repair hint.

## Implementation and migration

- Sema separates signature collection, typed value headers, ordered value
  checking, and deferred bodies. Type/member collection and capture machinery
  are reused, including generic scopes and inferred anonymous bodies.
- The initialization pass retains the actual semantic declaration identities.
  Recursive summaries propagate reads, writes, callbacks, returned callable
  values, contracts, defaults, and captures. Invocation cycle guards terminate
  recursive callable-storage expansion conservatively.
- Callable provenance is invalidated by writes and explicit var arguments,
  including postfix method calls. Unknown startup dependencies never imply
  safety. Local capture storage must exist before an environment is created.
- Compiled interfaces retain invocation/returned-parameter dependencies and
  unknown paths, including record methods. Aliases and warm reuse preserve the
  checks. The derived unit format is version 16; old sidecars rebuild normally.
  Dependency units still initialize before consumers, with existing visibility.
- Compiler callable allocation includes nested helpers in methods and anonymous
  functions. Direct calls, first-class routine values, and go use the semantic
  declaration identity when routines and callable bindings share a name.
- Editor completion and navigation use scope-wide signatures. Definitions and
  references retain the same declaration under shadowing and later enclosing
  value references, including anonymous callable bodies.
- All 22 example/app projects accept the selected contract; no consumer
  initialization reorder or additional language syntax was needed.
  [Declaration order](../../../pascal/language/types/declaration-order.md),
  routine/capture/default/value/unit handbooks, diagnostics, compact reference,
  grammar guidance, and authoring skill describe the delivered behavior.

This is bounded internal initialization analysis. AP25 purity, new forward
syntax, lazy initialization, initializer reordering, and task-lifetime changes
are outside this package. Existing contract-expression restrictions remain.

## Verification and independent review

- Rust formatting and workspace build passed. All workspace test packages passed
  except the unchanged socket-timeout test below; the remaining VM suite passed
  with that test excluded. Workspace documentation tests passed.
- FPAS suite: 509 passed, one skipped, zero failed (510 total). All 22 example/app
  manifests checked; FPAS formatting and documentation links passed.
- Release binaries and all 111 standard-library units built. The AP38 runtime
  regression also passed with the optimized release compiler.
- Semantic regressions cover forward/generic/mutual calls, safe initializer
  effects, direct/transitive premature access, local captures, default omission,
  callback handoffs, mutable provenance, and terminating recursive analysis.
- Runtime coverage includes nested functions/procedures, methods, generic
  helpers, named helpers in inferred anonymous functions, initialization order,
  mutable captures, and direct/value/go shadowing in both directions.
- Encoded-interface and cold/warm-build regressions verify callback dependencies,
  import aliases, dependency startup, and rejection with reused sidecars.
  Editor regressions verify definitions, completion, references, and anonymous
  body references to later enclosing bindings.
- Independent review found recursive-provenance stack overflow, missing postfix
  var invalidation, and incorrect compiler shadowing. All three were repaired,
  covered by regressions, and independently rechecked. No AP38 finding remains.

The unchanged test
`vm::hosted::net::connections::cancellable_write::tests::write_deadline_is_not_reset_by_retries_and_preserves_socket_timeouts`
fails because the operating system returns 32 ms for a requested 30 ms socket
timeout. It was already reproduced at baseline 03b98038 in AP37; network
implementation and tests were not changed by AP38. The separate pre-existing
qualified record/receiver-name collision observed during review was also
reproduced with AP37 and remains outside AP38's routine-declaration changes.
