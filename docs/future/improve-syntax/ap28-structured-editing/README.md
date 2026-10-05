# AP28: Structured editing

Status: outside the language plan; transfer to editor/LSP planning after AP13
and AP24 (Q23). This package has no work packages here. Completion is tracked
in the [central README](../README.md).

## Destination (Q23)

Retain this as tooling work outside the language plan. After AP13 and AP24 are
complete, transfer it to editor/LSP planning. This README preserves the
handoff scope; it is not an active language package and changes no syntax.

## Scope to transfer

- Audit existing LSP/editor facilities before adding a stable, small
  declaration/symbol contract; reuse the import rules from AP05.
- Start with one symbol-aware rename or enum-arm insertion.
- Keep Pascal source text and canonical formatter output authoritative.
- Derive a compact public-signature view from the same declarations, not
  a separately maintained second interface.
- Test that structured edits preserve unrelated source and name binding,
  and that public views match source declarations.

Tooling acceptance: a local structured edit preserves unrelated behavior and
binding, and generated contract views remain consistent with their source.

## Transfer conditions

- AP13 and AP24 are complete.
- The receiving editor/LSP plan under `docs/future/` takes over this scope;
  this README then links to it.
