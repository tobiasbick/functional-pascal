# AP27: Typed placeholders

Status: retained as optional (Q21); AP22 implementation required. Effort: very
large. Completion is tracked in the [central README](../README.md); the
process is in [development-process.md](../development-process.md).

## Goal

Incomplete routines produce useful analysis but cannot accidentally become
executable programs.

## Decisions (Q21)

- Retain this package as optional, after AP22's limited local inference is
  implemented. Preserve its annotation requirements and inference limits; do
  not broaden inference.
- Report expected types and suitable available values/functions for incomplete
  code; reject unresolved placeholders in executable programs.
- The placeholder spelling still requires an explicit decision before language
  implementation.

## Open decisions

- The single fixed placeholder spelling (AP27.1).

## Dependencies

- AP02 (structured diagnostics carry the analysis).
- AP22 (implemented limited local inference).

## Order

AP27.1 records the spelling; AP27.2 implements analysis and rejection.

## Work packages

- [ ] [AP27.1: Placeholder spelling decision](01-placeholder-spelling-decision.md)
- [ ] [AP27.2: Placeholder analysis](02-placeholder-analysis.md)

## Acceptance

Incomplete routines produce useful analysis but cannot accidentally become
executable programs.
