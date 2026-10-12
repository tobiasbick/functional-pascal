# AP34: Consistent type inference

Status: complete. Effort: large.
Completion is tracked in the [central plan](../README.md). The selected rules
are recorded in [AP34.1](01-inference-decision.md) and implemented by
[AP34.2](02-inference-rules.md). AP34 replaces Q14's initializer-form restriction;
AP22 remains complete for its literal/construction work.

## Implemented behavior

The [local inference rule](../../../pascal/language/basics/local-variables.md)
accepts every fully determined statement-local initializer. Ordinary calls and
collections, constructors and factories, operators and member access all retain
their checked types without requiring annotations solely because of their form.

For `function Empty<T>(): Option of T`, an annotated direct `Empty()` call and
an explicitly typed callable both determine T from their result context.
Supplied values contribute first; context completes only unresolved types.
Native callbacks and sibling collection/decision values follow the same rule.
Every unresolved routine, callable-value or constructor site is rejected before
lowering, including sites hidden by complete outer results. See
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

Generic parameters retain their complete lexical declaring path. Equal source
names in nested routines do not make their parameters interchangeable. The
existing receiving-task annotation rule remains in
[AP30](../ap30-task-result-types/README.md).

## Dependencies and work packages

AP22/AP24 are complete. AP34.1 precedes AP34.2. Do not wait for AP27 or reopen
it. Numeric promotion policy belongs to AP33; inference must not invent it.

- [x] [AP34.1: Inference decision](01-inference-decision.md)
- [x] [AP34.2: Consistent inference rules](02-inference-rules.md)

## Acceptance

One recorded rule explains when annotations may be omitted and how generic
arguments are determined. Equivalent direct/callable/constructor contexts
follow that rule or have an explicitly retained semantic distinction.

## Open decisions

None within the implemented scope.
