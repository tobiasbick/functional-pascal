# Task 23 — Decide ownership of a source shared by consumer and dependency

Status: decision required
Severity: P1 invariant risk; behavior pending
Difficulty: medium after decision
Language gate: yes (project-model specification)
Depends on: none

## Question

What happens when one physical `.fpas` file is included by the consumer's `[sources]` and also
arrives through a library dependency?

## Verified ambiguity

`dependencies.rs::merge_dependency` first records a library origin. Later,
`mark_own_source_origins` overwrites it with `Own` when the consumer includes the same file. This can
bypass `[exports]` checks or cause library-internal edges to be evaluated with consumer ownership.
The current [`Projects`](../../../pascal/program-structure/projects.md) page discusses duplicate
source entries inside one source list, but does not define cross-project physical-file overlap.

## Options for user agreement

1. **Reject overlap (recommended):** one physical source has one project owner. Emit an actionable
   manifest error naming both projects.
2. **Library ownership wins:** deduplicate the file and retain the dependency/export boundary.
3. **Consumer ownership wins:** document that explicit consumer inclusion intentionally escapes
   dependency exports; not recommended because it weakens the boundary.

## Tests after decision

- Physical equality and symlink/alias equality use `same_file` consistently.
- Non-exported dependency units cannot be imported accidentally.
- Library-internal imports and ordinary non-overlapping projects remain valid.
- Update `projects.md` with the selected ownership rule.

## Decision record

- Selected option: pending
- Approved by user: pending
