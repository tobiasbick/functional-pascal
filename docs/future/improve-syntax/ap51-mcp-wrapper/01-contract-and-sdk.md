# AP51.1: Agree scope and validate the SDK

Package: [AP51](README.md). Status: Decision pending.

## Deliverable and prerequisites

Record the user's Q58–Q64 decisions in the package README and produce a concrete
API/SDK coverage matrix for both roles and the selected protocol. Q58 and Q64 are
resolved: client and server use the official Rust SDK `rmcp` with an FPAS wrapper.
SDK selection is settled; validate its release, features and integration. This is a decision
and feasibility work package, not authorization to implement the public API.
Prerequisites are the delivered ACP/process infrastructure and the versioned
MCP sources linked from the package README.

## Investigation

1. Verify a published `rmcp` release, pin its exact version and necessary Cargo
   features, and confirm the workspace Rust toolchain and TLS backend fit.
   The inspected 3.7.0 source is a candidate; do not depend on floating `main`.
2. Map core and selected extension methods, result variants, schema types,
   discovery/version rules, MRTR, subscriptions and cancellation onto SDK APIs.
   Separate missing SDK support from missing FPAS binding support. Any SDK gap
   requires a bounded repair or an explicit scope decision, not a second engine.
3. Inspect the SDK's internal queues, eager decode behavior, HTTP buffering,
   cancellation, child cleanup and handler futures. An outer queue limit does
   not prove bounded memory inside the SDK or transport.
4. Specify FPAS unit names, handles, events, typed errors, request/continuation
   state transitions, limits and explicit effects. Follow `Std.Acp` conventions
   without transplanting ACP sessions or the legacy MCP handshake.
5. Confirm that the chosen SDK configuration supports modern discovery without
   silently falling back to `initialize`. Define behavior when discovery is
   skipped and a method or capability is unavailable.
6. For HTTP, decide credential ownership and auth-failure behavior. Full OAuth
   would additionally require issuer/resource validation, registration policy,
   callbacks, refresh and storage; it cannot be hidden inside a token parameter.

Specify the server APIs delivered by AP51.7 and AP51.8 before closing
this decision step. Cover native listener ownership, per-request caller identity,
bounded concurrent handlers, local execution policy, transport headers and the
integrity/replay rules of application-owned MRTR `requestState`.

## Affected files

| Action | Path | Purpose |
| --- | --- | --- |
| Modify | `docs/future/improve-syntax/ap51-mcp-wrapper/README.md` | Record decisions and the final capability/API coverage matrix |
| Modify | `docs/future/improve-syntax/ap51-mcp-wrapper/02-types-and-codecs.md` through `09-acceptance.md` | Align scope, owners, dependencies and acceptance |
| Modify | `docs/future/improve-syntax/README.md` | Update package status and next free IDs if the breakdown grows |

SDK experiments and downloaded sources belong under `.temp-data/`. They do not
become product dependencies or current handbook claims at this stage.

## Acceptance

All seven decisions are recorded, the chosen release and features are verified,
and each selected capability has an implementation and test owner. Public API
examples remain proposals until approved. Resolve the remaining API/transport/auth
boundaries before implementation; check links and `git diff --check`.
