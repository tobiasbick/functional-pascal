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

### Debugger evaluation accepts positional call arguments only

- **Source shape:** a debugger `evaluate` expression with named arguments, for
  example `Sub(Left := 1, Right := 2)`, `Shape.Rect(Width := 1.0, Height := 2.0)`,
  or typed record construction `Point(X := 1, Y := 2)`.
- **Restriction:** `crates/fpas-debug/src/evaluation/validate.rs` rejects
  `Expr::NamedArgument`; the debugger resolves calls without semantic analysis
  and therefore has no parameter-name mapping.
- **Workaround:** pass routine or variant arguments by position in declaration
  order. For records, evaluate an existing value or call a positional factory
  such as `Point.Create(1, 2)`; record construction itself is named only.
- **Resolution:** map names to the callee's parameter or variant-field names
  from the debug metadata, evaluate in written order, and add debugger
  evaluation tests for routines, methods, variant constructors, and typed record
  construction with field defaults and visibility checks.

### Debugger cannot write or pass `var` parameters

- **Source shape:** a debugger assignment to a `var` parameter, a debugger
  `evaluate` expression with a `var` argument, or a debugger call of a routine
  that declares a `var` parameter.
- **Restriction:** inspection shows a `var` parameter as the caller's current
  value, but the debugger treats the binding as read-only;
  `crates/fpas-debug/src/evaluation/validate.rs` rejects `Expr::VarArgument`,
  and a debugger call of a `var`-parameter routine fails inside the callee
  because no reference is passed. Portable debug types describe reference
  types as `Dynamic`, so debugger function-value assignment cannot compare
  parameter modes.
- **Workaround:** assign the caller's variable in the caller frame, and call
  routines with `var` parameters from program code.
- **Resolution:** carry parameter modes in portable debug types, write through
  the reference with the same checks as program code, reject `var`-parameter
  callees before invocation with a clear message, and add debugger tests for
  each case.
