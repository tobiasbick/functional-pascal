# Source-debugger roadmap

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
path. Implicit switching through a stale payload-child handle remains deferred.

Debugger initialization of a visible, source-declared mutable local or global
before normal execution initializes that binding is implemented through the
same mutation surfaces. Descendant writes on empty storage, skipping the later
source initializer, and treating parameters or captures as uninitialized
targets remain deferred.

Explicit, variant-qualified descendant assignment that constructs one complete
single-payload enum, `Result`, or `Option` variant is implemented through
textual `setExpression` / JSONL `expression.set`. Stale-handle switching,
unqualified variant guessing, multi-field incremental construction, and
virtual Variables children remain deferred.

Bounded assignment of an already materialized, visible, non-task-bound
first-class function value onto a structurally compatible mutable target is
implemented through the same `setVariable`/`setExpression` surfaces. Function
construction, task-bound closures, Dynamic endpoints, and opaque resources
remain recorded centrally in [deferred.md](deferred.md).

Bounded forced return from the active ordinary callee is implemented through
JSONL `frame.return`, DAP `fpas/forceReturn`, and the VS Code command
`functionalPascal.debug.forceReturn`. Remaining control-flow exclusions stay in
[`forced-return/consciously-deferred.md`](forced-return/consciously-deferred.md).

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
