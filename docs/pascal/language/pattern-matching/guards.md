# Guards

Add conditions to case arms with `if`. The guard is evaluated after the label matches; the arm executes only when the guard is `true`:

Labels are tried in written order. If several overlapping labels in one arm
match, a false guard continues to the next label and can be evaluated again.
The selector is evaluated once; bindings come from the label being tried.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (guarded `case_label`).

```pascal
function Classify(N: integer): string;
begin
  case N of
    when 0:
      return 'zero';
    when const N if N > 0:
      return 'positive';
    else
      return 'negative';
  end case;
end function;
```

The final `else` supplies an explicit return for every remaining input. The
compiler does not prove that arbitrary guards cover all possible scalar values.
Guards never contribute coverage, even constant-true guards. Boolean and
subrange cases require unguarded finite coverage and forbid `else`.

Guards work with all label types — values, ranges, destructuring, and enum patterns:

```pascal
case S of
  when Shape.Circle(const R) if R > 10.0:
    WriteLn('Large circle');
  when Shape.Circle(const R):
    WriteLn('Small circle');
  when Shape.Rectangle(const W, const H) if W = H:
    WriteLn('Square');
  when Shape.Rectangle(const W, const H):
    WriteLn('Rectangle');
  when Shape.Point:
    WriteLn('Point');
end case;
```

The guard expression has access to any bindings introduced by the label.
For enum, Result, and Option patterns, payload positions bind with `const Name`, ignore with `_`, or compare with constants; put ranges and computed checks in the `if` guard.

## Scalar guard bindings

In scalar `case` arms, `const Name` with a guard binds the matched value:

```pascal
case Value of
  when const N if N > 0:
    WriteLn('positive');
  when const N if N < 0:
    WriteLn('negative');
  else
    WriteLn('zero');
end case;
```

`N` is available in both the guard and the arm body, but only inside that arm.

Rules:

- `const Name` must be the arm's only label and requires a guard (FP3032).
  Integer/string cases use `else` for remaining values; finite selectors use
  explicit unguarded arms according to the [coverage matrix](exhaustiveness.md).
- It is valid in scalar `case` statements and expressions. A case expression
  must produce a value for every input, using the same coverage rules as a
  statement. Result, Option, and data-enum patterns bind inside the pattern,
  for example `Some(const Value)`.
- A bare identifier label is always a value comparison. It must name a
  compile-time constant or an enum member; a computed `const` is rejected. A
  name that resolves to nothing reports FP3031 with the `const Name` form.

## See also

- [Enum patterns](enum-patterns.md)
- [Exhaustiveness](exhaustiveness.md)
