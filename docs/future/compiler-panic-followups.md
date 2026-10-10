# Compiler panics and language-limit follow-ups

This document is the intake list for compiler panics and language limits discovered during normal
FPAS work. Resolved behavior belongs in the implementation, regression suite, and current
documentation; broader language proposals belong in their own future notes.

## Entry rule

When development finds a compiler panic or a language limitation that requires a workaround, add an
entry here in the same change. Include the source shape, observed failure or restriction, temporary
workaround, and a concrete later resolution with regression coverage. Do not silently extend the
language or hide the limitation inside a library implementation.

## Scheduling

The debugger repairs are complete and included in the
[reviewed scope](improve-syntax/review/reviewed-scope.md).
The [syntax review plan](improve-syntax/review/README.md) is an empty placeholder
for the next review.

## Open entries

No entries are currently open.

## Completed debugger repairs

### Named calls and typed record construction

- [x] Declared routines, record methods, and enum constructors map fully named
  arguments to retained parameter or field names, case-insensitively. Argument
  expressions run in written order before reordering. Function values and
  debugger intrinsics use positional arguments.
- [x] Record construction selects the exact visible nominal type, including
  source type aliases and `uses` aliases. Supplied field expressions precede
  omitted defaults, which run in declaration order in the declaration scope.
- [x] Required fields, duplicate and unknown names, value types, and private-field
  construction restrictions are checked before default bodies execute.
- [x] Object linking and compiled-program serialization preserve constructor
  visibility, source-local aliases, and default routines. Available units without
  a source import do not expose constructor names.
- [x] Rust integration tests cover mapping, evaluation order, nominal identity,
  imports, defaults, visibility, effects, and portable metadata. DAP coverage
  exercises named calls and typed construction.

### Reference parameters in debugger calls and assignments

- [x] Portable debug types retain reference parameter modes, including inside
  first-class function signatures. Calls and function-value assignments compare
  these modes.
- [x] Explicit `var` arguments resolve initialized writable bindings, stored
  record fields, and array elements with exact types and distinct roots. Invalid
  argument modes or paths fail before the callee body executes.
- [x] Reference calls share the detached evaluation sandbox; writes do not alter
  the stopped program. Later arguments resolve paths against the sandbox's
  current storage.
- [x] `setVariable` and `setExpression` write through stopped reference parameters
  to caller storage. Failed writes preserve values and handles; successful writes
  commit atomically and refresh handles.
- [x] Rust and real DAP regressions cover accepted reference calls, rejected call
  shapes, detached writes, and live write-through behavior.
