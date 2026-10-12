# AP34: Consistent type inference

Status: Agreed direction; AP34.1 complete, AP34.2 not implemented. Effort: large.
Completion is tracked in the [central plan](../README.md). The selected rules
are recorded in [AP34.1](01-inference-decision.md). Current compiler and handbook
behavior remains Q14 until AP34.2 delivers its replacement; AP22 remains complete.

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

## Selected decisions

- Infer every fully determined initializer for local bindings in statement
  lists, regardless of expression form. Calls, operators, field/index access,
  factories and literals follow the same rule. Explicit annotations remain valid.
  This replaces Q14's initializer-form whitelist within that scope.
- Infer generic arguments from supplied values first. An expected type fills
  only unresolved parameters; it cannot override arguments already determined.
  Reject conflicts and unresolved parameters. Apply the same rule to direct
  routine/member calls, constructors and callable values.
- Keep parameters, routine results and record fields explicitly typed, along
  with declarations outside statement lists, including those before `begin`.
  Preserve constraints, nominal type identity and AP33's explicit numeric
  conversion rule. Defaults do not determine generic arguments.

The decision is recorded; this planning update does not implement AP34.2.

## Dependencies and work packages

AP22/AP24 are complete. AP34.1 precedes AP34.2. Do not wait for AP27 or reopen
it. Numeric promotion policy belongs to AP33; inference must not invent it.

- [x] [AP34.1: Inference decision](01-inference-decision.md)
- [ ] [AP34.2: Consistent inference rules](02-inference-rules.md)

## Acceptance

One recorded rule explains when annotations may be omitted and how generic
arguments are determined. Equivalent direct/callable/constructor contexts
follow that rule or have an explicitly retained semantic distinction.

## Open decisions

None within the selected scope. AP34.2 remains the implementation work package.
