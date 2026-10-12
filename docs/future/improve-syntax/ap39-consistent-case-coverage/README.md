# AP39: Consistent case coverage

Status: Decision pending. Effort: large. Completion is tracked in the
[central plan](../README.md). Existing AP03/AP20/AP21 coverage rules remain
implemented and are not reopened merely by recording this proposal.

## Finding

The [coverage matrix](../../../pascal/language/pattern-matching/exhaustiveness.md)
depends on type and statement/expression position: enum/Option/Result must be
exhaustive and forbid else; scalar statements may be incomplete; Boolean
expressions accept full coverage or else; integer/string/subrange/distinct
expressions require else. Even enumerated finite subranges need an expression
fallback. Duplicate scalar labels can pass where unreachable variant patterns
fail. The Boolean introductory documentation mismatch is owned by AP32.

## Proposed direction and open decisions

Evaluate the principle "case handles every possibility; if selects a case."
Decide statement/expression parity, Boolean treatment, finite subrange
coverage, fallback rules, guards, scalar ranges, and unreachable-arm checks.
Preserve the useful diagnostic when an enum gains a new variant; a universal
catch-all must not silently remove that benefit without a user decision.

## Dependencies and work packages

AP03/AP20/AP21 are complete. AP39.1 precedes AP39.2. AP35's condition decision
must be settled if the selected rule uses expanded pattern predicates in
guards; otherwise its implementation is not a prerequisite.

- [ ] [AP39.1: Coverage decision](01-coverage-decision.md)
- [ ] [AP39.2: Coverage checks](02-coverage-checks.md)

## Acceptance

A compact normative matrix covers every selector family and both positions.
Missing/unreachable arms are checked consistently, and changes to closed
types still surface required handling as decided.
