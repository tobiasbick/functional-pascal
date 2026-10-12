# AP12.1: Callable expression decision

Package: [AP12: Callable expressions](README.md)

Status: complete. The user decided to defer AP12 and retain it for possible
later reassessment. This decision changes no language or application code.

## Scope

Record that AP12 does not proceed now. Additional call targets and evaluation
order remain unapproved; deciding them is a prerequisite if AP12 is reopened.

## Prerequisites

- AP06.1 (dot-call decisions), so the relation between callable fields and
  methods is settled.

## Decision and rationale

Direct returned-function calls such as `MakeAdder(3)(5)` do not fit the user's
preferred readability. Existing anonymous functions, named callable bindings,
indexed callable designators, and record-method chains provide alternatives.

The proposal is deferred, not permanently rejected. AP12.2 remains as an
inactive implementation sketch. No complete parser/checker inventory or
approval of a general call suffix is claimed. The
[package README](README.md#decision) records the decision and reopening gate.

## Affected areas

- `docs/future/improve-syntax/ap12-callable-expressions/`.
- `docs/future/improve-syntax/README.md` (status).

## Migration

None.

## Documentation

Planning documentation only.

## Verification

- The package and central plan record deferral; AP12.1 is complete as a
  decision, while AP12.2 remains unimplemented and unscheduled.
- The retained proposal and implementation sketch remain linked, without
  claiming that draft syntax is implemented or approved.
