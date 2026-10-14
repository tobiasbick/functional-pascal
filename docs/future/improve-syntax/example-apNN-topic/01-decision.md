# APNN.1: Decide the contract

Template only. Replace placeholders and remove this paragraph in the copied
package. Package: [APNN](README.md). Status: Decision pending.

## Deliverable

`<The precise questions this work package resolves and where their decisions are recorded.>`
The deliverable is a recorded user decision, not an implementation.

## Prerequisites

`<Existing behavior, evidence and earlier decisions needed to evaluate this proposal.>`

## Scope and evidence

- Inspect `<owning implementation, current documentation, consumers and tests>`.
- Record a concrete current example and the expected behavior under the proposal.
- Compare the smallest viable contracts, their migration cost and excluded scope.
- Identify affected compiler, runtime, artifact, CLI and editor boundaries.

## Affected files

| Path | Purpose |
| --- | --- |
| README.md | Record the selected contract, decision IDs, open details and dependencies |
| 02-implementation.md | Reflect the selected scope and its prerequisites |

Replace or extend this table with the actual paths before starting work.

## Acceptance and verification

`<Questions requiring an explicit user answer.>`
After the answer, record it once in the package README and align dependent
files. Verify that examples, scope, prerequisites and acceptance agree, that
relative links resolve and that `git diff --check` passes.

Mark APNN.1 complete in the package README only after the decision is recorded.
An accepted direction with unresolved blocking details does not authorize
APNN.2. Follow the [decision gates](../development-process.md#decision-gates).
