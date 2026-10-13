# AP35: Predictable condition composition

Status: complete (AP35.1–AP35.3). Effort: medium to large.
Completion is tracked in the [central plan](../README.md).

## Goal

[Nonbinding pattern tests](../../../pascal/language/pattern-matching/is-test.md)
compose as ordinary Boolean expressions. Pattern data is extracted only by
case arms. [Postconditions](../../../pascal/language/functions/contracts.md)
use an explicit `(Name):` prefix for a return binder, keeping grouped Boolean
conditions unambiguous.

## Recorded decisions

The user selected one canonical form for each purpose:

- `Value is Pattern` is a Boolean expression that tests a pattern and never
  introduces names. Grouping is transparent. It is valid wherever an ordinary
  Boolean expression is valid: conditions, `and`/`or`/`xor`/`not`, case guards,
  loops including `until`, bindings, arguments, and returns. Existing operator
  precedence, mixed-operator grouping rules, and short-circuit order remain.
- Pattern data is bound only in `case` arms, including case expressions.
  `const Name` anywhere inside an `is` pattern is rejected with a case example.
  Existing nonbinding patterns, nested variants, field wildcards, constant
  comparisons, and case binding/coverage rules remain unchanged.
- A function postcondition names its return value only with
  `ensures (Name): Expression;`. The colon is mandatory for a binder.
  `ensures (Flag);` is an ordinary grouped Boolean condition. The name remains
  immutable, has the declared return type, and belongs to that clause alone.
  Procedure postconditions cannot introduce a return binder.
- AP23's contract expression allowlist and postcondition timing remain.
  Nonbinding `is` tests in contracts obey the same operand restrictions;
  ordinary helper calls remain unavailable there. AP25 stays rejected.

AP35.2 replaces AP20.3's condition-only binding rule. AP35.3 replaces AP23/Q16's
binder punctuation while preserving return-value semantics and clause scope.
AP20 and AP23 remain complete; their records and the handbook describe the
resulting rules.

## Selected examples

These fragments show the implemented forms.
The containing program declares the referenced values and routines.

```pascal
const Present: boolean := Value is Some(_);
if not (Value is Some(_)) then
  HandleMissing();
end if;

case Value of
  when Some(const V):
    Use(V);
  when None:
    null;
end case;

function Increment(Value: integer): integer;
ensures (Returned): Returned = Value + 1;
begin
  return Value + 1;
end function;
```

## Open decisions

None within AP35's delivered scope. Case coverage, operator rules, inference,
and the contract allowlist retain their existing owners.

## Dependencies and work packages

AP20/AP23 are complete. AP35.1 supplies the recorded contract for both
implementation parts. Both parts and their repository migrations are complete.
AP39 and AP43 consume the resulting rules. Independent review has no open findings.

- [x] [AP35.1: Condition decisions](01-condition-decisions.md)
- [x] [AP35.2: Pattern conditions](02-pattern-conditions.md)
- [x] [AP35.3: Postcondition binding](03-postcondition-binding.md)

## Acceptance

Nonbinding pattern tests have the same Boolean meaning in every permitted
expression context, including grouped and negated tests. `is` never creates
names; case bindings retain their existing arm scope. Migration preserves
evaluation count, effects, and control flow. A return binder requires a colon
and cannot be mistaken for an ordinary grouped condition. Implementation,
consumer migration, documentation, regression coverage, and review are complete.
