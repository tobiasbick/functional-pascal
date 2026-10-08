# AP17: Visible caller mutation

Status: complete. Effort: large. Completion is tracked in the
[central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

Caller mutation follows explicit parameter modes, call-site marking for
written arguments, and the recorded implicit-receiver exception. Aliasing
between writable arguments is rejected, and local reassignment cannot become
a caller-visible change during migration.

Current behavior: value parameters are read-only, `var` parameters reference
writable caller storage, and explicitly written reference arguments carry a
`var` marker. Native Push/Pop use the same storage checks with an unmarked
implicit receiver, including fully named `Push(Value := V)` calls.

## Decisions

- Parameters are read-only by default. `var` in the signature allows the
  routine to change the caller's variable, and every explicitly written
  argument for a `var` parameter is marked with `var`:
  `Increase(var Counter)`. Named form: `Increase(Value := var Counter)`.
- A `var` argument must be a `var` binding or a field or element of one;
  constants, `const` bindings, and temporaries are invalid.
- Two `var` arguments of one call must not share a root variable. This also
  rejects `Swap(var A[I], var A[J])`; use a routine such as `SwapAt(var A, I, J)`.
- Intrinsics that change a caller variable use the same `var` safety rules
  and explicit marking for written arguments. The implicit receiver of a
  dot call is the agreed exception: native operations such as `Items.Push(V)`
  and `Items.Pop()` use ordinary dot syntax without a receiver `var` marker
  or additional parentheses. Their catalog entries require a writable
  receiver, checked as a `var` binding, a field or element of one, or a
  forwarded `var` parameter; `const` bindings and temporaries are rejected.
  Include the receiver in aliasing, lifetime, `go`, evaluation, and failure
  checks. Record methods retain their existing semantics. The native catalog
  records this receiver mode.
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

## Dependencies

- AP09 (named-argument mapping).
- AP13 (routine and expression closers).
- AP16 (binding mutability).

[AP06](../ap06-dot-call-targets/README.md) supplies native catalog resolution
and signatures using the shared mutation checks. AP25 and AP26 depend on AP17.

## Work packages

- [x] [AP17.1: var parameters and arguments](01-var-parameters.md)
- [x] [AP17.2: Named var arguments](02-named-var-arguments.md)
- [x] [AP17.3: Caller-mutating intrinsics](03-caller-mutating-intrinsics.md)

## Acceptance

Written `var` arguments are explicitly marked. Caller-mutating intrinsics
apply the shared safety checks with ordinary dot syntax for implicit
receivers. Aliasing between writable arguments is rejected. Native resolution
and named native calls use these same mutation checks. Local parameter copies
and caller reference parameters retain their distinct storage behavior.
