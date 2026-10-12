# Structured editing in the editor and LSP

Status: planning only; implementation has not started.

This plan contains the tooling scope of [AP28](README.md). It is separate from
the language plan and changes no FPAS syntax or semantics.

## Prerequisites

- [AP13: Explicit block boundaries](../ap13-explicit-block-boundaries/README.md).
- [AP24: Generic data structures](../ap24-generic-data-structures/README.md).
- Reuse the import rules from
  [AP05](../ap05-qualified-imports/README.md).

## Agreed scope

- Audit existing LSP/editor facilities before adding a stable, small
  declaration/symbol contract.
- Start with one symbol-aware rename or enum-arm insertion.
- Keep Pascal source text and canonical formatter output authoritative.
- Derive a compact public-signature view from the same declarations, not
  a separately maintained second interface.
- Test that structured edits preserve unrelated source and name binding,
  and that public views match source declarations.

## Implementation owners

- `crates/fpas-language-service/`: declarations, symbols, and name resolution.
- `crates/fpas-lsp/`: protocol requests and source edits.
- `editors/vscode/`: editor integration and real Extension Host verification.
- `crates/fpas-fmt/`: canonical source formatting.

## Next step

Inspect the existing facilities and select one rename or enum-arm insertion
operation. Define its declaration/symbol contract and affected paths before
implementation, retaining the agreed scope above.

## Acceptance

A local structured edit preserves unrelated behavior and binding. Generated
public-signature views remain consistent with their source declarations.
Verify the language-service and LSP contracts, resulting FPAS source, canonical
formatting, and the operation in the real Extension Host.
