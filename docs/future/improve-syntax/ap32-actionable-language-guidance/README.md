# AP32: Actionable language guidance

Status: Decision pending (bounded correction plan; implementation not started).
Effort: small. Completion is tracked in the [central plan](../README.md).

## Findings

- Ambiguous-import hints recommend resolving a free function as `Value.Name`,
  contrary to the implemented [dot-call rules](../../../pascal/language/functions/fluent-calls.md).
- Method explanations show type-qualified instance calls as if they were
  usable FPAS, although the [method rules](../../../pascal/language/types/record-methods.md)
  reject that form.
- Boolean `case` introductions omit the valid expression fallback alternative.
- Task-group prose says handles can be discarded, while the `discard` keyword
  rejects task-containing values; merely retaining an unused binding differs.
- Routine branch and loop examples add unnecessary blocks despite already
  having a scope, offering competing default styles.

## Proposed scope and open decisions

Correct guidance against current behavior without changing syntax or semantics.
Confirm this bounded scope when selecting work. AP35/AP39/AP40 may later
change the underlying rules; this package must not preempt their decisions.
The syntax implementation review directory remains a separate review process,
not a duplicate backlog for these new proposals.

## Dependencies and work packages

Existing AP02/AP06/AP13/AP26 rules are the baseline. The two work packages are
independent corrections and do not wait for later redesigns.

- [ ] [AP32.1: Valid diagnostic hints](01-valid-diagnostic-hints.md)
- [ ] [AP32.2: Current-language examples and terminology](02-current-language-guidance.md)

## Acceptance

Suggested corrections compile in the context that triggered the diagnostic.
Examples teach valid current forms, and conceptual lowering is described in
prose rather than as a forbidden alternative call.
