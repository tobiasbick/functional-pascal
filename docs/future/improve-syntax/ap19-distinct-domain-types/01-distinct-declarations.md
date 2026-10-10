# AP19.1: Distinct type declarations and conversions

Package: [AP19: Distinct domain types](README.md)

Status: complete.

## Result

`distinct` is a reserved keyword. `type UserId = distinct integer;` declares a
nominal type over `integer`, `real`, `string`, or `boolean`, written directly or
through an alias. Records, enums, containers, callables, generic parameters, and
other distinct types are rejected as underlying types with a diagnostic that
names the allowed types. `type X = type Y` and `distinct` outside a type
declaration are parse errors with the canonical spelling.

`UserId(Value)` wraps a value of the exact underlying type (or keeps a
`UserId`); `integer(Value)` and an alias such as `Raw(Value)` unwrap a distinct
value whose underlying type is exactly that type. Built-in type name calls do
nothing else. Direct conversion between two distinct types is rejected with the
`OrderId(integer(U))` hint. Implicit conversions in either direction are type
mismatches whose hints name the explicit conversion.

No operators are inherited: arithmetic, concatenation, equality, ordering,
`in`, and logical operators report the distinct type with an unwrapping hint.
Built-in dot operations, variadic standard-library arguments such as `WriteLn`,
and the `Comparable`, `Numeric`, and `Printable` constraints do not accept
distinct values. Unconstrained generics, containers, record fields, and record
defaults do.

A conversion of a compile-time constant is a compile-time constant and folds to
the underlying value. Distinct identities, owner units, and underlying types
are persisted in compiled-unit interfaces (`.fpascu` format version 12).
Runtime values use the underlying IR type; conversions lower to their argument.

Debugger evaluation accepts the same conversion spellings. Each compiled source
records the distinct type names it declares or imports (plain imports by short
and qualified name, aliased imports by alias) with their scalar debug types;
the table survives object linking and the `.fpascp` format (program and
bytecode version 18). The debugger checks only the runtime scalar type, because
distinct values share their underlying representation.

## Ownership

- `crates/fpas-lexer/src/token/{keywords.rs,kind.rs}` reserve `distinct`;
  `crates/fpas-parser/src/parser/decl/data/type_defs.rs` parses
  `TypeBody::Distinct` and diagnoses `type X = type Y`;
  `crates/fpas-parser/src/parser/decl/type_expr.rs` diagnoses misplaced
  `distinct`.
- `crates/fpas-sema/src/types/mod.rs` defines `Ty::Distinct` and its nominal
  compatibility; `check/decl/types/distinct.rs` validates declarations;
  `check/expr/distinct_conversion.rs` checks conversions, implicit-unwrap
  rejections, and conversion hints; `check/expr/operators.rs` rejects
  operators; `check/decl/consts/{classification.rs,scalar.rs}` classify and
  fold constant conversions.
- `crates/fpas-unit/src/interface/types.rs` defines `DistinctType`;
  `crates/fpas-sema/src/interface/conversion/` converts it.
- `crates/fpas-compiler/src/lowering/{types.rs,calls.rs}` erase distinct types
  and lower conversions.
- The formatter, project source map, language-service symbol extraction, and the
  VS Code TextMate grammar handle the new type body and keyword.
- Debugger names: `crates/fpas-compiler/src/lowering/types/debug_distinct.rs`
  collects them, `fpas-bytecode` (`metadata/distinct_types.rs`, validation),
  `fpas-unit` objects, `fpas-linker/src/emit/distinct_types.rs`, and
  `fpas-program/src/format/executable/distinct_types.rs` carry them, and
  `crates/fpas-vm/src/vm/debug/calls/distinct_conversion.rs` evaluates them.

## Regression coverage

- Lexer and parser keyword inventories, reserved-name hints, distinct bodies,
  and both parse diagnostics.
- Sema declaration tests (underlying types, aliases, declaration order, constant
  classification, record defaults) and conversion tests (wrap/unwrap, rejected
  implicit conversions, swapped domain arguments, exact underlying types,
  built-in type name calls, argument shape, arithmetic, comparisons,
  membership, `WriteLn`, dot operations, constraints, `go`).
- Interface tests for exported identity, owner unit, constants, and equally
  named distinct types from different units.
- A build test across units with plain and aliased imports and sidecar reuse.
- Formatter round trip and golden output; grammar verification.
- `tests/runner/distinct_types_test.fpas` runs the conversions end to end.
- Debugger tests evaluate wrap/unwrap conversions, argument and type errors, and
  plain, aliased, and missing imports after linking; bytecode tests validate the
  table; the program format fixture round-trips it.

## Current documentation

- [Distinct types](../../../pascal/language/types/distinct-types.md).
- [Debugger](../../../pascal/tools/debugger.md) conversions and
  [compiled program](../../../pascal/program-structure/compiled-programs.md)
  format versions.
- [Keywords](../../../pascal/getting-started/keywords.md) and
  [`grammar.ebnf`](../../../specs/grammar.ebnf) (`distinct_type`,
  `distinct_conversion`).

## Related work

[AP19.2](02-distinct-comparisons.md) owns inherited comparisons, membership,
`Comparable`, dictionary keys, and `case` selectors and labels.
