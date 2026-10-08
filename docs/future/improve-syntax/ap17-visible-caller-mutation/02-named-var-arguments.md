# AP17.2: Named var arguments

Package: [AP17: Visible caller mutation](README.md)

Status: complete.

## Result

`Increase(Value := var Counter)` is the named reference-argument form.
AP09 mapping uses declared names while AP17 storage, exact-type, aliasing,
lifetime, forwarding and task checks apply in written argument order.
Completed writes survive early `try` returns. Function values remain positional.

Formatter and signature help recognize the form. Parameter labels and referenced
variables navigate and rename separately.

## Regression coverage

Parser, sema, compiler, CLI, editor and FPAS tests cover reordering, traces,
methods, generics, imports/compiled-unit reuse, retained writes, invalid names,
missing/excess markers, read-only storage, aliasing, mixed calls and `go`.
See [var parameters](../../../pascal/language/functions/var-parameters.md).
