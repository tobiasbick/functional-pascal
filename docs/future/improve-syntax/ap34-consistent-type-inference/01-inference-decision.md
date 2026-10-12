# AP34.1: Inference decision

Package: [AP34](README.md). Status: Decision pending; decision work package.

## Scope and prerequisites

Compare mandatory annotations, complete local expression inference, and the
current Q14 whitelist using literals, factories, collections, conversions,
generic constructors, direct calls, and callable values. AP22/AP24 supply
the existing baseline. This assessment is authorized planning, not approval
to revise Q14.

## Deliverable and affected areas

Record the user's selected binding rule separately from the generic argument
rule. Include empty collections, payloadless variants, result-only generic
parameters, conflicting expected types, and imported signatures. State the
migration cost and how success differs from guessing a common numeric type.
Update this package's planning files only.

## Documentation and verification

Keep the current local/generic handbook unchanged. The decision must predict
the `ReadValue()` versus `[ReadValue()]` pair and direct `Empty()` versus
typed callable assignment, with a canonical form and useful rejection
guidance. Explicitly record any superseded part of Q14 before implementation.
