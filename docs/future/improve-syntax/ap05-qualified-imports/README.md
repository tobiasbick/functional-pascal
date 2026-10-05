# AP05: Qualified imports

Status: agreed direction (Q04). Effort: medium. Completion is tracked in the
[central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

Qualified lookup is stable under unrelated imports, with concrete collision
diagnostics, and the default meaning of `uses` is a recorded decision.

Current behavior: `uses` brings every public symbol of a unit into scope under
its short name; ambiguous short names are reported at the point of use, and the
fully qualified name always works ([grammar](../../../specs/grammar.ebnf),
`uses_clause`).

## Decisions (Q04)

- A plain `uses Std.Str;` keeps opening public symbols under their short names.
  Ambiguous short names are errors at the point of use; qualified names remain
  available.
- Add `uses Std.Str as Text;`. An aliased import opens no short names; its
  symbols are reached only through the alias, for example `Text.Trim(Input)`.
- Diagnose an alias that collides with another alias or a local name.
- Public parameter and result types stay explicit.

## Dependencies

- AP01 (reference style), AP02 (diagnostic codes).

AP06 and AP19 depend on this package.

## Order

AP05.1 delivers the language change; AP05.2 brings editor tooling up to date.

## Work packages

- [ ] [AP05.1: Import aliases](01-import-aliases.md)
- [ ] [AP05.2: Alias-aware editor tooling](02-alias-aware-tooling.md)

## Acceptance

Qualified lookup is stable under unrelated imports, with concrete collision
diagnostics, and plain imports keep their recorded default meaning.

## Reference

The reference branch `codex/syntax-changes` implemented alias-only imports
(every import requires an alias; no short names). That diverges from Q04 and
is not adopted. Its owner map remains useful: parser `program.rs`, sema
`check/entry.rs`, `check/name_resolution/`, `interface/install.rs`, and
project `unit_graph/resolve.rs`.
