# AP14.2: Remove property declarations

Package: [AP14: Remove computed properties](README.md)

Status: complete.

## Result

`property` is an ordinary identifier. In a record member list, `property Name`
reports FP2018 (`PARSE_REMOVED_PROPERTY`) and skips the invalid declaration
while preserving the other members. The hint names the written accessor
methods, for example `Value.GetZoom()` and `Value.SetBase(NewValue)`; it does
not generate or invent accessors. The contextual `read` and `write` clauses
in this diagnostic are ordinary identifier tokens, as specified by
[AP15](../ap15-remove-event-declarations/README.md).

Records contain stored fields, instance methods, and static routines. The
parser, semantic checker, compiler, runtime formats, unit interfaces, formatter,
debugger, editor, and grammar use this model without property metadata.

## Ownership

- `crates/fpas-parser/src/parser/decl/data/removed_properties.rs` owns FP2018
  detection and recovery.
- `crates/fpas-lexer/src/token/keywords.rs` owns ordinary identifier scanning.
- The record field and method implementations own direct reads, writes, and
  explicit accessor calls.

## Regression coverage

`crates/fpas-parser/src/tests/errors/removed_properties.rs` and
`crates/fpas-cli/src/main_tests/projects/removed_properties.rs` cover
replacement hints, accessor names, recovery, visibility, and ordinary
`property` declarations. Lexer keyword tests cover identifier scanning.
`tests/runner/static_record_procedure_test.fpas` verifies a record without
stored fields or instance methods.

## Current documentation

- [Record methods](../../../pascal/language/types/record-methods.md).
- [Keywords](../../../pascal/getting-started/keywords.md).
- [Diagnostics](../../../pascal/tools/diagnostics.md).
