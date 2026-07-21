# Compiler panics and language-limit follow-ups

This document tracks compiler panics and language limits discovered during normal FPAS work. Each
entry records the smallest known source shape, the temporary source-level workaround, and the
intended later resolution. These entries are implementation follow-ups, not current language
behavior.

## Record method in a boolean expression

**Observed while:** implementing `TuiScrollBar.IsAlive`.

**Source shape:** a boolean `return` expression combines registry checks with a bound record method,
for example `IsCurrent and Custom.IsAlive()`.

**Failure:** semantic analysis accepted the source, but compilation panicked in
`Compiler::infer_const_ty` with `expression type missing after semantic analysis` while compiling a
property receiver and binary operation.

**Temporary workaround:** calculate the registry condition in a local boolean, return `false` when
it is not current, then return `Custom.IsAlive()` in a separate statement.

**Later work:** ensure semantic analysis assigns an expression type to a bound record-method call
when it appears as an operand of a binary expression. Add a compiler regression test for the exact
return-expression shape; the compiler must emit bytecode or a diagnostic, never panic.

## Indexed record followed by a bound method

**Observed while:** implementing `TuiScrollBar.Focus`.

**Source shape:** an array-index expression is immediately followed by a bound record-method call,
for example `return ScrollBarViews[Index].Focus()`.

**Failure:** semantic analysis accepted the source, but compilation panicked in
`Compiler::infer_const_ty` with `expression type missing after semantic analysis` while compiling a
property receiver and binary operation.

**Temporary workaround:** first assign the indexed record to a local variable, then call the bound
method through that variable.

**Later work:** preserve the resolved record type through indexed postfix expressions before bound
method compilation. Add a compiler regression test for indexed-record method calls in a return
statement; the compiler must emit bytecode or a diagnostic, never panic.

## Reserved enum variant `End`

**Observed while:** adding `TuiScrollBar` keyboard navigation.

**Source shape:** `TuiKeyKind.End`.

**Failure:** `End` is a `Std.Console.KeyKind` variant, but `end` is also a Pascal keyword. The parser
rejects the member expression before semantic analysis, so FPAS source currently cannot refer to
that key value.

**Temporary workaround:** `TuiScrollBar` supports Up, Down, and Home plus pointer navigation; it
does not claim End-key support.

**Later work:** choose and implement a Pascal-compatible spelling or escape mechanism for public
members whose names collide with keywords. Update the key-event API consistently and add parse,
semantic, and runtime regression coverage.

## Reserved property name `Result`

**Observed while:** adding dialog command completion.

**Source shape:** a record property or parameter named `Result`.

**Failure:** `result` is a Pascal keyword, so the parser rejects the declaration and use before
semantic analysis.

**Temporary workaround:** the dialog API uses `CompletedCommand` for its read-only command value.

**Later work:** decide whether FPAS should provide a Pascal-compatible escape mechanism for public
members that collide with keywords, and add parser coverage for the chosen spelling.

## Entry rule

When development finds a compiler panic or a language limitation that requires a workaround, add an
entry here in the same change. Include the source shape, observed failure or restriction, temporary
workaround, and a concrete later resolution with regression coverage. Do not silently extend the
language or hide the limitation inside a library implementation.

## Transient `TuiCanvas` capability cannot be made opaque

**Context:** restricting canvas operations to an active `OnPaint` callback.

**Source shape:** a public record type with public fields and public static constructors, such as
`TuiCanvas.Create(...)`.

**Restriction:** FPAS has no opaque record representation or private record fields. Application
source can construct or copy a canvas value, so a library-only active-callback token would be
forgeable and could not enforce the documented transient lifetime.

**Temporary workaround:** `TuiCanvas` remains a value-level drawing API. Tui2 does not claim that
the runtime rejects a canvas retained or constructed outside `OnPaint`.

**Later work:** add opaque record handles or per-field visibility with a library-only constructor
path, then attach an application paint-generation token to each canvas and reject stale operations.
Add compile-time visibility coverage plus a runtime stale-canvas regression test.

## Computed record property inside arithmetic can panic the compiler

**Context:** the first `TuiRadioGroup` implementation used `Self.Selected - 1` and
`Self.Selected + 1` when moving the selection.

**Failure:** compilation panics in `fpas_compiler::compiler::binary_op::infer_const_ty` with
`expression type missing after semantic analysis` rather than reporting a diagnostic.

**Temporary workaround:** assign the property to a typed local before arithmetic. The broader
RadioGroup draft was removed until every related source shape is isolated and covered.

**Later work:** make binary-operator lowering reject absent semantic types with a source-located
diagnostic, add regressions for computed record properties in binary expressions, and resume the
RadioGroup implementation from a minimal compiling shape.
