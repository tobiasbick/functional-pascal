# AP51.2: Protocol values and FPAS API registration

Package: [AP51](README.md). Status: Not started; blocked on AP51.1.

## Contract and scope

Expose the approved MCP types and lossless payload codecs through `Std.Mcp.Types`
and the agreed client/server operational units. Include server configuration,
incoming requests and single-use response identities. Use the pinned official
`rmcp` SDK as protocol owner. The codec layer converts FPAS values to and from
SDK values and delegates wire serialization to the SDK. Preserve
arbitrary tool arguments, JSON Schema, structured tool output, content blocks,
metadata and peer error data. `structuredContent` may be any JSON value, not
only an object. Keep omitted fields, explicit null and protocol defaults distinct
where the specification makes them distinct.

Model `complete`, `input_required` and selected extension result types explicitly.
Apply specified missing-field defaults, and reject unsupported result variants
according to the agreed protocol rules. Preserve raw diagnostic data without
turning unsupported methods into enabled capabilities. Keep local failures
separate from peer JSON-RPC errors and tool results with `isError`.

Schema-derived registration is preferable to parallel handwritten DTO catalogs.
Inspect ACP's binding implementation before choosing reuse: shared generic code
must have a protocol-neutral owner; `fpas-mcp` must not depend on ACP sessions or
the ACP wire schema. Keep compiler syntax and existing ACP names unchanged.

## Affected files and owners

| Action | Path | Purpose |
| --- | --- | --- |
| Create | `crates/fpas-mcp/Cargo.toml`, `crates/fpas-mcp/src/lib.rs`, `crates/fpas-mcp/src/binding/`, `crates/fpas-mcp/src/codec/` | SDK-backed values, schema registration and codec boundary |
| Modify | `Cargo.toml`, `Cargo.lock` | Register the crate and pinned SDK/features |
| Modify | `crates/fpas-std/src/std_units/`, `crates/fpas-std/src/intrinsics.rs` | Publish agreed units and intrinsic dispatch metadata |
| Create/modify | `crates/fpas-bytecode/src/intrinsic/mcp.rs`, `crates/fpas-bytecode/src/intrinsic/mod.rs` | Intrinsic identities and signatures |
| Create/modify | `crates/fpas-sema/src/std_registry/loaded/mcp/`, `crates/fpas-sema/src/std_registry/loaded/mod.rs` | Register protocol types and native APIs |
| Inspect/modify | `crates/fpas-compiler/src/intrinsic_catalog.rs`, `crates/fpas-compiler/src/bytecode/selection/intrinsics.rs` | Wire native calls through existing lowering/catalog mechanisms |
| Generate | `lib/api/Std/Mcp.fpas`, `lib/api/Std/Mcp.Types.fpas`, `lib/api/Std/Mcp.Server.fpas` | Editor declarations from the canonical client/server API |
| Create | `docs/pascal/std/ai/mcp/types.md`, `crates/fpas-mcp/tests/codec.rs`, `tests/stdlib/mcp/types_test.fpas` | Delivered type contract and round-trip coverage |

Finalize any necessary cross-crate extraction paths in AP51.1 before editing.
Apply the standard-library implementation touchpoint checklist, including Cargo
dependencies of every consuming crate and artifact identity when required.

## Acceptance and verification

Test SDK-to-FPAS-to-wire round trips, nested enums, exact numeric bounds,
null/missing/default distinctions, extensible metadata, dynamic schemas and
malformed payloads. Test native naming and reserved-word collisions, forged
handles and unsupported schema forms. Generated declarations must match CLI
checking, editor navigation and typed error handling. Run the owning Rust and
FPAS cases, formatting, build and the applicable workspace checks.
