# AP39: Consistent case coverage

Status: Complete; shared coverage, reachability, migration, and tooling implemented.
Effort: large. Completion is tracked in the [central plan](../README.md).

## Addressed finding

The [coverage matrix](../../../pascal/language/pattern-matching/exhaustiveness.md)
previously depended on type and statement/expression position. Scalar statements
could be incomplete, Boolean expressions accepted else, and explicitly covered
subrange expressions still needed a fallback. Duplicate scalar labels could
pass where unreachable variant patterns failed. AP39 replaces these distinctions
with one exhaustive selector policy and shared reachability checks.

## Selected contract

`case` handles every possibility; `if` handles selective conditions. Statements
and expressions use the same coverage and reachability rules.

| Selector | Required unguarded coverage | `else` |
|----------|-----------------------------|--------|
| Enum, Option, Result | Every variant explicitly, including nested patterns | Forbidden |
| Boolean | Both true and false | Forbidden |
| Subrange | The entire declared range, using values or ranges | Forbidden |
| Integer, string | Explicit labels or guarded arms plus a fallback | Required |
| Distinct | The rules of its supported underlying selector type | Follows the underlying type |

- Guards never contribute coverage, even constant-true guards. Earlier guarded
  arms never make later labels unreachable. Existing guard evaluation order
  and case-only binding scopes remain intact.
- Apply coverage recursively to nested patterns. Payloads may still be ignored
  with `_` or bound with `const Name`; these cover their entire payload position.
  No whole-selector catch-all replaces explicit closed-type handling or the
  required scalar `else`.
- Reject duplicate labels within an arm and labels fully covered by earlier
  unguarded arms, for every supported selector type. Equivalent compile-time
  values and equivalent matched ranges count as the same label. Partial overlaps
  remain legal: the first matching arm with a successful guard wins.
- Restrict reasoning to existing patterns, compile-time values, and supported
  ranges. Subrange coverage uses interval reasoning without enumerating every
  value. Do not prove arbitrary guard conditions or add new pattern syntax.
- Selective statements migrate to `if`, or explicitly handle the remaining
  cases with existing `null` statements. Missing coverage and forbidden `else`
  diagnostics give the canonical repair.

The full decision and example matrix are recorded in
[AP39.1](01-coverage-decision.md). AP39 replaces the scalar coverage distinctions
from AP20/AP21 and the corresponding AP18/AP19 case rules. AP03's explicit
closed-variant handling is retained. All completed packages keep their status.

## Open decisions

None. AP39.2 delivers the selected contract, including diagnostics, consumer
migration, current language documentation, editor guidance, and regressions.

## Dependencies and work packages

AP03/AP18/AP19/AP20/AP21 are complete. AP39.1 precedes AP39.2. AP35's implemented
nonbinding conditions and case-only pattern bindings are preserved; this
package introduces no additional guard context.

- [x] [AP39.1: Coverage decision](01-coverage-decision.md)
- [x] [AP39.2: Coverage checks](02-coverage-checks.md)

## Acceptance

One matrix covers statements and expressions, with consistent duplicate and
unreachable-label checks. Finite subranges need no fallback when explicitly
covered; extending an enum or subrange reveals missing coverage. All consumers
and language/editor guidance follow the selected contract.
