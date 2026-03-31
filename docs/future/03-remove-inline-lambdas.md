# Remove: Inline Anonymous Functions (Lambdas)

> Priority: 3 — no other feature depends on inline lambdas.

## What to remove

- Inline anonymous function expressions:
  `function(X: integer): integer begin return X * X end`
- Closure capture by value for anonymous functions.

## What stays

- **First-class named functions** — passing a named function as a value.
- **Nested function declarations** — `function Inner(...) begin ... end;`
  inside another function, with lexical scope access.

Named nested functions cover the same use-cases as lambdas with better
readability. `Map(Items, Double)` is cleaner than a multi-line inline lambda.

## Scope

- **Parser:** remove `function_expr` production.
- **Compiler:** remove anonymous function compilation, capture analysis for
  inline lambdas (keep capture for nested named functions if needed).
- **Grammar (EBNF):** remove `function_expr` from `primary_expr`.
- **Docs:** remove lambda section from `04-functions.md`. Update
  `closures` example to use nested named functions.
