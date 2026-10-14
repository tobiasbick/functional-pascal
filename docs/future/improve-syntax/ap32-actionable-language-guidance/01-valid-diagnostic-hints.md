# AP32.1: Valid diagnostic hints

Package: [AP32](README.md). Status: complete.

## Result

FP3004 lists qualified candidates for ambiguous imported calls. Both expression
and statement calls use the shared ambiguity hint; free routines are selected
by name, not by receiver type. Migration hints for removed native free
operations still recommend the implemented native receiver forms.

## Implementation ownership

- `crates/fpas-sema/src/check/name_resolution/std_names.rs` owns the qualified
  candidate hint shared with other ambiguous-name diagnostics.
- `crates/fpas-sema/src/check/expr/calls/mod.rs` and `check/stmt/calls.rs` use
  that hint at their respective call-resolution boundaries.
- `crates/fpas-sema/src/std_registry/native/mod.rs` owns the
  separate removed-native-operation guidance.

## Migration and documentation

No language migration or diagnostic-code change. The
[diagnostic reference](../../../pascal/tools/diagnostics.md#required-qualification)
shows a real `ReadText` conflict and its valid qualified correction.

## Regression coverage

`crates/fpas-sema/src/tests/integration/short_names.rs` checks a real
`Std.Console`/`Std.Fs` `ReadText` ambiguity, FP3004, candidate qualification,
and a valid qualified call in the original context. It checks the same help
for statement calls and retains executable native repairs for `Unwrap`,
`Length`, and `Contains`.

## Open follow-ups

None within this work package's scope.
