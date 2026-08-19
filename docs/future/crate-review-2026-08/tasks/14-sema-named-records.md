# Task 14 — Decide named-record compatibility

Status: decision required
Severity: unclassified until decision
Difficulty: medium after decision
Language gate: yes
Depends on: 15

## Question

Are distinct named records nominally incompatible, or are records with compatible public shapes
structurally compatible?

## Why this is blocked

`Ty::compatible_with` currently compares records structurally when neither has private members.
Records with private members use additional name/owner checks. The current
[`Records`](../../../pascal/language/types/records.md) page documents declaration, literals,
defaults, mutation, and visibility but does not define type identity or assignment compatibility.
Changing the rule would therefore change FPAS semantics rather than implement a written rule.

## Options for user agreement

1. **Nominal named records (recommended):** different declared names are incompatible; anonymous
   literals receive the expected named type contextually through task 15.
2. **Structural public records:** keep compatible shapes interchangeable and document the rule,
   including methods/properties/events and owner/private-member boundaries.

Record the selected option here before changing code or `docs/pascal/language/types/records.md`.

## Implementation after decision

- Add positive same-type/alias tests and negative/positive cross-name tests matching the choice.
- Preserve anonymous record literal contextual typing from task 15.
- Treat owner identity and private members consistently across source and imported interfaces.
- Update the records specification with the selected compatibility rule.

## Decision record

- Selected option: pending
- Approved by user: pending
