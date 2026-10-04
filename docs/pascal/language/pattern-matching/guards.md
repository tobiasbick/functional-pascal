# Guards

Add a Boolean condition to a case arm with `if`. The complete pattern must match
before the guard runs. If the guard is false, matching continues with the next
arm. The scrutinee is evaluated once, and only the selected body runs.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`case_arm`,
`case_value_arm`, and `pattern`).

```pascal
function Classify(N: integer): string;
begin
  return case N of
    when 0: 'zero';
    when const Value if Value > 0: 'positive';
    else 'negative';
  end case;
end function;
```

`const Value` introduces an immutable binding for the matched value. A plain
identifier refers to a static constant and never introduces a binding, including
when the arm has a guard. Function calls and nonstatic comparisons belong in
guards, rather than value labels or range endpoints.

## Payload bindings

Bindings introduced by any nested payload pattern are available in the guard
and body:

```pascal
case S of
  when Shape.Circle(const Radius) if Radius > 10.0:
    Console.WriteLn('Large circle');
  when Shape.Circle(_):
    Console.WriteLn('Small circle');
  when Shape.Rectangle(const Width, const Height) if Width = Height:
    Console.WriteLn('Square');
  when Shape.Rectangle(_, _):
    Console.WriteLn('Rectangle');
  when Shape.Point:
    Console.WriteLn('Point');
end case;
```

Literal and static-constant payload patterns compare values directly; they do
not need a guard. `_` ignores a payload without introducing a name.

## Binding scope and coverage

Bindings belong to one arm. They may shadow outer locals, but cannot shadow an
import qualifier or be redeclared within the same arm scope. Grouped patterns
must introduce the same binding names with the same types. Each grouped arm
evaluates its guard once after the first matching alternative.

Guards do not establish exhaustiveness. Closed enum, Option, and Result cases
need unguarded coverage of every named variant and its payload alternatives.
See [Exhaustiveness](exhaustiveness.md).

## See also

- [Enum patterns](enum-patterns.md)
- [Scalar labels](scalar-labels.md)
- [Exhaustiveness](exhaustiveness.md)
