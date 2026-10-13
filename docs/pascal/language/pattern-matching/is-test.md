# Pattern test with `is`

`Value is Pattern` evaluates to `boolean`. It tests a value without introducing
names and can appear wherever a Boolean expression is accepted:

```pascal
const Present: boolean := Item is Some(_);
if (Item is None) or not Present then
  ReportMissing();
end if;
return Response is Ok(Some(_));
```

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`comparison_expr`, `pattern`).

## Patterns

The right side accepts `_` for ignored fields, literal and compile-time constant
comparisons, and nested variant and Result/Option patterns, using the same
validation as `case` (see [Syntax](syntax.md#pattern-bindings) and
[Nested patterns](syntax.md#nested-patterns-and-comparisons)). `_` cannot stand
for the whole value. Every `const Name` binding is rejected, including bindings
inside nested patterns (FP3034). An unknown comparison name is also rejected;
it never creates a binding.

Use a `case` arm when you need a payload:

```pascal
case Item of
  when Some(const Value):
    Use(Value);
  when None:
    null;
end case;
```

Pattern names are introduced only by `case` arms, in statements and expressions.

Named fields follow the same complete-list rules as case patterns:
`Item is Some(Value := _)` and
`ShapeValue is Shape.Rect(Height := 3, Width := _)` are valid.
Labels denote declared fields; they introduce no variables. Nested `const`
bindings remain forbidden, including `Some(Value := const N)`.

## Where `is` is valid

- `is` has comparison precedence: `X is Some(_) and Ready` means
  `(X is Some(_)) and Ready`.
- Grouping and `and`, `or`, `xor`, and `not` follow the ordinary
  [operator rules](../basics/operators.md). Mixing logical operator kinds
  requires parentheses, as elsewhere.
- Tests work in `if`/`elsif`, `while`, `repeat ... until`, `case` guards, stored
  values, arguments, returns, and decision expressions.
- [Contracts](../functions/contracts.md) accept nonbinding tests with their
  existing restrictions on operands and operations.
- `is` has no exhaustiveness check; use `case` when every variant needs
  handling.

## Evaluation

The tested value is evaluated once per evaluation of the expression. Nested
payloads are inspected only after their enclosing variant matches. Ordinary
Boolean short-circuit rules apply: `and` skips its right operand when the left
is false; `or` skips it when the left is true. `xor` evaluates both operands.
Loop conditions evaluate the test anew each time the condition is reached.

`is` is a reserved keyword.

## See also

- [Syntax](syntax.md)
- [Exhaustiveness](exhaustiveness.md)
- [If / then / else](../control-flow/if-then-else.md)
- [While and repeat](../control-flow/while-repeat.md)
