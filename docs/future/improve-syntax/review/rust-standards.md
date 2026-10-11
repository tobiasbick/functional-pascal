# Rust standards review

This is the **Standards** axis of the [current review](README.md), separate from
the semantic and tooling findings.

## Scope and result

The reviewed range is `5b04c39697f2d57e6390648194ae294ddff614bb` through
`34397373799a5897dfcfc09e754d16fcad3c7386`. Requirements come from
[AGENTS.md](../../../../AGENTS.md),
[development-process.md](../development-process.md), and the repository's
Rust and test-audit skills. Inspection covered new semantic, lowering,
generic-substitution, contract, formatter and language-service modules.

**No independent, actionable Rust structure or style violation was confirmed.**
New features generally use focused concern modules; event owners were removed
coherently. Large existing files, unmeasured clone costs and lint-enforced
stylistic choices are not findings by themselves.

Duplication, misplaced responsibilities, repeated dispatch, unclear names,
unnecessary delegation and speculative abstractions were considered as
judgement calls. No separate refactor package was justified. Restructuring
needed for repairs should stay within their owners and follow AGENTS concern
and file-size rules.

## Completion and test quality

The concrete defects belong to the **Spec** axis in
[compiler-and-semantics.md](compiler-and-semantics.md) and
[tooling-and-tests.md](tooling-and-tests.md). They also contradict completion
expectations, but are not counted again as style findings.

Existing tests were reviewed for their contract, inputs and production entry
point. No test deletion is proposed. Missing cases are listed with each repair.
Prefer extending existing owner-boundary fixtures; use another layer only for
a distinct risk such as diagnostic transport or runtime invocation.

Every new regression must fail on the reviewed revision for its stated reason
and pass after the owner is fixed. A source grep or test restating the new
helper implementation is not adequate evidence.

## Checks and closure

Actual checks and their limits are in [verification.md](verification.md).
The fixed-scope review found no separate standards repair. Implementation
repairs and regressions are tracked on the Spec axis; current handbook rules
remain unchanged. Reorganizations stay within the affected owners.
