# AP01: Pascal conventions

Status: complete. Effort: small. Completion is tracked in the
[central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

Give the whole plan one consistent, Pascal-oriented reference style so that
later packages and their examples use the same spellings. Proposed spellings
are drafts, not current-language examples.

## Decisions

- Retain case-insensitivity and Pascal's routine, record, case, and block words.
- Keep `array of T` and use `of` for every type application, including
  user-defined generic types (AP24). Do not introduce `Array<T>`, `Option<T>`,
  or other angle-bracket type applications.
- Where Pascal or Delphi has an established spelling for a planned concept
  (subranges, distinct types, `var` parameters), use it unless a recorded
  decision says otherwise. Recorded exceptions: `distinct`
  ([AP19, Q12](../ap19-distinct-domain-types/README.md)).
- Generic routine type-parameter declarations keep angle brackets, such as
  `function Identity<T>(Value: T): T;`; calls infer their type arguments.
  This is not a type application
  ([AP24](../ap24-generic-data-structures/README.md)).
- A formatter change is not required for this package.

## Dependencies

None. AP05, AP11, and AP13 depend on this package.

## Order

AP01.1 establishes the reference examples; AP01.2 checks the plan against them.

## Work packages

- [x] [AP01.1: Reference examples](01-reference-examples.md)
- [x] [AP01.2: Plan spelling review](02-plan-spelling-review.md)

## Acceptance

The reference examples state a consistent Pascal-oriented form without
presenting unresolved grammar as implemented behavior.

## Delivery

The [reference style](reference-style.md) provides ten annotated draft examples.
The [spelling review](spelling-review.md) records the plan-wide checks,
corrections, and decisions left with the owning packages. Both work packages
are complete on the working branch under the
[development process](../development-process.md#status-tracking).
