# Structured editing in the editor and LSP

Status: complete; the bounded tooling scope below is implemented and reviewed.

This plan contains the tooling scope of [AP28](README.md). It is separate from
the language plan and changes no FPAS syntax or semantics.

## Prerequisites

- [AP13: Explicit block boundaries](../ap13-explicit-block-boundaries/README.md).
- [AP24: Generic data structures](../ap24-generic-data-structures/README.md).
- Reuse the import rules from
  [AP05](../ap05-qualified-imports/README.md).

## Agreed scope

- Reuse the existing symbol-aware F2 rename, source snapshots, visibility,
  reference lookup and versioned workspace edits. Do not add a second rename path.
- Complete rename acceptance by applying its edits in the real Extension Host,
  checking unchanged surrounding source, name binding and canonical formatting.
- Keep Pascal source text and canonical formatter output authoritative.
- Provide one read-only public-signature view for the current unit, derived from
  its current source declarations, including unsaved changes. Show imports,
  exported binding types, public types and record members, routine headings and
  declared contracts; omit implementation bodies and private members. Reuse the
  canonical formatter's declaration emission. The view is editor documentation,
  not an independently stored or compilable interface.
- Reject incomplete syntax and program sources with an actionable message.
- Keep enum-arm insertion outside this delivery.
- Test that structured edits preserve unrelated source and name binding,
  and that public views match source declarations.

## Implementation owners

- `crates/fpas-language-service/`: declarations, symbols, and name resolution.
- `crates/fpas-lsp/`: protocol requests and source edits.
- `editors/vscode/`: editor integration and real Extension Host verification.
- `crates/fpas-fmt/`: canonical source formatting.

## Delivery

- [x] Reuse and complete editor acceptance of symbol-aware rename.
- [x] Deliver the source-derived, read-only public-signature view and its
  language-service, LSP and Extension Host checks.

## Acceptance

A local structured edit preserves unrelated behavior and binding. Generated
public-signature views remain consistent with their source declarations.
Verify the language-service and LSP contracts, resulting FPAS source, canonical
formatting, and the operation in the real Extension Host.
The view updates after source edits and never writes sources, manifests or
compiled units. AP28 changes no language syntax or semantics.

## Regression coverage

- Formatter tests own public visibility projection, generic and nominal type
  spellings, record defaults, headings/contracts and source/AST consistency.
- Language-service tests own unsaved snapshot selection, program/syntax rejection
  and preservation of source files and compiled-unit sidecars.
- LSP tests own notification ordering, editor versions and closed-document errors.
- The Extension Host applies cross-file rename, preserves comments and strings,
  checks definition binding, applies canonical formatting with a noncanonical
  control, and checks read-only input, live public-view refresh and syntax errors.
- Qualified completion stops at outer argument/index boundaries while preserving
  balanced receiver expressions; its public endpoint has a regression test.

## Verification

Formatting, the Rust build, the full workspace test suite and TypeScript
compilation pass. The AP28 Extension Host acceptance checks pass, including
applied rename and public-view refresh. Both specification and standards reviews
are complete. The full editor suite and strict Clippy gate still encounter the
independent findings listed below.

## Independent follow-ups

- The existing debugger evaluation host fixture searches for an unqualified
  `WriteLn(Offset)` breakpoint although its source contains `Console.WriteLn`.
  Align that fixture's breakpoint selection in debugger-owned work.
- Existing strict-Clippy findings in array sorting, editor navigation/intellisense
  and older editor tests require their own cleanup. They are independent of this
  tooling delivery.
