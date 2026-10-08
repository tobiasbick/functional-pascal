# AP05: Qualified imports

Status: complete (Q04). Effort: medium. Completion is tracked in the
[central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

Qualified lookup is stable under unrelated imports, with concrete collision
diagnostics, and the default meaning of `uses` is a recorded decision.

Current behavior: plain `uses` imports open short and fully qualified public
names; aliases expose only `Alias.Name`. Repeated direct imports and alias
collisions are rejected; ambiguous plain-import short names are reported at
the point of use ([grammar](../../../specs/grammar.ebnf),
`uses_clause`).

## Decisions (Q04)

- A plain `uses Std.Fs;` keeps opening public symbols under their short names.
  Ambiguous short names are errors at the point of use; qualified names remain
  available.
- `uses Std.Fs as Files;` exposes an alias. An aliased import opens no short names; its
  symbols are reached only through the alias, for example `Files.ReadText(Path)`.
  The five type-helper units removed by AP06 are unavailable as imports;
  other standard units follow these rules.
- Import each unit at most once per source file, comparing canonical unit
  names case-insensitively. Repeated plain imports, multiple aliases for one
  unit, and a mixture of plain and aliased imports of that unit are errors.
  This applies to that file's direct imports, not to transitive dependencies.
- `as` is contextual syntax in `uses`, recognized case-insensitively after
  the imported unit name. It is not globally reserved and remains an ordinary
  identifier outside that position.
- Diagnose an alias that collides with another alias or a local name.
- Public parameter and result types stay explicit.

## Open decisions

None. Language semantics and editor tooling implement these rules.

## Dependencies

- AP01 (reference style), AP02 (diagnostic codes).

AP19 depends on this package. AP06's native built-in type operations are
available without imports and do not depend on AP05; other units continue
to follow this import model.

## Editor behavior

Completion, navigation, references, hover, signatures, type navigation, rename,
and highlighting use source-local aliases while keeping canonical imported
symbol identities. Alias completion exposes only public declarations.
Auto-import reuses direct imports case-insensitively and inserts an existing
alias qualifier without adding another import. Renames reject alias collisions
and update local alias declarations and uses. `as` is a keyword only in its
import-modifier position in both semantic tokens and TextMate highlighting.

## Work packages

- [x] [AP05.1: Import aliases](01-import-aliases.md)
- [x] [AP05.2: Alias-aware editor tooling](02-alias-aware-tooling.md)

## Acceptance

Qualified lookup is stable under unrelated imports, with concrete collision
diagnostics, and plain imports keep their recorded default meaning.
