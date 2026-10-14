# APNN.2: Implement the approved contract

Template only. Replace placeholders and remove this paragraph in the copied
package. Package: [APNN](README.md). Status: not started; blocked on APNN.1.

## Prerequisites

- [APNN.1](01-decision.md) is complete and its selected contract is recorded.
- `<Other actual prerequisites, linked to their owners, or None.>`
- No decision listed by this work package remains unresolved.

## Contract and scope

`<Exact accepted behavior, concrete accepted/rejected examples and preserved invariants.>`
Excluded: `<Related work owned elsewhere.>`

## Affected files and implementation owners

| Action | Path | Purpose |
| --- | --- | --- |
| `<create/modify/move/split/remove>` | `<actual repository path>` | `<One responsibility and why this work needs it>` |

Inspect module size, nearby implementations and shared logic before filling
this table. Call out reorganizations explicitly. Keep the implementation and
all required consumer migration within this bounded work package.

## Migration and documentation

- Consumers: `<All affected FPAS sources, embedded fixtures, generators and tooling.>`
- Current documentation: `<Owning handbook, grammar and tooling pages.>`
- Compact reference and generated API declarations: `<Update or confirm, as applicable.>`
- Removed forms and diagnostics: `<Canonical replacement, or Not applicable.>`

## Acceptance and verification

`<Observable pass/fail result, including preserved behavior and failure cases.>`

Name the owning tests and exact checks before implementation. Cover positive,
negative and boundary cases using existing tests where sufficient. Follow the
[required checks](../development-process.md#required-checks) and relevant
[shared boundaries](../shared-constraints.md).

After delivery and review, mark APNN.2 complete in the package README. If it is
the final work package, mark APNN complete in the central plan. Replace planned
steps with the delivered contract, implementation/regression owners and actual
open follow-ups. Report commands and results separately from the plan.
