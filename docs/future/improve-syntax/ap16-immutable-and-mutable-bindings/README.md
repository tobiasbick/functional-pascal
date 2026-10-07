# AP16: Immutable and mutable bindings

Status: agreed direction. Effort: large. Completion is tracked in the
[central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

Binding keywords match Pascal expectations: `const` cannot be reassigned,
`var` can. Initialization, constant contexts, and captures have explicit rules,
and the migration preserves each binding's mutability.

Current behavior: `var` declares an immutable binding, `mutable var` a mutable
one, and `const` a compile-time constant
([variables](../../../pascal/language/basics/variables.md)). Readers trained on
Pascal and most other languages expect `var` to be reassignable.

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
- The `mutable` keyword is removed from the language, for bindings and
  parameters (see AP17).
- A `for` loop variable is an immutable binding for each iteration, written
  without a keyword. Assigning to it is an error.
- Binding immutability is separate from deep immutability of shared handles and
  referenced data.
- Compile-time constants are preserved: a computed `const` is rejected where a
  compile-time constant is required (for example subrange bounds or case
  labels), with a diagnostic that names the non-constant part.
- Keep the existing constant-expression forms when their operands are
  compile-time known. Function and method calls are runtime computations,
  including user routines, standard-library/intrinsic routines, and native
  type operations, even when a routine appears pure. A `const` initialized
  from such a call or from another computed `const` remains computed;
  dependent bindings cannot be used in compile-time constant contexts.
  Optimization or purity analysis does not change this language-level
  classification.
- Closure capture keeps today's behavior: a captured `const` is copied, a
  captured `var` shares one mutable cell (today's `mutable var` rule).
- Teaching examples prefer `const` and use `var` only for reassignment.
  Explicit types stay until AP22's limited local inference is implemented.
- The keyword migration (immutable `var` to `const`, `mutable var` to `var`)
  happens in one step together with the parameter keyword change of AP17.

```pascal
const Caption: string := Text.Trim(Input);
var Attempts: integer := 0;
Attempts := Attempts + 1;

for I: integer := 1 to 10 do
  Log(I);
end for;
```

## Dependencies

- AP11 (one keyword per declaration; AP11.2 lets a former group be split into
  `const` and `var` declarations).

AP17, AP18, AP19, AP22, AP23, and AP25 depend on this package.

## Order

AP16.1 adds computed `const` without changing `var`. AP16.2 moves every
immutable `var` to `const`; this preserves behavior and is valid in the current
language after AP16.1. AP16.3 is the single keyword switch: `var` becomes
reassignable and `mutable` disappears from bindings and parameters. AP17's
`var` parameters follow in AP17.1, so `var` never means two different things
on `main`.

## Work packages

- [ ] [AP16.1: Computed const bindings](01-computed-const-bindings.md)
- [ ] [AP16.2: Migrate immutable var to const](02-migrate-immutable-var.md)
- [ ] [AP16.3: Keyword switch to const and var](03-keyword-switch.md)

## Acceptance

Initialization, constant contexts, and captures have explicit rules; the
migration is more than a keyword replacement and preserves mutability.

## Reference

The reference branch `codex/syntax-changes` found that `check/decl/consts.rs`
requires static initialization and that `decl/vars.rs` has a special bare-task
inference path; it separated static-expression classification from binding
mutability. It also reported that binding migration should not ship while
implicit caller-mutation intrinsics remain; this plan keeps intrinsics
unchanged until AP17.3 and records that ordering in AP17.
