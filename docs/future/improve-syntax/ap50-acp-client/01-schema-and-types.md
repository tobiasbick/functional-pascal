# AP50.1: Lossless schema types and codec

Package: [AP50](README.md). Status: complete.

## Prerequisites

Q55 is recorded in the package README. The pinned SDK and schema releases
are available. Existing native records, data enums, generic fields and error
families provide the representation; no syntax prerequisite is missing.

## Delivered value boundary

`fpas-acp` pins the SDK and derives the complete v1/v2 declaration graphs from
its schema. Generated validator tables instantiate the actual SDK serde types;
the binding contains no handwritten method table or JSON-RPC envelope driver.
Native registration and VM conversion consume the same detached type graph.
Stable values live in `Std.Acp.Types`, v1 extensions in `Std.Acp.Experimental`,
and v2 has its own nominal family. `Std.Acp.Host` aliases the filesystem and
terminal DTOs. The namespace unit declares opaque connection identities.

Each version exposes decode/encode pairs for all seven request, response and
notification roots. Decoding returns a typed Payload and original RawPayload;
encoding checks the root, method, complete value shape and SDK validation.
The connection driver and negotiated protocol state remain AP50.3 onward.

RawJson retains arbitrary number spellings and provides original object
subtrees through `RawJsonMember`, independently of SDK normalization. UInt64
conversions are exact, IDs are distinct strings, and Field distinguishes
Missing, NullValue and replacement. Shared record metadata checks structured
error payloads; ordinary ACP diagnostics omit associated values.

The serde adapter presents finite decimal tokens normally to untagged SDK
numeric enums while preserving oversized arbitrary JSON numbers. Patch overlays
retain SDK MaybeUndefined fields and untagged v1 defaults that JSON Schema
alone does not distinguish.
VM conversion handles generic enum storage erasure using the concrete protocol
graph and validates record/enum identities and layouts before returning a value.

## Deliverable and contract

Implement every stable v1 schema value reachable from the client method and
notification inventory, plus the versioned experimental/v2 type families.
Do not ship a placeholder type or an untyped JSON argument for a known typed
ACP payload. Preserve nominal type identity across imports and compiled units.

Create a small `fpas-acp` Rust crate for SDK-facing types, validation and codec.
It must not depend on compiler or VM crates. The VM adapter translates detached
protocol values into FPAS layouts; shared registration defines those layouts.
Use the pinned SDK structs/enums as the wire contract, not a second handwritten
ACP message dispatcher. Source-module convenience functions may wrap these
native operations without duplicating validation or the protocol parser.

Use the value mapping in [interface.md](interface.md#values-and-handles).
Add `RawJson`, `UInt64` and tri-state field construction/inspection operations
to the ACP modules. They are scoped library types. `Std.Json` and its existing
number representation retain their current behavior.

Enable `serde_json`'s `raw_value` and `arbitrary_precision` in the ACP dependency
graph in addition to insertion-order preservation. Keep arbitrary JSON subtrees
as validated raw JSON through SDK handoff and FPAS conversion. Retain raw input
when needed to preserve fields the SDK normalizes. Never reconstruct `_meta`,
extension parameters or error data through `Std.Json.JsonValue`.

Validate mandatory fields and outbound limits before effects. Follow pinned
schema defaults, invalid-optional-field handling and skipped malformed list
entries on inbound messages. A future enum variant remains unknown data rather
than a fabricated known variant. Unknown fields in incoming payloads remain
available in an accompanying raw payload when the typed schema ignores them.

## Affected files and implementation owners

All paths below are relative to the repository root. Directory entries name a
focused family of new files, split by protocol concern, normally below 500 lines.

| Action | Path | Purpose |
| --- | --- | --- |
| Modify | `Cargo.toml`, `Cargo.lock` | Exact SDK/schema dependencies and JSON feature selection |
| Create | `crates/fpas-acp/Cargo.toml`, `crates/fpas-acp/src/lib.rs` | SDK adapter crate and documented entry point |
| Create | `crates/fpas-acp/src/codec/` | Stable, v2 and experimental payload validation/conversion |
| Create | `crates/fpas-acp/src/value.rs`, `crates/fpas-acp/src/error.rs` | Lossless values and operational errors |
| Create/modify | `crates/fpas-std/src/std_units/symbols/std_symbols/acp.rs`, parent symbol/units modules | Canonical ACP names and hierarchical intrinsic-unit lookup |
| Create/modify | `crates/fpas-sema/src/std_registry/loaded/acp/`, `loaded/mod.rs`, `loaded/native_records.rs`, `loaded/errors.rs` | Focused type and function registration, including shared nested record shapes |
| Create/modify | `crates/fpas-std/src/native_errors/acp.rs`, native-error registry | Nominal AcpError and preserved host/protocol causes |
| Create | `crates/fpas-vm/src/vm/hosted/acp/values/` | Native-to-FPAS detached layout conversion |
| Generate | `lib/api/Std/Acp.fpas`, `Acp.Types.fpas`, `Acp.Host.fpas`, `Acp.Experimental.fpas`, `Acp.V2.fpas` | Editor declarations under the existing `lib/api/Std/` layout |
| Create | `crates/fpas-acp/tests/{binding,codec,schema,method_fixtures,tagged_fixtures}.rs`, `tests/methods/`, `tests/stdlib/acp/` | Independent wire fixtures, complete method coverage and public type behavior |
| Create | `docs/pascal/std/ai/acp/` | Module, codec, nullability, unknown-value and precision contracts |
| Create/modify | `crates/fpas-bytecode/src/intrinsic/acp.rs`, parent intrinsic/export modules, `crates/fpas-compiler/src/intrinsic_catalog.rs`, `intrinsic_catalog/tests.rs` | Value-operation selectors, lowering and canonical route coverage |
| Create/modify | `crates/fpas-std/src/acp/`, `src/native_records.rs`, aggregate/runtime dispatch | Checked detached value operations and shared record shapes |
| Modify | `crates/fpas-std/src/json/parse.rs`, `Cargo.toml` | Preserve existing finite-number behavior under JSON feature unification |
| Create/modify | `crates/fpas-cli/tests/acp_types.rs`, `crates/fpas-vm/src/vm/layouts/native_records.rs` | Compiled-unit reuse and image admission checks |
| Modify | `crates/fpas-cli/src/main_tests/test_suite.rs` | Include public ACP tests in Cargo's themed library suite |
| Modify | `crates/fpas-sema/examples/export_intrinsic_std_api.rs`, supporting `generics.rs`, `documentation.rs`, `qualified_names.rs` | Preserve generic parameters, distinct types, aliases, nested unit references and diagnostic descriptions |
| Modify | `crates/fpas-bytecode/src/value/aggregate.rs`, `display.rs`, affected display fixtures | Apply native ACP payload-display policy while retaining inspectable values |
| Create | `crates/fpas-acp/src/binding/`, `src/normalizers/`, `examples/generate_normalizers.rs` | SDK-derived declarations, lossless detached codec and generated direct SDK validators |
| Modify | `crates/fpas-acp/src/schema.rs`, `src/lib.rs` | Include protocol-level cancellation roots and expose the binding contract |
| Modify | `crates/fpas-sema/Cargo.toml`, `crates/fpas-vm/Cargo.toml` | Consume the shared protocol declarations and detached values |
| Modify | `crates/fpas-compiler/src/object/mod.rs`, API exporter helpers | Retain native DTO layouts and export distinct types and aliases |
| Create/modify | `crates/fpas-vm/src/vm/hosted/acp/`, hosted dispatch, intrinsic ownership metadata | VM conversion and typed payload codecs, without a connection driver |

Also update `crates/fpas-std/src/native_errors.rs`, affected error metadata,
artifact serialization under `crates/fpas-sema/src/interface/` and layout
validation under `crates/fpas-vm/src/vm/layouts/`. The current native error
field catalog has scalar fields and selected native causes. Extend its shared
field-shape description to reference registered record/optional-record payloads
for ProtocolError and RawJson. This is registration/artifact work for existing
language value shapes, not new syntax or ACP-only serialization. Verify this
prerequisite before exposing AcpError.Peer with a structured cause.

## Migration and documentation

There are no ACP consumers to migrate. Inspect shared JSON tests after Cargo
feature unification; retain their documented finite-real conversion behavior.
Register the module names without making a source-defined unit masquerade as
an intrinsic. Generate editor declarations through the existing exporter.
Keep proposed declarations here until implemented; current handbook pages
describe only the types delivered by this work package.

## Acceptance and verification

- Independently specified JSON fixtures cover all required fields and tagged
  variants, optional values, schema defaults and unknown open enums.
- Verify exact `9007199254740993`, signed extrema and
  `18446744073709551615` without conversion through binary64. Reject invalid
  UInt64 digits, negative values and values above the u64 maximum.
- A missing patch field, explicit null and an empty replacement have distinct
  observable results. Preserve nested `_meta`, arbitrary error data, unknown
  fields and extension payloads, including numeric values beyond u64.
- Reject malformed raw JSON, duplicate keys at construction, nonfinite known
  floating-point fields and out-of-domain bounded integers. Preserve valid
  unknown JSON number spellings in RawJson rather than imposing an f64 limit.
- Import every public module, construct and pattern-match the payload families,
  and pass them through an ordinary channel. A reused `.fpascu` retains nominal
  identities and generic field types.
- Type checking rejects a response from the wrong protocol family. Dynamic
  conversion also rejects invalid shapes without panic or partial publication.

Run owning codec/sema/layout tests, API export and declaration checks, existing
JSON regression tests, then the Rust/library checks required by the development
process. These fixtures protect conversion; later packages own protocol state
and real transport behavior.
