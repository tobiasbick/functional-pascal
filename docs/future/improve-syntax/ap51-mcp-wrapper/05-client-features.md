# AP51.5: Core operations and interaction patterns

Package: [AP51](README.md). Status: Not started; blocked on AP51.1–AP51.3 and
Q61. HTTP acceptance additionally depends on AP51.4.

## Contract and scope

Provide the approved typed client operations for tool listing/calls, resource
listing/templates/reads, prompt listing/retrieval and argument completion.
Expose pagination cursors without implicit unbounded traversal. Preserve tool
annotations as peer data; they are not authorization or a promise of safety.
Keep structured tool errors distinct from JSON-RPC and transport failures.

Support core request/response, progress, subscriptions and MRTR. Discovery is
optional and capability/version metadata travels as required with each request.
Subscription changes arrive through `subscriptions/listen`, with explicit
ownership, cancellation and bounded queues; no legacy `resources/subscribe` API
should leak from SDK compatibility helpers into the modern FPAS contract.

For `input_required`, publish typed elicitation requests and a single-use local
continuation right tied to the exact originating operation and client context.
The caller explicitly supplies accept/decline/cancel results. A continuation
uses a fresh JSON-RPC ID, preserves the original method/arguments and echoes
opaque `requestState` exactly when present. Parallel calls must never exchange
input IDs or state. Bound retained state, round count and cumulative lifetime;
check the final policy and defaults in AP51.1.

The default proposal does not advertise deprecated sampling, roots or logging.
If selected in Q61, model their actual modern MRTR/notification semantics and
caller authorization explicitly; do not add automatic LLM or filesystem effects.
Expose cache hints (`ttlMs`, `cacheScope`) faithfully without introducing an
automatic cache or sharing results between authentication contexts.

## Affected files and owners

| Action | Path | Purpose |
| --- | --- | --- |
| Create | `crates/fpas-mcp/src/client/`, `crates/fpas-mcp/src/interaction/` | Core methods, continuation ownership and selected input validation |
| Modify | `crates/fpas-mcp/src/connection/`, `crates/fpas-vm/src/vm/hosted/mcp/` | Progress, subscriptions, results and lifecycle cleanup |
| Modify | `crates/fpas-sema/src/std_registry/loaded/mcp/`, `crates/fpas-bytecode/src/intrinsic/mcp.rs` | Public typed operations and intrinsic metadata |
| Create | `crates/fpas-mcp/tests/client.rs`, `crates/fpas-mcp/tests/interactions.rs`, `tests/stdlib/mcp/client_test.fpas`, `tests/stdlib/mcp/interactions_test.fpas` | Method and interaction coverage |
| Create | `docs/pascal/std/ai/mcp/client.md`, `docs/pascal/std/ai/mcp/interactions.md` | Supported features and explicit application responsibilities |

## Acceptance and verification

Test mixed-content and non-object structured results, pagination boundaries,
unsupported features, valid and invalid elicitation answers, multiple input
requests and several MRTR rounds. Include late/duplicate responses, forged and
cross-connection continuation handles, cancellation during input collection,
reuse after close, and subscription flood/quiet shutdown. Observe unrelated
request progress throughout. Verify cache metadata and caller identity survive
round trips. Repeat lifecycle cycles to detect retained payloads, rights or slots.
