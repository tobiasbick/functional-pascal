# AP09.1: Named arguments for routines and methods

Package: [AP09: Named arguments](README.md)

## Scope

Allow fully named calls `Name := Value` to declared functions, procedures, and
record methods, with written-order evaluation.

## Prerequisites

- AP02 (diagnostic codes).

## Implementation

- Parser and AST: named argument lists; a call is fully positional or fully
  named.
- Sema: map names to the declared parameter names (case-insensitive); reject
  unknown, duplicate, and missing names, mixed calls, and named calls on
  function values; list the expected parameter names in the diagnostic.
- Generic routines infer type arguments from the mapped parameters.
- Compiler: evaluate arguments in written order and pass them in parameter order.
- Confirm that no parameter default values exist in the current language.
- Formatter and signature help.

## Affected areas

- `crates/fpas-parser/src/parser/expr/` (call arguments), AST.
- `crates/fpas-sema/src/check/calls.rs` (split argument checking into a focused
  module when touched; 418 lines at planning time).
- `crates/fpas-compiler/src/lowering/calls.rs`, `fpas-fmt`,
  `crates/fpas-language-service/src/intellisense/signature_help/`.

## Migration

None; positional calls stay valid.

## Documentation

- `docs/specs/grammar.ebnf` (`call_args`), `docs/pascal/language/functions/parameters.md`.

## Verification

- Same-typed argument roles, reordered named arguments with side-effect
  traces, method calls, generic routines, imported routines.
- Rejections: unknown, duplicate, missing, mixed, function values.
- Formatter round trip and signature help.
