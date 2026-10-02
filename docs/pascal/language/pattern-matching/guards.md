# Guards

Add conditions to case arms with `if`. The guard is evaluated after the label matches; the arm executes only when the guard is `true`:

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (guarded `case_label`).

```pascal
function Classify(N: integer): string;
begin
  case N of
    when 0:
      return 'zero';
    when N if N > 0:
      return 'positive';
    when N if N < 0:
      return 'negative';
  end case;
end function;

```

Guards work with all label types — values, ranges, destructuring, and enum patterns:

```pascal
uses Std.Console as Console;

case S of
  when Shape.Circle(R) if R > 10.0:
    Console.WriteLn('Large circle');
  when Shape.Circle(R):
    Console.WriteLn('Small circle');
  when Shape.Rectangle(W, H) if W = H:
    Console.WriteLn('Square');
  when Shape.Rectangle(W, H):
    Console.WriteLn('Rectangle');
  when Shape.Point:
    Console.WriteLn('Point');
end case;
```

The guard expression has access to any bindings introduced by the label.
For enum patterns, pattern arguments bind names only; put literals and extra checks in the `if` guard.

## Scalar guard bindings

In scalar `case` arms, a single bare identifier with a guard introduces a binding for the matched value:

```pascal
uses Std.Console as Console;

case Value of
  when N if N > 0:
    Console.WriteLn('positive');
  when N if N < 0:
    Console.WriteLn('negative');
  else
    Console.WriteLn('zero');
end case;
```

`N` is available in both the guard and the arm body, but only inside that arm.

Rules:

- The arm must have exactly one label.
- The label must be a single bare identifier, not a range or a comma-separated label list.
- If the identifier resolves to a compile-time constant or enum member, it remains a normal value label instead of becoming a binding.

## See also

- [Enum patterns](enum-patterns.md)
- [Exhaustiveness](exhaustiveness.md)
