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

## Result

Delivered as planned, with these agreed scope details:

- Declared `Std.*` routines accept named arguments with the parameter names of
  their generated signatures. Polymorphic standard-library operations (for
  example `Abs`, `Push`) and variadic routines (`WriteLn`, `Format`) take
  positional arguments only (FP3026).
- Receiver calls `Value.Name(...)` and calls through function values or
  callable record members take positional arguments only (FP3026). Native
  receiver operations are revisited with AP06.
- Enum variant constructors rejected named arguments in this work package;
  [AP09.2](02-named-variant-arguments.md) added them.
- Mixed calls are a parse error (FP2016); unknown, duplicate, and missing
  names are FP3025.
- Editor support: signature help selects the named parameter, and navigation
  and rename resolve a `Name :=` label to the callee's parameter. Debugger
  evaluation accepts positional call arguments only.
