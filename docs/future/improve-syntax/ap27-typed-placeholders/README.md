# AP27: Typed placeholders

Status: deferred by user decision; optional proposal retained for possible
later reassessment. No work is scheduled. AP02 and AP22 dependencies are
complete, but no spelling or implementation is approved. Effort if reopened:
very large. Completion is tracked in the [central README](../README.md); the
process is in [development-process.md](../development-process.md).

## Goal

The retained proposal would let incomplete routines produce useful analysis
without accidentally becoming executable programs. This is an inactive goal,
not implemented behavior.

## Decisions (Q21)

- Defer this package. The user does not currently want to pursue its editor
  convenience and analysis cost; whether it is worth implementing remains open.
  Retain the optional proposal from Q21 for possible later use.
- Reopening requires a new explicit user decision supported by concrete demand
  for incomplete-code analysis. Technical prerequisites alone do not restart it.

## Retained proposal (inactive)

- Preserve AP22's annotation requirements and inference limits; do not broaden
  inference.
- Report expected types and suitable available values/functions for incomplete
  code; reject unresolved placeholders in executable programs.
- The placeholder spelling still requires an explicit decision before language
  implementation.

## Open decisions

- Whether the benefit justifies reopening and implementing the package.
- The single fixed placeholder spelling (AP27.1).

## Dependencies

- AP02 (structured diagnostics carry the analysis).
- AP22 (implemented limited local inference).

## Order

No active next step. If reopened by explicit agreement, AP27.1 records the
spelling before AP27.2 implements analysis and rejection. Both drafts remain
unimplemented and unscheduled.

## Work packages

- [ ] [AP27.1: Placeholder spelling decision (deferred)](01-placeholder-spelling-decision.md)
- [ ] [AP27.2: Placeholder analysis (deferred)](02-placeholder-analysis.md)

## Acceptance

For the current decision: retain the proposal, record deferral and unresolved
details, and leave current language behavior unchanged. If reopened, the agreed
scope must ensure incomplete routines provide useful analysis while unresolved
placeholders cannot become executable programs.
