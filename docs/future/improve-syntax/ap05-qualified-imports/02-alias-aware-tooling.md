# AP05.2: Alias-aware editor tooling

Package: [AP05: Qualified imports](README.md)

## Scope

Make completion, navigation, rename, semantic tokens, and auto-import aware of
import aliases.

## Prerequisites

- AP05.1 (import aliases).

## Implementation

- Completion after `Alias.` lists the unit's public symbols.
- Go to definition and references resolve through aliases.
- Rename reports conflicts with aliases.
- Auto-import prefers an existing alias when one is present for the unit.
- Highlighting of `as` in `uses`.

## Affected areas

- `crates/fpas-language-service/src/intellisense/` (`completion.rs`,
  `auto_import.rs`), navigation and rename modules; `editors/vscode/`.

## Migration

None.

## Documentation

- `docs/pascal/tools/editor-integration.md` if behavior visible to users changes.

## Verification

- Language-service tests for completion, navigation, rename conflicts, and
  auto-import with and without aliases.
- `node editors/vscode/scripts/verify-grammar.mjs`.
