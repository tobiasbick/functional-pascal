# AP51.2: Shared types and public API registration

Package: [AP51](README.md). Status: Complete.

## Contract

Register `Std.Mcp` and `Std.Mcp.Types` with SDK-backed server values and
operations. Definitions include names, descriptions, schemas and supported
annotations. Request context includes call/server/attachment identity and
cancellation. Results preserve content, structured JSON and the distinction
between tool failure and JSON-RPC failure.

Define the uniform FPAS handler type and registration lifetime specified in
AP51.1. Register server creation, `Mcp.RegisterTool`, cooperative serving and
close/release. Add `Acp.RegisterTool` and existing-server attachment using the
same MCP definition, handler and server types.

Importing or serving `Std.Mcp` requires no ACP connection, ACP payload type or
agent process. `Std.Acp` retains its connection/session interface and delegates
its new convenience operations to MCP. Specify how MCP failures and ACP setup
failures appear to each caller without duplicating the tool error model.

Use explicit schemas and structured argument values. Automatic reflection over
arbitrary FPAS signatures is outside this package. Delegate wire serialization
and protocol validation to the SDK. The shared offline `jsonschema` validator
checks registered input/output schemas. Preserve dynamic JSON, metadata,
omission/default behavior and numeric bounds in both directions.

The native catalog, SDK model exports and offline schema compilation are in
`crates/fpas-mcp/`. The semantic registry, VM value conversion and generated
`Std.Mcp`, `Std.Mcp.Types` and `Std.Acp` editor declarations share the public
types. Programs importing only `Std.Mcp` or only `Std.Acp` are covered by FPAS
regression tests.

## Implementation owners

| Action | Path | Purpose |
| --- | --- | --- |
| Create | `crates/fpas-mcp/` | Exact official SDK pin and server value/codec bridge |
| Modify | `Cargo.toml`, `Cargo.lock`, consuming manifests | Audited server dependency configuration |
| Create/modify | `crates/fpas-sema/src/std_registry/loaded/mcp/`, `crates/fpas-sema/src/std_registry/loaded/acp/` | Shared MCP declarations and ACP convenience signatures |
| Modify | `crates/fpas-sema/src/std_registry/loaded/mod.rs`, `crates/fpas-sema/src/std_units.rs`, `crates/fpas-std/src/std_units/`, `crates/fpas-std/src/intrinsics.rs` | Unit discovery and native names |
| Create/modify | `crates/fpas-bytecode/src/intrinsic/mcp.rs`, `crates/fpas-bytecode/src/intrinsic/acp.rs`, compiler intrinsic owners | Metadata and lowering for both entry points |
| Generate | `lib/api/Std/Mcp.fpas`, `lib/api/Std/Mcp.Types.fpas`, `lib/api/Std/Acp.fpas` | Canonical editor declarations |
| Modify | `crates/fpas-sema/examples/export_intrinsic_std_api.rs`, `crates/fpas-language-service/src/symbols/intrinsic_api.rs` | Shared enum-alias declarations, variant lookup and parameter documentation |
| Create/modify | `docs/pascal/std/ai/mcp/`, `docs/pascal/std/ai/acp/`, `tests/stdlib/mcp/`, `crates/fpas-language-service/tests/` | Delivered interface and language/editor coverage |

Inspect actual registration/export owners before editing. Extract ACP binding
helpers only where concrete reuse warrants it. MCP values and error identities
remain independent of the ACP wire schema.

## Acceptance

Test SDK/FPAS conversions for schemas, arguments, results, error data, malformed
values and numeric boundaries. Verify handler typing, native names, generated
declarations, artifact identity and editor diagnostics. Cover programs importing
only `Std.Mcp`, convenience callers importing only `Std.Acp` and programs
attaching an existing server. Introduce no outbound MCP client interface or
product client transport dependency.
