# Task 18 — Decide public signatures that mention private types

Status: decision required
Severity: unclassified until decision
Difficulty: medium after decision
Language gate: yes
Depends on: none

## Question

What does a public unit declaration mean when its signature contains a unit-private named type?

## Why this is blocked

`interface/export.rs` can serialize such a signature while omitting the private type declaration.
The current [`Visibility`](../../../pascal/program-structure/visibility.md) and
[`Units`](../../../pascal/program-structure/units.md) pages say which declarations are exported but
do not state whether a public signature may expose a private type. Rejecting it is a new semantic
rule unless the user selects it.

## Options for user agreement

1. **Reject the declaration (recommended):** diagnose at the exporting unit with a hint to make the
   type public or stop exporting the declaration.
2. **Opaque public use:** define and implement an interface representation that lets importers hold
   or pass the value without naming/constructing its private type. This is substantially larger.

Do not export the private type implicitly; that would defeat the existing visibility modifier.

## Implementation after decision

- Walk all nested signature types, including arrays, tasks, generic callables, records, and enums.
- Add unit-interface tests for return, parameter, global, and nested type positions.
- Update visibility/unit documentation with the selected rule.

## Decision record

- Selected option: pending
- Approved by user: pending
