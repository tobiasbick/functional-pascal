# Tooling and derivation

The retained packages are optional. Rejected work is recorded without
implementation tasks. See the [steering document](README.md) for dependencies
and explicit decision gates.

## AP27: Typed placeholders

### Decided direction (Q21)

- Retain this package as optional, after AP22's limited local inference is
  implemented. Preserve its annotation requirements and inference limits.
- Report expected types and suitable available values/functions for incomplete
  code; reject unresolved placeholders in executable programs.
- The placeholder spelling still requires an explicit decision before language
  implementation.

### Tasks

- [ ] Require AP22's approved limited local inference to be implemented before
  placeholder analysis; preserve its annotation requirements for ordinary calls
  and other unsupported initializer forms.
- [ ] Choose one fixed placeholder form for incomplete code and report expected
  types, local values, and matching signatures.
- [ ] Distinguish incomplete-code analysis from executable compilation. Reject
  unresolved placeholders when compiling an executable.

Acceptance: incomplete routines produce useful analysis but cannot accidentally
become executable programs.

## AP28: Structured editing

### Decided destination (Q23)

Retain this as tooling work outside the language plan. After AP13 and AP24 are
complete, transfer it to editor/LSP planning. This section preserves the
handoff scope; it is not an active language work package and changes no syntax.

Scope to transfer:

- Audit existing LSP/editor facilities before adding a stable, small
  declaration/symbol contract; reuse the import rules from AP05.
- Start with one symbol-aware rename or enum-arm insertion.
- Keep Pascal source text and canonical formatter output authoritative.
- Derive a compact public-signature view from the same declarations, not
  a separately maintained second interface.
- Test that structured edits preserve unrelated source and name binding,
  and that public views match source declarations.

Tooling acceptance: a local structured edit preserves unrelated behavior and binding,
and generated contract views remain consistent with their source.

## AP29: Bounded derivation

Status: rejected and closed by decision Q22; removed from the active plan.

Records and enums already have structural equality, the package's main
proposed use case. Do not add derivation syntax or machinery. If concrete needs
for debug rendering or ordering arise, address them with ordinary
standard-library functions.
