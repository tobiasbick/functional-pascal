# AP37.1: Mutation model

Package: [AP37](README.md). Status: Decision pending; decision work package.

## Scope and prerequisites

Compare explicit receiver write markers, general mutating methods, and
value-producing collection APIs. AP06/AP17 provide the current rule; inspect
AP36.1's argument decision or resolve the shared syntax question together.
Do not silently weaken the completed visible-caller-mutation goal.

## Deliverable and affected areas

Record the user's receiver syntax, permitted Self modes, native catalog
classification, and expected behavior for chaining, indexing, fields,
bound methods, callbacks, and `go`. Keep aliasing and lifetime constraints
separate from stylistic convenience. Planning files only change.

## Documentation and verification

Current [var](../../../pascal/language/functions/var-parameters.md),
[method](../../../pascal/language/types/record-methods.md), and native API
references remain unchanged. The decision table must classify Push, Pop,
Sort, Remove, a user updater, and a value-returning record transformation.
Record costs and migration rules before AP37.2 starts.
