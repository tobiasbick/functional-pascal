# APNN: `<Topic>`

Template only. Copy this directory as described in the
[central plan](../README.md#create-a-package), replace all placeholders and
remove this paragraph. This example is not scheduled or approved work.

Status: Decision pending. Effort: `<estimate>`.
Completion is tracked only in the [central plan](../README.md#package-status).

## Goal

`<Concrete problem, affected users and the observable result this package delivers.>`

## Current behavior and proposed contract

- Current behavior: `<Checkout-backed example and owning handbook or source path.>`
- Proposed contract: `<The smallest precise replacement, identified as a proposal.>`
- Preserved behavior: `<Evaluation, effects, identity and other relevant invariants.>`
- Excluded scope: `<Related work this package does not deliver.>`

## Decisions

| ID | Question | User decision | Affected work packages |
| --- | --- | --- | --- |
| QNN | `<Required design choice>` | Pending | APNN.1, APNN.2 |

Record the actual user decision here. Agreement on a direction, a complete
specification, implementation and verification are separate states.

## Open decisions

`<List unresolved choices and the work they block. Write None only after they are resolved.>`

## Dependencies and work packages

Prerequisites: `<Link actual package/work-package prerequisites or write None.>`
Execution order follows those prerequisites, not the package number.

- [ ] [APNN.1: Decide the contract](01-decision.md)
- [ ] [APNN.2: Implement the approved contract](02-implementation.md)

This is the only work-package checklist for this AP. Follow
[status tracking](../development-process.md#status-tracking); completion does
not imply commit or merge approval.

## Acceptance

`<Observable pass/fail criterion for the whole package, including rejected and boundary cases.>`

## Documentation and regression owners

- Current documentation: `<Owning handbook, grammar or tooling pages.>`
- Implementation: `<Owning modules and shared implementations.>`
- Tests: `<Existing contract coverage and distinct gaps, if any.>`
- Cross-package checks: [shared constraints](../shared-constraints.md).
