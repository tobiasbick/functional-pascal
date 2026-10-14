# AP43.2: Error APIs and guidance

Package: [AP43](README.md). Status: complete; implemented, verified, and independently reviewed.

## Implemented contract

The [AP43.1 contract](01-error-conventions.md) is enforced across runtime APIs,
checking, lowering, consumers, diagnostics, generated declarations, and guidance.

- Std.Parse Int/Real/Bool/HexInt take Text: string and return
  Result of (target type, string). The retired Try-prefixed names and four
  aborting Conv parsers are rejected. Parser-plus-Unwrap is deliberate abort.
  Existing general/hex interpretations, separators, signed bounds, Boolean
  trimming/case, finite reals, signed zero, subnormals, and round-trips are retained.
- Native Option/Result expose Map, AndThen, OrElse, and Unwrap. Nonbinding is
  tests variants; exhaustive case binds payloads, handles locally, and selects
  plain defaults. Native predicates and UnwrapOr are removed.
- OrElse callback argument expressions remain eager, while the callback body
  runs only on Error/None. Case fallback work runs only in the selected arm.
  Migrated callers preserve receiver/argument order, original occurrences,
  initialization and control-flow locality, eager default effects, and aborts.
- Result try propagation requires compatible error types, including identical
  generic parameters inside aggregates and preserved nominal identity. Generic
  parameters are not wildcards at this propagation boundary. Option propagates
  into Option; success types may differ. Cross-family, incompatible, ordinary
  value, procedure, and program-body uses are rejected. Panic and runtime failures
  remain uncaught. Existing return, cleanup, and postcondition behavior is retained.
- All four parsers are callable values with their checked Text signatures,
  including imported and reused compiled-unit consumers. Source-local aliases,
  complete named arguments, inference, and same-name user routines remain valid.
- Bytecode version 23 and compiled-unit/program envelope version 20 reject stale
  executable contracts and let source-owned units rebuild automatically.

## Implementation ownership

- `crates/fpas-numeric/src/text.rs` and `crates/fpas-std/src/parse.rs` own the
  existing text grammar and Result runtime. Conv retains typed formatting and
  numeric conversion; `result_option.rs` owns explicit unwrap failure.
- Bytecode intrinsic enums, Std symbol definitions, loaded/native sema registries,
  and the compiler catalog expose the selected public surface. The normal
  intrinsic API generator owns `lib/api/Std/` editor declarations.
- `check/name_resolution/parser_migration.rs`, native migration hints, and fluent
  lookup diagnose removed spellings using actual import aliases and actionable
  parser/is/case repairs. Unwrap diagnostics teach case for expected failures.
- `check/expr/mod.rs` uses strict recursive propagation compatibility from
  `types/compatibility.rs`; generic call inference keeps its separate rules.
- `lowering/closures/parser_values.rs` registers parser references. Shared
  `intrinsic_routines.rs` reuses intrinsic task-entry lowering for parser wrappers.
  Task registration remains in `intrinsic_tasks.rs`.
- Std source units, applications, examples, FPAS/Rust fixtures, benchmarks,
  authoring guidance, current handbooks, and editor queries use the canonical forms.
  AP06/AP21/AP33 retain completed status with AP43 replacement notices.

## Regression ownership

- `fpas-sema` error-convention tests own parser signatures/Text labels, retired-name
  repairs, user-name preservation, and the complete static try matrix, including
  incompatible generic and nominal errors.
- `tests/stdlib/parse/canonical_parsers_test.fpas` owns valid/invalid parser input,
  numeric and Boolean boundaries, error payloads, typed callable parsers, and
  higher-order invocation. Existing numeric fixtures retain AP33 boundary coverage.
- `tests/runner/canonical_error_handling_test.fpas` owns observable receiver/default
  order, eager argument creation versus conditional callbacks, local lazy defaults,
  loop/short-circuit paths, and propagation early exits.
- Compiler error-convention tests own parser-plus-Unwrap failure, uncaught panic,
  and panicking eager defaults even with successful constant receivers.
- CLI project error-convention tests own imported parser callable values and try
  signatures across cold/reused unit interfaces and a rebuilt consumer.
- Language-service error-convention tests own the retained wrapper completion
  surface and aliased parser completion/named signature help. Existing task,
  postcondition, format-compatibility, numeric, and consumer suites retain coverage.

## Open follow-ups

No selected AP43 decisions or implementation work remain open. Independent
baseline TCP timing-test failures are recorded in
[AP41's verification notes](../ap41-canonical-qualified-names/02-qualified-name-rules.md).
The unchanged DAP step fixture can consume its event in the immediate handle
response before inspecting wait output; this timing race is outside AP43.
