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
The completed implementation record, decisions, regressions, and restart
instructions live in
[`variant-replacement/`](variant-replacement/README.md).

Debugger initialization of a visible, source-declared mutable local or global
before normal execution initializes that binding is implemented through the
same mutation surfaces. Descendant writes on empty storage, skipping the later
source initializer, and treating parameters or captures as uninitialized
targets remain deferred. The implementation record, verification mapping, and
restart instructions live in
[`uninitialized-binding-assignment/`](uninitialized-binding-assignment/README.md).

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
also implemented. Later mutation forms remain recorded
in [deferred.md](deferred.md).

The implemented debugger does not change FPAS syntax, semantics, or the
language specification.
