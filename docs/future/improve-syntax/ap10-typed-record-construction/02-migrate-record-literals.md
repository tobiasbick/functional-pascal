# AP10.2: Migrate record literals

Package: [AP10: Typed record construction](README.md)

Status: Complete.

## Scope

Positive record-construction consumers use `TypeName(Field := Value, ...)`.
[AP10.3](03-remove-record-literals.md) removed the literal form after this
migration. Current expression parsing rejects `record ... end` with FP2017;
semantic analysis and lowering use typed construction.

## Prerequisites

- AP10.1 (typed construction).

## Implementation

- Constructors name the expected record type, with a visible alias or unit
  qualification where needed. Facade aliases such as `TuiKeyEvent` respect
  the existing imports.
- Field values and defaults are preserved. Migrated arguments follow field
  declaration order to preserve literal evaluation order. Interleaved defaults
  with side effects use intermediate bindings before construction.
- Empty construction uses `TypeName()`; nested records, arrays, enum payloads,
  `Result`/`Option` values, and generic calls name their record types explicitly.

## Affected areas

- `.fpas` sources under `lib/`, `apps/`, `examples/`, and `tests/`.
- Rust-embedded fixtures, parser/lexer assertions, formatter goldens, editor
  fixtures and generated test sources, documentation examples, and the FPAS
  authoring skill.
- Debugger expression fixtures use existing positional factories where named
  construction is unavailable in debugger evaluation. The restriction and
  workaround are tracked in [compiler-panic-followups.md](../../compiler-panic-followups.md).
- The JSONL breakpoint contract matches the migrated fixture's executable line.

## Migration

The conversion tool is temporary and is not part of the repository.

Representative coverage:

- `crates/fpas-compiler/src/tests/aggregates/record_construction.rs`
- `crates/fpas-parser/src/tests/expr/aggregates.rs`
- `crates/fpas-lexer/src/tests/integration/declarations.rs`
- `crates/fpas-fmt/tests/golden_output.rs`
- `crates/fpas-fmt/tests/expression_closer_regressions.rs`
- `crates/fpas-debug/tests/jsonl_contract.rs`
- `tests/stdlib/records/typed_construction_test.fpas`

## Documentation

Record, record-method, default, declaration-order, console, TUI, and formatter
examples use typed construction. The authoring skill describes named fields
and the constructor's evaluation order.

## Verification

- AST inspection finds no record literals in `.fpas` files or positive embedded
  fixtures. Embedded legacy forms remain only in negative tests.
- The full FPAS suite and example/app checks pass. Formatting, formatter goldens,
  editor compilation, grammar/contracts, generated debugger-program type checks,
  and plan links are checked.
- Workspace coverage for migrated consumers passes. The full cloud run retains
  the two existing VM socket-timeout failures; the existing hanging LSP watcher
  test is excluded.
