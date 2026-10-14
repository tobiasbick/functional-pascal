# Compiler panics and language-limit follow-ups

This document is the intake list for compiler panics and language limits discovered during normal
FPAS work. Resolved behavior belongs in the implementation, regression suite, and current
documentation; broader language proposals belong in their own future notes.

## Entry rule

When development finds a compiler panic or a language limitation that requires a workaround, add an
entry here in the same change. Include the source shape, observed failure or restriction, temporary
workaround, and a concrete later resolution with regression coverage. Do not silently extend the
language or hide the limitation inside a library implementation.

## Open entries

### Chained method call inside an array index loses expression typing

During AP41 migration, the current compiler reports FP9001 for this source:

```pascal
program IndexCallRepro;
type ImportInfo = record Owner: string; end record;
begin
  const Import := ImportInfo(Owner := 'Std.Console');
  const Alias := Import.Owner.Split('.')[Import.Owner.Split('.').Length() - 1];
  discard Alias;
end.
```

The index subtraction has no recorded expression type after semantic analysis.
The temporary workaround is `const Parts := Import.Owner.Split('.');` followed
by `Parts[Parts.Length() - 1]`. A later compiler fix must retain expression typing
through chained native calls in index expressions, with a compile/run regression
for this source and the expected final segment `Console`.
