# Phase 10: IntelliSense

Status: planned; depends on Phases 08 and 09.

## Goal

Turn basic declaration completion into useful editing assistance while keeping
all suggestions grounded in the compiler's syntax, symbols, and visibility
rules.

## Scope

- Enrich completion items with accurate kind, qualified owner, type/signature,
  source detail, sorting, replacement range, and documentation when available.
- Add completion resolve so expensive documentation is computed only for the
  selected item.
- Complete routine parameters, local declarations, visible unit declarations,
  record members, enum members, and appropriate keywords in recovered source.
- Add signature help for functions, procedures, constructors, methods, nested
  routines, function values, and generic calls supported by the current
  language.
- Track the active argument across nested calls and multiline expressions.
- Add repository-owned snippets for common, valid FPAS declarations and control
  flow without duplicating semantic completion logic.
- Offer an auto-import completion only when one public declaration and one
  unambiguous unit import can satisfy the unresolved identifier safely.

## Out of scope

- Guessing imports when multiple units export the same short name.
- AI-generated completion.
- New language syntax, overload rules, or implicit imports.

## Expected ownership

```text
crates/fpas-language-service/src/intellisense/
  completion.rs       — candidates, context, ranking, replacement ranges
  signature_help.rs   — callable and active-parameter resolution
  auto_import.rs      — deterministic unresolved-name import edits
crates/fpas-lsp/src/intellisense/
  ...                 — LSP conversion and resolve handlers
editors/vscode/snippets/
  fpas.json           — declarative language snippets
```

If current navigation completion is moved, remove the old implementation and
keep one authoritative completion path.

## Acceptance criteria

- [ ] Completion remains correct with incomplete but recoverable source.
- [ ] Suggestions exclude private, shadowed, and non-exported declarations.
- [ ] Member completion reports the member's real type and owner.
- [ ] Signature help selects the correct callable and active parameter in nested,
  generic, and multiline calls.
- [ ] Completion replacement ranges preserve surrounding punctuation and Unicode
  strings/comments.
- [ ] Auto-import edits only the compilation unit's `uses` clause, preserve
  formatter output, and are absent for ambiguous or inaccessible candidates.
- [ ] Snippets produce source accepted by the current parser and formatter.

## Required tests

- [ ] Positive, ambiguous, inaccessible, shadowed, malformed, and nested-call tests
  in the language service.
- [ ] LSP tests for completion resolve, text edits, snippets, signature ranges, and
  UTF-16 positions.
- [ ] Extension Host tests for member completion, signature help, snippets, and one
  safe auto-import.

## Progress record

- Started: not started.
- Completed: not completed.
- Extension version: next unused patch version.
- Artifact: not built.
- SHA-256: not recorded.
- Verification: not run.
