# AP34: Consistent type inference

Status: Decision pending. Effort: large. Completion is tracked in the
[central plan](../README.md). AP22 remains complete under Q14 until a new
explicit decision changes its implemented rule.

## Findings

The [local inference whitelist](../../../pascal/language/basics/local-variables.md)
rejects `const X := ReadValue()` but allows `const X := [ReadValue()]` when
the element type is known. Moving a constructor behind a factory changes
annotation requirements.

For `function Empty<T>(): Option of T`, an annotated direct `Empty()` call
cannot infer T from its result. Assigning `Empty` to an explicitly typed
callable first works. Generic constructors also use expected types, so three
contexts expose different inference rules. See
[generic routines](../../../pascal/language/functions/generic-routines.md) and
[constructor inference](../../../pascal/language/types/generics.md#constructor-inference).

## Proposed direction and open decisions

- Reassess Q14 explicitly: require annotations on all bindings, or infer every
  fully determined local initializer. Keeping Q14 is also a valid decision;
  the tradeoff must be recorded instead of silently broadening inference.
- Align missing generic argument inference across constructors, routine
  calls, and callable values; prefer arguments first, expected type filling
  only unresolved parameters, with conflicts rejected.
- Keep public signatures explicit, unknown types rejected, and generic
  defaults separate from inference unless expressly decided otherwise.

## Dependencies and work packages

AP22/AP24 are complete. AP34.1 precedes AP34.2. Do not wait for AP27 or reopen
it. Numeric promotion policy belongs to AP33; inference must not invent it.

- [ ] [AP34.1: Inference decision](01-inference-decision.md)
- [ ] [AP34.2: Consistent inference rules](02-inference-rules.md)

## Acceptance

One recorded rule explains when annotations may be omitted and how generic
arguments are determined. Equivalent direct/callable/constructor contexts
follow that rule or have an explicitly retained semantic distinction.
