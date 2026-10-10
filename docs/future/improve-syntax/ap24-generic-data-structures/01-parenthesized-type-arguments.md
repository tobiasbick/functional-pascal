# AP24.1: Parenthesized type argument lists

Package: [AP24: Generic data structures](README.md)

Status: complete.

## Scope

Built-in `Result` types use exactly two parenthesized arguments:
`Result of (T, E)`. Single-argument applications and `dict of K to V` keep
their existing forms. User-defined generic records and enums remain with
AP24.2 and AP24.3.

## Prerequisites

None.

## Implementation

- The shared parser accepts nested parenthesized `Result` applications and
  rejects unparenthesized or angle-bracket applications with an `of` correction.
  Recovery preserves following declarations.
- The formatter, semantic signatures, and debugger type names emit the same
  parenthesized form. Intrinsic editor declarations use those signatures.
- The intrinsic declaration exporter ends each routine parameter list at its
  matching delimiter, independently of parentheses in the return type.

## Affected areas

- `crates/fpas-parser/src/parser/decl/type_expr.rs` and `type_arguments.rs`.
- `crates/fpas-fmt/src/emit/types.rs`, `crates/fpas-sema/src/types/mod.rs`,
  and compiler/VM debugger type rendering.
- The intrinsic declaration exporter in
  `crates/fpas-sema/examples/export_intrinsic_std_api.rs`, its documentation
  renderer in `export_intrinsic_std_api/documentation.rs`, and generated
  declarations under `lib/api/`.

## Migration

Repository FPAS sources, embedded Rust and editor fixtures, generated intrinsic
declarations, diagnostics, and current/future documentation use the canonical
form. Rejected forms remain only in intentional negative tests.

## Documentation

- `docs/specs/grammar.ebnf` (`type_expr`),
  `docs/pascal/language/types/result-option-types.md`, `generics.md`,
  affected `Std` pages.

## Verification

- Parser type-fragment and application tests cover unchanged single-argument
  types, nested applications, source spans, wrong arity, missing delimiters,
  both rejected forms, and declaration recovery.
- Formatter corpus round trips and the handbook CLI checks cover canonical
  output; the formatter CLI rejects obsolete forms without changing files.
- `tests/stdlib/result/parenthesized_type_arguments_test.fpas` covers nested
  success/error values, aliases, callable types, and dictionary value types.
- Intrinsic exporter tests cover parameterless routines and callbacks with
  parenthesized return types without generating spurious parameter comments.
- Workspace and FPAS suites, corpus formatting, example/app checks, editor
  contracts, and the real Extension Host verify the migrated consumers.
