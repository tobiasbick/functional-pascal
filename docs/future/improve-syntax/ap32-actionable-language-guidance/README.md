# AP32: Actionable language guidance

Status: Complete. Diagnostic hints and language guidance describe implemented behavior.
Effort: small. Completion is tracked in the [central plan](../README.md).

## Current behavior

- Ambiguous-import hints list qualified candidates for expression and statement
  calls. Native-operation migration hints retain their valid receiver forms;
  [dot calls](../../../pascal/language/functions/fluent-calls.md) do not search free routines.
- [Method explanations](../../../pascal/language/types/record-methods.md) describe
  implicit `Self` in prose and show valid value-based instance calls.
- Boolean `case` expressions use unguarded `true` and `false` arms or an `else`
  fallback; Boolean statements need not be exhaustive.
- Task guidance distinguishes lexical ownership, retained handles, unused
  bindings, and the `discard` keyword's task-safety restriction.
- Branch and loop examples use direct statement lists. Explicit nested blocks
  remain where they demonstrate an additional scope; empty bodies use `null`.

## Scope

Guidance corrections retain current syntax and semantics.
AP35/AP39/AP40 retain their separate decisions about the underlying rules.
The syntax implementation review directory remains a separate review process,
not a duplicate backlog.

## Dependencies and work packages

Existing AP02/AP06/AP13/AP26 rules are the baseline. The two work packages are
independent corrections and do not wait for later redesigns.

- [x] [AP32.1: Valid diagnostic hints](01-valid-diagnostic-hints.md)
- [x] [AP32.2: Current-language examples and terminology](02-current-language-guidance.md)

## Acceptance

Suggested corrections compile in the context that triggered the diagnostic.
Examples teach valid current forms, and conceptual lowering is described in
prose rather than as a forbidden alternative call.

## Open follow-ups

None within this package's scope.
