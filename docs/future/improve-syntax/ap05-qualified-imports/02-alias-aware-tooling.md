# AP05.2: Alias-aware editor tooling

Package: [AP05: Qualified imports](README.md)

Status: complete.

## Result

Completion after `Alias.` lists the imported unit's public declarations.
Aliases appear as local namespace declarations; imported symbols keep their
canonical unit-qualified identities. Aliased imports expose neither short
names nor their original qualified paths in ordinary source queries.

Go to definition, references, hover, signatures, named arguments, and type
navigation resolve through aliases. Selecting an alias navigates to its local
import declaration; selecting an imported unit path navigates to the unit
source. Named types resolve in their declaration's own import environment.
Different files may use different aliases for the same canonical declaration.

Rename rejects collisions with aliases, including unused aliases in nested
scopes. Renaming an alias edits its declaration and local qualifiers, subject
to the compiler's name and namespace reservations. Renaming an imported symbol
updates canonical declarations and uses through aliases while preserving alias
spellings.

Auto-import compares direct imports by canonical unit name, case-insensitively.
Existing plain imports supply normal declaration completions. Existing aliases
produce qualified insertions such as `M.Answer`, with no `uses` edit even when
the import has comments. These take precedence over new imports of equal short
names; distinct existing aliases retain distinct qualified suggestions.
Adding another unit preserves existing aliases and avoids duplicate imports.

Semantic tokens and TextMate highlight `as` as a keyword only in its import
modifier position. Case variations and multiline imports with comments follow
the same rule. Unit segments, aliases, and ordinary identifiers named `As`
retain identifier highlighting.

## Implementation ownership

- `crates/fpas-language-service/src/navigation/document/imports.rs` owns source
  namespaces and canonical import identity; `navigation/resolve.rs` shares
  lookup across editor queries.
- `crates/fpas-language-service/src/symbols/extract/imports.rs` extracts local
  alias declarations; `navigation/rename/aliases.rs` enforces rename reservations.
- `crates/fpas-language-service/src/intellisense/completion.rs` and
  `intellisense/auto_import.rs` implement alias members and import reuse.
- `crates/fpas-language-service/src/semantic_tools/tokens.rs` classifies aliases
  and contextual modifiers. `crates/fpas-lsp/src/semantic_tools/legend.rs`
  maps them to the LSP namespace and keyword token categories.
- `editors/vscode/syntaxes/fpas.tmLanguage.json` tracks unit paths, the optional
  modifier, and alias identifiers across lines without globally reserving `as`.

## Regression coverage

- `crates/fpas-language-service/tests/import_aliases/` covers public member
  completion, hidden names, canonical metadata, plain and aliased import reuse,
  comments, case variations, alias precedence, and preservation of existing aliases.
- The same suite covers definition, references and cross-file symbol rename,
  local alias rename, lexical and namespace conflicts, hover, signatures,
  named arguments, declared result types, and contextual highlighting.
- `crates/fpas-lsp/tests/semantic_tools.rs` verifies the keyword legend and
  encoded modifier, alias, and callable tokens.
- `editors/vscode/test/grammar/import_aliases.fpas` and
  `node editors/vscode/scripts/verify-grammar.mjs` cover fallback highlighting,
  multiline imports, comments, strings, and the distinct roles of `As`.

## Documentation

- [Editor integration](../../../pascal/tools/editor-integration.md) documents
  import-aware navigation, completion, rename, and highlighting.
- [Units](../../../pascal/program-structure/units.md) defines the import model
  shared by the compiler and editor.
