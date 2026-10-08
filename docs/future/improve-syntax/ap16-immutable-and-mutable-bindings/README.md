# AP16: Immutable and mutable bindings

Status: complete. Effort: large. Completion is tracked in the
[central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

Binding keywords match Pascal expectations: `const` cannot be reassigned,
`var` can. Initialization, constant contexts, and captures have explicit rules,
and the migration preserves each binding's mutability.

Current behavior: `const` declares an immutable binding with a static or
computed initial value; `var` declares a reassignable binding. Value parameters are
read-only unless declared `var`, and `mutable` is an ordinary identifier
([variables](../../../pascal/language/basics/variables.md),
[parameters](../../../pascal/language/functions/parameters.md)).

## Decisions

- `const` prevents reassignment of the binding and allows a computed initial
  value; `var` permits reassignment. A local initializer is evaluated once
  whenever execution reaches its declaration, in statement order. A loop-body
  declaration initializes on every iteration that reaches it; an untaken
  branch does not evaluate its initializer. Program and unit initializers
  retain their existing declaration and unit-initialization order.
- Put a computed binding before a loop when its value should be evaluated
  once for that whole loop. Binding immutability does not memoize calls or
  hoist declarations. Optimizations must preserve observable effects,
  failures, and whether the declaration is reached.
- `mutable` is an ordinary identifier. Writable reference parameters use
  `var` under AP17.
- A `for` loop variable is an immutable binding for each iteration, written
  without a keyword. Assigning to it is an error.
- Binding immutability is separate from deep immutability of shared handles and
  referenced data.
- Compile-time constants are preserved: a computed `const` is rejected where a
  compile-time constant is required, including scalar case labels, with a
  diagnostic that names the non-constant part. Subrange bounds are planned
  under AP18.
- Scalar `case` value labels and both endpoints of a range must be compile-time
  constants. Runtime expressions are rejected in these positions; express
  dynamic conditions with guards. Enum, `Option`, and `Result` patterns keep
  their existing rules.
- Keep the existing constant-expression forms when their operands are
  compile-time known. Function and method calls are runtime computations,
  including user routines, standard-library/intrinsic routines, and native
  type operations, even when a routine appears pure. A `const` initialized
  from such a call or from another computed `const` remains computed;
  dependent bindings cannot be used in compile-time constant contexts.
  Optimization or purity analysis does not change this language-level
  classification.
- Closure capture preserves the existing storage behavior: a captured `const`
  or value parameter is copied; a captured `var` shares one mutable cell.
- Teaching examples prefer `const` and use `var` only for reassignment.
  Explicit types stay until AP22's limited local inference is implemented.

```pascal
const Caption: string := MakeCaption();
var Attempts: integer := 0;
Attempts := Attempts + 1;

for I: integer := 1 to 10 do
  Log(I);
end for;
```

## Dependencies

- AP11 (one keyword per declaration).

AP17, AP18, AP19, AP22, AP23, and AP25 depend on this package.

## Work packages

- [x] [AP16.1: Computed const bindings](01-computed-const-bindings.md)
- [x] [AP16.2: Migrate immutable var to const](02-migrate-immutable-var.md)
- [x] [AP16.3: Keyword switch to const and var](03-keyword-switch.md)

## Acceptance

Initialization, constant contexts, and captures have explicit rules; the
migration is more than a keyword replacement and preserves mutability.
