# Guards

Add conditions to case arms with `if`. The guard is evaluated after the label matches; the arm executes only when the guard is `true`:

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (guarded `case_label`).

```pascal
function Classify(N: integer): string;
begin
  case N of
    0:
    begin
      return 'zero';
    end;
    N if N > 0:
    begin
      return 'positive';
    end;
    N if N < 0:
    begin
      return 'negative';
    end;
  end;
end function;
```

Guards work with all label types — values, ranges, destructuring, and enum patterns:

```pascal
case S of
  Shape.Circle(R) if R > 10.0:
  begin
    WriteLn('Large circle');
  end;
  Shape.Circle(R):
  begin
    WriteLn('Small circle');
  end;
  Shape.Rectangle(W, H) if W = H:
  begin
    WriteLn('Square');
  end;
  Shape.Rectangle(W, H):
  begin
    WriteLn('Rectangle');
  end;
  Shape.Point:
  begin
    WriteLn('Point');
  end;
end;
```

The guard expression has access to any bindings introduced by the label.
For enum patterns, pattern arguments bind names only; put literals and extra checks in the `if` guard.

## Scalar guard bindings

In scalar `case` arms, a single bare identifier with a guard introduces a binding for the matched value:

```pascal
case Value of
  N if N > 0:
  begin
    WriteLn('positive');
  end;
  N if N < 0:
  begin
    WriteLn('negative');
  end;
  else
  begin
    WriteLn('zero');
  end;
end;
```

`N` is available in both the guard and the arm body, but only inside that arm.

Rules:

- The arm must have exactly one label.
- The label must be a single bare identifier, not a range or a comma-separated label list.
- If the identifier resolves to a compile-time constant or enum member, it remains a normal value label instead of becoming a binding.

## See also

- [Enum patterns](enum-patterns.md)
- [Exhaustiveness](exhaustiveness.md)
