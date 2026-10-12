# AP35: Predictable condition composition

Status: Decision pending. Effort: medium to large. Completion is tracked in
the [central plan](../README.md).

## Findings

The [is-test rules](../../../pascal/language/pattern-matching/is-test.md)
reject parentheses, `or`, `not`, guards, `until`, and value positions even
for tests that bind no names. A normal grouping edit changes validity.

The [contract grammar](../../../pascal/language/functions/contracts.md)
uses leading `(Name)` as a return binder: `ensures Flag` works, while
`ensures (Flag)` is consumed as a binder and fails. Parenthesized compound
conditions work, making ordinary expression composition unpredictable.

## Proposed direction and open decisions

- At minimum make grouping transparent for supported pattern conditions.
  Decide whether `is` is a true Boolean predicate or explicitly condition-only.
- Define binding visibility for each permitted Boolean context; do not expose
  names on paths where the pattern has not matched.
- Give the postcondition return binder an unambiguous delimiter or form.
  A colon after a binder is a candidate, not an approved spelling.
- Keep AP23's restricted contract expressions; this does not reopen rejected
  AP25, add purity metadata, or allow arbitrary helper calls in contracts.

## Dependencies and work packages

AP20/AP23 are complete. AP35.1 precedes both independent implementation parts.
AP39 and AP43 consume the resulting pattern rules.

- [ ] [AP35.1: Condition decisions](01-condition-decisions.md)
- [ ] [AP35.2: Pattern conditions](02-pattern-conditions.md)
- [ ] [AP35.3: Postcondition binding](03-postcondition-binding.md)

## Acceptance

Permitted conditions remain valid when harmless grouping is added. Return
binding cannot be mistaken for an ordinary Boolean expression. Pattern-bound
names remain scoped and initialized on every path that can use them.
