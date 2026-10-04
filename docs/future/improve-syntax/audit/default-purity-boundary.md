# Record-default purity boundary

Status: option 1 approved by the user. The coordinated stage-4 purity/default
implementation uses evaluation purity without the pure-function result-type
restriction on defaults. This audit records the contract, not implementation
completion.

## Approved rules and boundary

The [contract](../language-contract.md) requires pure record defaults, forbids
references to other fields and mutable enclosing bindings, and fixes construction
order: supplied fields in written order, followed by omitted defaults in field
declaration order, once per construction.

[Stage 5](../stages/05-effects-and-tasks.md) restricts pure function parameters and
results to resource-free data or pure callable values, recursively. It also
requires record defaults to invoke only pure operations. The user clarified that
the function-result type restriction does not apply to a default expression.

This distinction affects the approved replacement of event slots by
`Option of (HandlerType)` fields:

```pascal
type Handler = procedure();
type Button = record
  OnClick: Option of (Handler) := Option.None;
end record;
```

A program that constructs `Button()` and matches `OnClick` passes the current
checker and runs successfully. `Option.None` performs no action and contains no
handler. Its declared field type nevertheless permits an ordinary procedure.
That type cannot be the result of a source `pure function` under the approved
recursive capability rule. Empty resource-bearing containers raise the same
question, including generic defaults whose arguments later contain resources.

## Approved interpretation: evaluation purity

Check the default expression's evaluation with the shared purity checker:

- Invoke only explicitly pure functions.
- Read and capture only immutable resource-free data and pure callable values.
- Reject other-field references and mutable enclosing bindings.
- Check captures when constructing a closure; its body is checked separately
  according to its declared callable capability.
- Keep the default's result compatible with its field type, without requiring
  that field type to satisfy the pure-function result capability.

Under this interpretation, `Option.None` and empty collections remain valid
defaults for ordinary handler or resource-bearing field types. This is a general
evaluation rule, not an exception for event migration or particular libraries.
Reads of existing resource handles or ordinary closure values remain forbidden.
A source pure function still cannot return `Option of (Handler)`.

Compiler-generated default initializers retain their field result types
and checked default-expression metadata. They are internal evaluation machinery,
not source pure functions or first-class pure callable values. Their results must
not manufacture a source purity guarantee.

## Rejected alternative: restrict default result types too

Require every default expression's result type to satisfy the same recursive
capability as a pure function result. The example above is then rejected, even
though its value is absent. Such fields must be supplied explicitly during
construction. Generic defaults must enforce the restriction at concrete
instantiation or when an omitted default is selected.

This interpretation is stricter and changes which options records can supply
empty handler/resource slots by default. It needs an explicit contract statement
before migration.

## Required verification

Cover empty and present Option/Result payloads, empty collections, ordinary/pure
callable field types, resource-containing nominal data, generic instantiations,
captured mutable state, forbidden reads/calls, private construction and aliases.
Pure function parameter/result checks must remain strict in either interpretation.
Verify written-field/default order, skipped supplied defaults and once-per-build
evaluation through ordinary construction and cold/warm compiled-unit reuse.
