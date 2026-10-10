# AP15.1: Migrate events to optional handler fields

Package: [AP15: Remove event declarations](README.md)

Status: complete.

## Result

Repository handler consumers use ordinary `Option of HandlerType` fields with
`None` defaults. Assignments store `Some(...)` or `None` on mutable records.
Conditional calls use `is`; unconditional calls select the callable with
`Unwrap`, preserving failure for an absent handler. Construction, updates,
copies, visibility, and callable capture checks use the existing field rules.

The example prints `no handler`, `clicked 1`, and `no handler`. Its handler is
stored in the record instead of a global accessor slot. Positive fixtures use
the same optional-field model.

`None`, including a parenthesized default, survives exported record interfaces
and transparent aliases. Compiled-unit defaults distinguish scalar values from
`None` without broadening scalar constant folding. Parenthesized literals
retain the expected target type during lowering, including `(None)` when a
handler is cleared.

## Ownership

- `examples/pascal/record-methods/events.fpas` owns the runnable example.
- Parser, Sema, compiler, and CLI `handler_fields.rs` modules own focused
  handler tests. Compiler `aggregates/try_expressions.rs` covers evaluation
  order and failure propagation. Interface alias and editor/formatter fixtures
  use optional fields.
- `crates/fpas-unit/src/interface/types.rs` defines persisted field defaults;
  `crates/fpas-sema/src/interface/{export.rs,install.rs}` exports and restores them.
- `crates/fpas-compiler/src/lowering/aggregates/literals.rs` forwards expected types
  through parentheses.

## Regression coverage

Tests cover defaults, assignment and clearing, conditional calls, bound methods
and record copies, immutable-field rejection, construction and updates, `try`
propagation, aliases, formatting and semantic tokens. The CLI test checks a
consumer constructor, owner-unit calls, and dependency sidecar reuse after
rebuilding the consumer. Interface codec tests preserve both scalar and `None`
defaults.

Verification includes Rust formatting, workspace build/tests, FPAS formatting
and the complete FPAS suite, output comparison for the example, execution of
the handbook example, and documentation links.

## Current documentation

- [Optional handlers](../../../pascal/language/functions/first-class.md#optional-handlers).
- [Default field values](../../../pascal/language/types/records.md#default-field-values).

## Related work

[AP15.2](02-remove-event-declarations.md) owns ordinary identifier resolution
and the field/method-only grammar, metadata, and editor model. It adds no
migration diagnostics.
