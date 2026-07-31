# Phase 09: workspace navigation

Status: planned; depends on Phase 08.

## Goal

Expose the language service's project knowledge through the remaining
high-value navigation features used by VS Code-compatible editors.

## Scope

- Expose the existing workspace symbol index through `workspace/symbol`.
- Support deterministic query filtering, stable ordering, result limits, and
  duplicate short names from distinct units.
- Add document highlights for resolved reads, writes, and declarations in the
  current document.
- Add type-definition navigation for variables, parameters, fields, properties,
  routine results, and named aliases where the semantic model identifies a
  source declaration.
- Add syntax-aware selection ranges from identifier to declaration, statement,
  routine/type, and compilation-unit boundaries.
- Reuse the Phase 08 project index and current unsaved snapshots.

## Out of scope

- Call hierarchy, implementation hierarchy, CodeLens, and inlay hints.
- Program or unit rename that moves files or edits manifests.
- Approximate text matching that ignores symbol identity.

## Expected ownership

```text
crates/fpas-language-service/src/navigation/
  workspace_symbols.rs  — query and ranking over the existing symbol index
  highlights.rs         — same-document resolved occurrences
  type_definition.rs    — semantic type target resolution
  selection.rs          — nested syntax/source ranges
crates/fpas-lsp/src/navigation/
  ...                   — protocol conversion per feature
```

The exact layout may adapt to existing modules, but each feature keeps its own
implementation and tests.

## Acceptance criteria

- [ ] `Ctrl+T` finds public and local project declarations with qualified owners.
- [ ] Empty and partial workspace-symbol queries are fast, bounded, and stable.
- [ ] Equal short names from two units remain distinct.
- [ ] Document highlights respect lexical shadowing and ignore comments/strings.
- [ ] Type definition follows imported, qualified, aliased, and record-member types.
- [ ] Selection expansion never splits UTF-16 surrogate pairs or crosses malformed
  source recovery boundaries incorrectly.
- [ ] Unknown or inaccessible symbols return empty results, not protocol errors.

## Required tests

- [ ] Language-service tests for filtering, ranking, shadowing, aliases, members,
  private declarations, malformed source, and unsaved changes.
- [ ] LSP tests for UTF-16 conversion and every new advertised capability.
- [ ] Extension Host tests for workspace symbol search, document highlight, and
  type definition on the fixture workspace.

## Progress record

- Started: not started.
- Completed: not completed.
- Extension version: next unused patch version.
- Artifact: not built.
- SHA-256: not recorded.
- Verification: not run.
