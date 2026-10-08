# AP14.2: Remove property declarations

Package: [AP14: Remove computed properties](README.md)

## Scope

Remove `property` from the language and diagnose property declarations with
the method replacement. `read` and `write` remain keywords for event
declarations until [AP15.2](../ap15-remove-event-declarations/02-remove-event-declarations.md).

## Prerequisites

- AP14.1 (no property uses remain).

## Implementation

- Lexer: remove the `property` keyword; it becomes an identifier.
- Parser: recognize a former property declaration and report a diagnostic
  that tells the author to call the accessor methods directly or to declare
  methods deliberately. The hint names the written `read`/`write` methods; it
  does not invent getter or setter names.
- Remove property metadata from sema, compiler, unit interfaces, formatter,
  and language service.

## Affected areas

- `crates/fpas-lexer/src/token/keywords.rs`, parser record members,
  `crates/fpas-sema/src/check/decl/types/record_properties.rs`, compiler and
  unit-interface property paths, `fpas-fmt`, `fpas-language-service`,
  `editors/vscode/syntaxes/`.

## Migration

None beyond AP14.1.

## Documentation

- Remove `docs/pascal/language/types/record-properties.md` and its links;
  update `docs/specs/grammar.ebnf` (`record_property`), records and keyword
  pages.

## Verification

- Rejection test with the replacement hint; `property` usable as an identifier.
- Workspace tests, FPAS suite, VS Code grammar verification.

## Result

- `property` is an ordinary identifier. In a record member list, `property Name`
  reports FP2018 (`PARSE_REMOVED_PROPERTY`) and skips the old declaration while
  keeping the other members. The hint names the written accessors, for example
  `Value.GetZoom()` and `Value.SetBase(NewValue)`; it never invents accessor
  names. `read` and `write` stay keywords for event accessors until AP15.2.
- Removed: the parser AST node, sema property types and getter/setter metadata
  (including getter reads on receiver paths), property assignment checking,
  compiler lowering, IR/bytecode/object record property metadata, unit-interface
  property types, linker comparison, the debugger property getter fallback, the
  formatter emitter, language-service property symbols and semantic tokens, and
  the VS Code grammar keyword. Event accessor resolution became event-only
  (`check/decl/types/record_event_accessors.rs`).
- Formats: `.fpascu` 10, objects 10, bytecode 17, program images 16. The record
  section's minimum entry size shrank to 12 bytes; a record without fields and
  instance methods (`tests/runner/static_record_procedure_test.fpas`) covers it.
- Documentation: `record-properties.md` and its links were removed; grammar,
  keywords, record, event, visibility, debugger, editor, and diagnostics pages
  were updated. Plain `property` names are now valid identifiers.

Coverage: `crates/fpas-parser/src/tests/errors/removed_properties.rs`,
`crates/fpas-cli/src/main_tests/projects/removed_properties.rs`, and the
lexer keyword test.
