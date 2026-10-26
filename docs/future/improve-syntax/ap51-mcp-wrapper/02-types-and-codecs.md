# AP51.2: Server protocol types and API registration

Package: [AP51](README.md). Status: Not started; depends on AP51.1.

## Contract

Bind official SDK server values to `Std.Mcp.Types` and `Std.Mcp.Server`:
tool descriptions/input schemas, list/call requests, context, content/results
and typed errors. Preserve dynamic JSON, protocol metadata, omission/default
behavior and numeric bounds. A tool's `isError` differs from a JSON-RPC error.
Delegate wire serialization and protocol validation to the SDK.

Publish only server operations and supported features. Incoming client metadata
is server context, not an outbound FPAS client API.

## Implementation owners

| Action | Path | Purpose |
| --- | --- | --- |
| Create | `crates/fpas-mcp/` | Exact official SDK pin and server value/codec bridge |
| Modify | `Cargo.toml`, `Cargo.lock`, consuming manifests | Server-only product features |
| Create/modify | `crates/fpas-sema/src/std_registry/loaded/mcp/`, `crates/fpas-std/src/std_units/`, `crates/fpas-std/src/intrinsics.rs` | Units, types and signatures |
| Create/modify | `crates/fpas-bytecode/src/intrinsic/mcp.rs`, native compiler catalog/selection owners | Intrinsic metadata and lowering |
| Generate | `lib/api/Std/Mcp.Types.fpas`, `lib/api/Std/Mcp.Server.fpas` | Canonical editor declarations |
| Create | `docs/pascal/std/ai/mcp/types.md`, `tests/stdlib/mcp/types_test.fpas` | Contract and FPAS coverage |

Inspect registration/export touchpoints before implementation. Extract generic
ACP binding helpers only where actual reuse warrants it; MCP does not depend on
the ACP wire schema.

## Acceptance

Test SDK/FPAS round trips for schemas, arguments, results and error data,
malformed values, dynamic JSON and numeric boundaries. Verify native names,
CLI/editor typing, generated declarations and artifact identity. No outbound
MCP client API or client transport dependency is introduced.
