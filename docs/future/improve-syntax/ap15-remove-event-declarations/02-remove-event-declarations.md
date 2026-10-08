# AP15.2: Remove event declarations

Package: [AP15: Remove event declarations](README.md)

## Scope

Remove `event`, the event-only `nil` literal, and `Assigned`; diagnose them
with the optional-field replacement. Release `read` and `write`, which only
event accessors still use after AP14.2.

## Prerequisites

- AP15.1 (no event uses remain).
- AP20.3 (`is` test, shown in the diagnostic hint).

## Implementation

- Lexer: remove `event`, `nil`, `read`, and `write`; they become identifiers.
- Parser: recognize event declarations and `nil`; report the replacement
  (`Option of HandlerType` field, `None`, and an `is Some(const Handler)` test).
- Sema: diagnose `Assigned` with the same replacement.
- Remove event metadata from sema, compiler, unit interfaces, formatter, and
  language service.

## Affected areas

- `crates/fpas-lexer/src/token/keywords.rs`, parser record members,
  `crates/fpas-sema/src/check/decl/types/record_events.rs`,
  `check/expr/event_access.rs`, `check/stmt/event_assignment.rs`, compiler and
  unit-interface event paths, `fpas-fmt`, `fpas-language-service`,
  `editors/vscode/syntaxes/`.

## Migration

None beyond AP15.1.

## Documentation

- Remove `docs/pascal/language/types/record-events.md` and its links; update
  `docs/specs/grammar.ebnf` (`record_event`, `nil`), records and keyword pages.

## Verification

- Rejection tests for `event`, `nil`, and `Assigned` with hints; `read` and
  `write` usable as identifiers.
- Workspace tests, FPAS suite, VS Code grammar verification.
