# AP13.1: Reserve elsif, when, and null

Package: [AP13: Explicit block boundaries](README.md)

## Scope

Reserve `elsif`, `when`, and `null` as keywords before the block syntax uses
them, and rename every existing identifier that uses these words.

## Prerequisites

- AP02 (diagnostic codes).
- The open decision on the `JsonValue.Null` replacement name in the
  [package README](README.md).

## Implementation

- Lexer: add the three keywords.
- Parser: diagnose their use as identifiers with a rename hint.
- Rename the `Std.Json` variant `JsonValue.Null` to the agreed name in the
  registry, runtime, generated `lib/api/Std/Json.fpas`, and callers.
- Editor highlighting.

## Affected areas

- `crates/fpas-lexer/src/token/keywords.rs`, parser identifier handling.
- `Std.Json` registration and runtime in `fpas-sema` and `fpas-std`;
  `lib/api/Std/Json.fpas`.
- `editors/vscode/syntaxes/`.

## Migration

- Rename identifiers named `elsif`, `when`, or `null` (case-insensitive) in all
  repository consumers, including JSON tests and documentation examples.

## Documentation

- Keyword list, `docs/pascal/std/text/json.md` (variant name).

## Verification

- Lexer tests for each keyword in all letter cases, inside strings and comments.
- Parser rename diagnostics; JSON tests with the renamed variant.
- VS Code grammar verification.
