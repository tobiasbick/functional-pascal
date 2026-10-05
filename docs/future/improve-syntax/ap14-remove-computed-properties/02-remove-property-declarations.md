# AP14.2: Remove property declarations

Package: [AP14: Remove computed properties](README.md)

## Scope

Remove `property`, `read`, and `write` from the language and diagnose property
declarations with the method replacement.

## Prerequisites

- AP14.1 (no property uses remain).

## Implementation

- Lexer: remove the three keywords; they become identifiers.
- Parser: recognize a former property declaration and report a diagnostic
  showing the getter function and setter procedure.
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

- Rejection test with the replacement hint; `read`/`write` usable as identifiers.
- Workspace tests, FPAS suite, VS Code grammar verification.
