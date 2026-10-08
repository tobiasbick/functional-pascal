# Guards

Add conditions to case arms with `if`. The guard is evaluated after the label matches; the arm executes only when the guard is `true`:

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (guarded `case_label`).

```pascal
function Classify(N: integer): string;
begin
  case N of
    when 0:
      begin
        return 'zero';
      end;
    when const N if N > 0:
      begin
        return 'positive';
      end;
    when const N if N < 0:
      begin
        return 'negative';
      end;
  end case;
end function;
```

Guards work with all label types — values, ranges, destructuring, and enum patterns:

```pascal
case S of
  when Shape.Circle(const R) if R > 10.0:
    begin
      WriteLn('Large circle');
    end;
  when Shape.Circle(const R):
    begin
      WriteLn('Small circle');
    end;
  when Shape.Rectangle(const W, const H) if W = H:
    begin
      WriteLn('Square');
    end;
  when Shape.Rectangle(const W, const H):
    begin
      WriteLn('Rectangle');
    end;
  when Shape.Point:
    begin
      WriteLn('Point');
    end;
end case;
```

The guard expression has access to any bindings introduced by the label.
For enum, Result, and Option patterns, payload positions bind with `const Name`, ignore with `_`, or compare with constants; put ranges and computed checks in the `if` guard.

## Scalar guard bindings

In scalar `case` arms, `const Name` with a guard binds the matched value:

```pascal
case Value of
  when const N if N > 0:
    begin
      WriteLn('positive');
    end;
  when const N if N < 0:
    begin
      WriteLn('negative');
    end;
  else
    begin
      WriteLn('zero');
    end;
end case;
```

`N` is available in both the guard and the arm body, but only inside that arm.

Rules:

- `const Name` must be the arm's only label and requires a guard; use `else`
  for the remaining values (FP3032).
- It is valid only in scalar `case` statements. Result, Option, and data-enum
  patterns bind inside the pattern, for example `Some(const Value)`.
- A bare identifier label is always a value comparison. It must name a
  compile-time constant or an enum member; a computed `const` is rejected. A
  name that resolves to nothing reports FP3031 with the `const Name` form.

## See also

- [Enum patterns](enum-patterns.md)
- [Exhaustiveness](exhaustiveness.md)
