# Pattern test with `is`

`Value is Pattern` tests one pattern as the condition of `if`, `elsif`, or
`while` and binds the pattern's names for the guarded body:

```pascal
if Msg is TuiMsg.Resize(const W, const H) and W > 0 then
  Relayout(W, H);
elsif Msg is TuiMsg.Key(const Key) then
  HandleKey(Key);
end if;

while Queue.Next() is Some(const Job) do
  Run(Job);
end while;
```

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`comparison_expr`, `pattern`).

## Patterns

The right side is any pattern a `case` label accepts: `const Name` bindings,
`_` for ignored fields, literal and constant comparisons, and nested variant
and Result/Option patterns (see [Syntax](syntax.md#pattern-bindings) and
[Nested patterns](syntax.md#nested-patterns-and-comparisons)). `_` cannot stand
for the whole value.

## Where `is` is valid

- `is` binds like a comparison: `X is Some(const V) and V > 0` means
  `(X is Some(const V)) and (V > 0)`.
- It is valid only as an `if`, `elsif`, or `while` condition, alone or joined
  with other conditions by `and`. Each condition of the `and` chain may be an
  `is` test.
- Its bindings are visible in later conditions of the same `and` chain and in
  the guarded branch or loop body. They are not visible in `else`, in later
  `elsif` conditions, or after the statement.
- `is` under `or` or `not`, inside parentheses, in a `case` guard, in
  `repeat ... until`, or as a value is rejected (FP3034). Use a `case` for those
  positions.
- `is` has no exhaustiveness check; use `case` when every variant needs
  handling.

## Evaluation

The tested value is evaluated once. Conditions run left to right and stop at
the first one that fails, as with `and` elsewhere. In a `while` loop, the
condition and its bindings are evaluated anew before every iteration.

`is` is a reserved keyword.

## See also

- [Syntax](syntax.md)
- [Exhaustiveness](exhaustiveness.md)
- [If / then / else](../control-flow/if-then-else.md)
- [While and repeat](../control-flow/while-repeat.md)
