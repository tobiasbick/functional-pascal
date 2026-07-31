# Phase 11: semantic tokens and code actions

Status: planned; depends on Phases 08 through 10.

## Goal

Add semantic highlighting and safe, compiler-backed quick fixes without hiding
or reinterpreting compiler diagnostics.

## Scope

- Provide semantic tokens for namespaces/units, types, enums, enum members,
  functions, procedures, methods, parameters, variables, fields, properties,
  constants, and type parameters represented by the current semantic model.
- Mark declaration, readonly, public, and other supported modifiers only when
  the semantic model proves them.
- Support full-document tokens first; add delta responses only after measured
  benefit and explicit regression coverage.
- Keep the TextMate grammar as the startup and recovery fallback.
- Add code actions tied to stable `Fxxxx` diagnostics when the correction is
  deterministic and semantics-preserving.
- Initial actions may include adding one unambiguous `uses` import, removing an
  unused import when proven, or applying an exact compiler-provided replacement.
- Advertise no fix for a diagnostic that only has explanatory help.

## Out of scope

- Speculative refactors, source generation, or style suggestions.
- Suppressing diagnostics in the extension.
- AI-generated fixes.
- Semantic recoloring of comments or string contents.

## Expected ownership

```text
crates/fpas-language-service/src/semantic_tools/
  tokens.rs          — semantic token classification
  code_actions.rs    — diagnostic-to-edit mapping and safety checks
crates/fpas-lsp/src/semantic_tools/
  ...                — token encoding and code-action conversion
```

TextMate grammar and semantic token scopes remain separate concerns.

## Acceptance criteria

- [ ] Shadowed identifiers receive the classification of the declaration they
  actually resolve to.
- [ ] Tokens preserve stable ordering and non-overlapping UTF-16 ranges.
- [ ] Incomplete source returns safe partial tokens and never crashes the server.
- [ ] TextMate highlighting remains available before server activation and when
  semantic analysis is unavailable.
- [ ] Every offered code action is associated with the triggering diagnostic and
  produces parseable, canonically formatted source.
- [ ] Ambiguous, stale, inaccessible, or changed diagnostics offer no unsafe edit.
- [ ] Applying a code action refreshes diagnostics without restarting the server.

## Required tests

- [ ] Token classification tests for every supported symbol kind and modifier.
- [ ] Shadowing, qualified names, malformed source, stale document, and UTF-16 edge
  cases.
- [ ] One positive and at least one rejection test per code-action family.
- [ ] LSP capability, token encoding, diagnostic association, and workspace-edit
  tests.
- [ ] Extension Host tests for semantic highlighting availability and one quick fix.

## Progress record

- Started: not started.
- Completed: not completed.
- Extension version: next unused patch version.
- Artifact: not built.
- SHA-256: not recorded.
- Verification: not run.
