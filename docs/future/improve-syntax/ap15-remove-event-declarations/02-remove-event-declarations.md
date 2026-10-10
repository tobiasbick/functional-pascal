# AP15.2: Remove event declarations

Package: [AP15: Remove event declarations](README.md)

Status: complete.

## Result

`event`, `nil`, `read`, `write`, and `Assigned` are ordinary identifiers.
Declarations, references, fields, and routine calls use the normal
case-insensitive name-resolution rules. Unresolved `nil` and `Assigned` uses
report the ordinary unknown-name diagnostic.

Records contain stored fields and instance/static routines. There is no event
production, AST node, semantic type, accessor resolution, event lowering,
compiled-unit metadata, formatter emitter, or editor event symbol kind.
Optional callable fields use the ordinary handler model in
[AP15.1](01-migrate-events.md).

Invalid record declarations use ordinary parser errors. No migration
diagnostic, replacement hint, legacy event recognition, diagnostic code,
or migration code action is added. The event accessor-order code FP2011
is absent from the compiler and current diagnostic catalog; other codes
keep their values.

## Ownership

- `crates/fpas-lexer/src/token/{kind.rs,keywords.rs}` and
  `crates/fpas-parser/src/{ast,parser}` own keywords, expressions, and record
  declarations.
- `crates/fpas-sema/src/{check,types,interface}` owns ordinary declaration,
  field, callable, visibility, and exported-interface resolution.
- `crates/fpas-compiler/src/lowering/` and
  `crates/fpas-unit/src/interface/` use field and method metadata.
- Project source maps, formatter traversal/emission, and debugger expression
  parsing follow the current AST.
- Language-service symbol extraction, completion, navigation, and semantic
  tokens, LSP capabilities, and the VS Code grammar classify ordinary names.

## Regression coverage

Lexer tests check every ASCII casing of the five identifiers and keyword
parity with the handbook and formal grammar. Parser tests cover normal fields,
routines and type names, plus ordinary errors for invalid member declarations.
Semantic tests cover unknown names, case-insensitive user routines without
intrinsic metadata, and record field/method resolution.

`tests/runner/ordinary_names_test.fpas` executes field mutation, record methods,
record copies, and a user-defined `Assigned`. Debugger tests accept the same
names in expressions and field targets. Editor tests check semantic symbol
kinds, the LSP token legend, and TextMate scopes. Handbook declaration tests
use the complete optional-handler example.

The ordinary handler tests from AP15.1 retain constructor/default, assignment,
clearing, capture, copy, interface, and evaluation-order coverage.

## Current documentation

- [Keywords](../../../pascal/getting-started/keywords.md).
- [Records](../../../pascal/language/types/records.md) and
  [record methods](../../../pascal/language/types/record-methods.md).
- [Optional handlers](../../../pascal/language/functions/first-class.md#optional-handlers).
- [Visibility](../../../pascal/program-structure/visibility.md).
- [Formal grammar](../../../specs/grammar.ebnf),
  [formatter style](../../../pascal/tools/fmt-style.md),
  [editor integration](../../../pascal/tools/editor-integration.md), and
  [diagnostics](../../../pascal/tools/diagnostics.md).
