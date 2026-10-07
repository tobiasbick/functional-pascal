# AP17: Visible caller mutation

Status: agreed direction. Effort: large. Completion is tracked in the
[central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

Caller mutation follows explicit parameter modes, call-site marking for
written arguments, and the recorded implicit-receiver exception. Aliasing
between writable arguments is rejected, and local reassignment cannot become
a caller-visible change during migration.

Current behavior: ordinary parameters are read-only values, AP17.1 implements
`var` parameters and positional arguments, and AP17.2 implements named `var`
arguments. `Push`/`Pop` retain
their simple writable-array binding rule until AP17.3, so they do not accept
a `var` parameter yet.

## Decisions

- Parameters are read-only by default. `var` in the signature allows the
  routine to change the caller's variable, and every explicitly written
  argument for a `var` parameter is marked with `var`:
  `Increase(var Counter)`. Named form: `Increase(Value := var Counter)`.
- A `var` argument must be a `var` binding or a field or element of one;
  constants, `const` bindings, and temporaries are invalid.
- Two `var` arguments of one call must not share a root variable. This also
  rejects `Swap(var A[I], var A[J])`; use a routine such as `SwapAt(var A, I, J)`.
- `mutable` parameters are removed by
  [AP16.3](../ap16-immutable-and-mutable-bindings/03-keyword-switch.md), before
  AP17.1 introduces reference parameters. The migration uses fresh local
  `var` copies for reassignment, field/element writes, mutating intrinsics,
  and captures (including read-only captures). Only resolved parameter
  references are renamed, preserving shadowing and member names. Local
  changes never become caller mutation; shared capture cells stay task-bound.
- Intrinsics that change a caller variable use the same `var` safety rules
  and explicit marking for written arguments. The implicit receiver of a
  dot call is the agreed exception: native operations such as `Items.Push(V)`
  and `Items.Pop()` use ordinary dot syntax without a receiver `var` marker
  or additional parentheses. Their catalog entries require a writable
  receiver, checked as a `var` binding, a field or element of one, or a
  forwarded `var` parameter; `const` bindings and temporaries are rejected.
  Include the receiver in aliasing, lifetime, `go`, evaluation, and failure
  checks. Record methods retain their existing semantics. The user requested
  that this exception be discussed again before AP06.3/AP17.3; see
  [Follow-up discussion](#follow-up-discussion).
- A closure cannot capture a `var` parameter, and a `var` argument cannot be
  passed to a `go` call; the reference must not outlive the call.
- A `var` parameter may be forwarded as a `var` argument to another routine,
  written `var Value`.
- Function types may declare `var` parameters, for example
  `function(var Value: integer): integer`. Parameter modes are part of
  function-type compatibility: a read-only parameter and a `var` parameter
  cannot be substituted for each other. Calls through function values mark
  each `var` argument explicitly and follow the same argument, aliasing,
  forwarding, capture, and `go` restrictions as calls to declared routines.
  Function values remain positionally called, as agreed in AP09.
- The root and indices of each `var` argument are evaluated once, in written
  left-to-right order with the other arguments. Assignments through `var`
  parameters update the caller's variable as they execute. Writes completed
  before a panic or `try` exit remain visible; unwinding does not roll them
  back. This applies to whole-variable, field, and element arguments and to
  calls through function values.

```pascal
procedure Increase(var Value: integer);
begin
  Value := Value + 1;
end procedure;

Increase(var Counter);
Increase(Value := var Counter);
Increase(var P.X);
Swap(var A[I], var A[J]);   // error: both arguments refer to 'A'

var Items: array of integer := [1, 2];
Items.Push(3);
const Last: integer := Items.Pop();
```

## Follow-up discussion

The implicit-receiver exception is the current agreed decision. The user
explicitly requested that we discuss it again before implementing AP06.3 or
AP17.3. Revisit it in
[AP06.1](../ap06-dot-call-targets/01-catalog-and-rules-decision.md) and record
the outcome in both package READMEs; the
[AP06 follow-up](../ap06-dot-call-targets/README.md#follow-up-discussion)
tracks the shared review. This does not block AP16 or AP17.1/AP17.2.

## Dependencies

- AP09 (named arguments, for the named `var` form in AP17.2).
- AP13 (routine and expression closers; complete on the working branch).
- AP16 (the keyword switch AP16.3).

AP25 and AP26 depend on this package.

## Order

AP17.1 delivers positional `var` parameters with their safety rules. AP17.2
adds the named form. AP17.3 applies the shared `var` safety checks to
caller-mutating intrinsics, uses ordinary dot syntax for implicit receivers,
and removes their special simple-variable rule after the requested review.

## Work packages

- [x] [AP17.1: var parameters and arguments](01-var-parameters.md)
- [x] [AP17.2: Named var arguments](02-named-var-arguments.md)
- [ ] [AP17.3: Caller-mutating intrinsics](03-caller-mutating-intrinsics.md)

## Acceptance

Written `var` arguments are explicitly marked. Native mutating receivers use
ordinary dot syntax with catalog-defined writable-receiver checks after the
user-requested follow-up discussion. Aliasing between writable arguments is
rejected, and migration preserves local versus caller-visible reassignment.

## Reference

The reference branch `codex/syntax-changes` found that reference modes must be
carried through AST, sema, unit interfaces, IR, bytecode, and VM; that
`std_registry/builtins/array/mutation.rs` requires simple mutable variables for
`Push`/`Pop`; and that `lowering/calls/arrays.rs` has local and cell/global
read-modify-write paths. Debugger writes must follow the same restrictions.
