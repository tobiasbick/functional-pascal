# AP17.1: var parameters and arguments

Package: [AP17: Visible caller mutation](README.md)

Status: complete.

## Result

`var` parameters reference writable caller storage; explicitly written
arguments require a `var` marker. Whole variables, record fields and array
elements are valid, including public imported variables and forwarded reference
parameters. Constants, temporaries, dictionary entries and string characters
are invalid (FP3028). Two writable arguments cannot share a root variable.

Roots and indices are fixed once in written argument order. Writes take effect
as executed and remain visible after panic or early `try` return. Function types
carry parameter modes and reject read-only/reference substitution. Function
values use positional calls with explicit markers; record-method `Self` is
not a `var` parameter.

Closures cannot capture reference parameters and `go` cannot take references.
A closure may pass a captured local `var`. Named nested routines can access an
enclosing reference parameter when called directly; using them as values or
with `go` is rejected (FP3030). Aliasing checks apply to writable arguments of
one call, rather than every possible access through global names.

Runtime references identify a cell or global plus field/element steps. Modes
survive unit interfaces, IR, objects and bytecode. Debugger reference writes/calls
are tracked in [compiler follow-ups](../../compiler-panic-followups.md).

## Regression coverage

Tests cover storage forms, imports, forwarding, generic exact types, function
modes, captures, aliasing, task restrictions, traces and retained writes.
See [var parameters](../../../pascal/language/functions/var-parameters.md).
