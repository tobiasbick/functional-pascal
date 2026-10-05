# AP17: Visible caller mutation

Status: agreed direction. Effort: large. Completion is tracked in the
[central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

Caller mutation is visible at declaration and invocation, aliasing between
`var` arguments is rejected, and local reassignment cannot become a
caller-visible change during migration.

## Decisions

- Parameters are read-only by default. `var` in the signature allows the
  routine to change the caller's variable, and every call marks the argument
  with `var`: `Increase(var Counter)`. Named form: `Increase(Value := var Counter)`.
- A `var` argument must be a `var` binding or a field or element of one;
  constants, `const` bindings, and temporaries are invalid.
- Two `var` arguments of one call must not share a root variable. This also
  rejects `Swap(var A[I], var A[J])`; use a routine such as `SwapAt(var A, I, J)`.
- `mutable` parameters are removed. Where a routine reassigned a `mutable`
  parameter locally, the migration introduces a local `var` copy; it never
  turns local reassignment into caller mutation. This removal is delivered by
  the shared keyword switch in
  [AP16.3](../ap16-immutable-and-mutable-bindings/03-keyword-switch.md).
- Intrinsics that change a caller variable follow the same rule, for example
  `Push(var Items, 3)`.
- A closure cannot capture a `var` parameter, and a `var` argument cannot be
  passed to a `go` call; the reference must not outlive the call.
- A `var` parameter may be forwarded as a `var` argument to another routine,
  written `var Value`.

```pascal
procedure Increase(var Value: integer);
begin
  Value := Value + 1;
end procedure;

Increase(var Counter);
Increase(Value := var Counter);
Increase(var P.X);
Swap(var A[I], var A[J]);   // error: both arguments refer to 'A'
```

## Open decisions

- Whether function types may declare `var` parameters (for example
  `function(var Value: integer): integer`), so that function values can mutate
  a caller variable. Decide before AP17.1.
- Whether caller-mutating operations keep a dot form (for example
  `Items.Push(3)`) and how `var` is marked there is decided in
  [AP06.1](../ap06-dot-call-targets/01-catalog-and-rules-decision.md).

## Dependencies

- AP09 (named arguments, for the named `var` form in AP17.2).
- AP13 (recorded package dependency; recheck at AP17.1 whether a block-syntax
  work package is actually required).
- AP16 (the keyword switch AP16.3).

AP25 and AP26 depend on this package.

## Order

AP17.1 delivers positional `var` parameters with their safety rules. AP17.2
adds the named form. AP17.3 moves caller-mutating intrinsics to explicit `var`
arguments and removes their special rule.

## Work packages

- [ ] [AP17.1: var parameters and arguments](01-var-parameters.md)
- [ ] [AP17.2: Named var arguments](02-named-var-arguments.md)
- [ ] [AP17.3: Caller-mutating intrinsics](03-caller-mutating-intrinsics.md)

## Acceptance

Caller mutation is visible at declaration and invocation, aliasing between
`var` arguments is rejected, and local reassignment cannot become a
caller-visible change during migration.

## Reference

The reference branch `codex/syntax-changes` found that reference modes must be
carried through AST, sema, unit interfaces, IR, bytecode, and VM; that
`std_registry/builtins/array/mutation.rs` requires simple mutable variables for
`Push`/`Pop`; and that `lowering/calls/arrays.rs` has local and cell/global
read-modify-write paths. Debugger writes must follow the same restrictions.
