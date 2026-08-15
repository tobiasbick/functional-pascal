# Source-debugger roadmap

## Active implementation plans

The [source-debugger completion umbrella](umbrella/README.md) is the active,
resumable plan for the remaining debugger backlog. It coordinates bounded
packages in dependency order; it does not authorize one monolithic
implementation. Current behavior remains documented under
[`docs/pascal/tools/`](../../pascal/tools/debugger.md).

The source debugger includes detached controlled calls, read-only expression
evaluation, watches, conditional breakpoints, exact-hit conditions,
non-stopping logpoints, and stopped-state mutation of supported mutable values.
Its current user and protocol documentation lives under
[`docs/pascal/tools/`](../../pascal/tools/debugger.md).

[deferred.md](deferred.md) lists all consciously postponed debugger work
together with its rationale and re-entry gates. Deterministic, launch-owned,
all-stop task debugging is implemented; non-stop execution, task-control
mutation, cross-task stepping shortcuts, and persistent task history remain
consciously deferred there.

The implemented debugger includes complete-value replacement of mutable enum,
`Result`, and `Option` values through the existing `setVariable`/`setExpression`
path, plus explicit metadata-driven discovery and complete construction of
fieldless, single-field, and multi-field variants through JSONL
`variant.describe` / `variant.construct`, DAP `fpas/variantDescribe` /
`fpas/variantConstruct`, and VS Code **Debug: Construct Variant**. Implicit
switching through a stale payload-child handle remains unsupported.

Debugger initialization of a visible, source-declared mutable local or global
before normal execution initializes that binding is implemented through the
same mutation surfaces. Seeded descendant initialization below empty storage
is implemented through JSONL `storage.initialize`, DAP
`fpas/initializeStorage`, and VS Code **Debug: Initialize Empty Storage**.
Skipping the later source initializer and treating parameters or captures as
uninitialized targets remain deferred.

Explicit, variant-qualified descendant assignment that constructs one complete
single-payload enum, `Result`, or `Option` variant is implemented through
textual `setExpression` / JSONL `expression.set`. Stale-handle switching,
unqualified variant guessing, multi-field incremental construction, and
virtual Variables children remain deferred.

Bounded assignment of an already materialized, visible, non-task-bound
first-class function value, of one statically resolved non-capturing
executable routine, of a named nested routine whose direct captures are
immutable values from the exact selected live lexical-owner frame, and of a
named nested routine whose `Cell` and `EnclosingCell` captures clone existing
handles from that same owner frame onto a mutable local or parameter register,
is implemented through the same `setVariable`/`setExpression` surfaces.
Constructed cell-capturing functions are task-bound to the selected task.
An already materialized task-bound function can be copied within that selected
owner task and frame onto another mutable local or parameter register; the
exact function and cell handles are preserved. Global, descendant,
capture-cell, foreign-task, and stale-frame escape paths remain rejected.
Bound-receiver synthesis from compiler-retained record method mappings is
implemented through the same mutation surfaces with exact receiver-layout,
signature, and value-graph validation. Bounded copying of an already
materialized, visible task handle onto a structurally
compatible mutable target is implemented through those same surfaces: the copy
preserves the exact runtime task ID and does not consult the scheduler. Newly
entered anonymous closures require the versioned live-image work in `UMB-90`.
Dynamic callable endpoints, opaque hosted-resource assignment, and in-place
callable child editing were rejected by `UMB-10D`. Task-bound writes into
capture-cell destinations remain blocked on stable cell identities in
`UMB-70A`.

Bounded forced return from a selected ordinary callee — including an older
frame of the stop-owning task — is implemented through JSONL `frame.return`,
DAP `fpas/forceReturn`, and the VS Code command
`functionalPascal.debug.forceReturn`. Broader control-flow mutation is owned by
`UMB-30` in the [active umbrella](umbrella/implementation-plan.md).

Textual debugger expression mutation is implemented through DAP
`setExpression` and JSONL `expression.set` for the existing bounded mutation
domain.

Explicit dictionary insertion, removal, and key replacement are implemented
through JSONL, DAP custom requests, and VS Code commands. Bounded array
insertion/removal and Unicode-scalar string character replacement are also
implemented through all three surfaces. Writable descendants of the currently
active data-carrying enum, `Result`, and `Option` payload are implemented
through standard `setVariable`/`setExpression` and their JSONL counterparts.
Complete-value replacement of those same enum, `Result`, and `Option` roots is
also implemented. Textual qualified single-payload variant transition is
implemented through `setExpression` / `expression.set`. Later mutation forms
remain recorded in [deferred.md](deferred.md).

The implemented debugger does not change FPAS syntax, semantics, or the
language specification.
