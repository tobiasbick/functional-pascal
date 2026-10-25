# AP50.1: Lossless schema types and codec

Package: [AP50](README.md). Status: planned; implementation not started.

## Prerequisites

Q55 is recorded in the package README. The pinned SDK and schema releases
are available. Existing native records, data enums, generic fields and error
families provide the representation; no syntax prerequisite is missing.

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
| Create/modify | `crates/fpas-sema/src/std_registry/loaded/acp/`, `loaded/mod.rs` | Focused type and function registration for all public ACP modules |
| Create/modify | `crates/fpas-std/src/native_errors/acp.rs`, native-error registry | Nominal AcpError and preserved host/protocol causes |
| Create | `crates/fpas-vm/src/vm/hosted/acp/values/` | Native-to-FPAS detached layout conversion |
| Generate | `lib/api/Std/Acp.fpas`, `Acp.Types.fpas`, `Acp.Host.fpas`, `Acp.Experimental.fpas`, `Acp.V2.fpas` | Editor declarations under the existing `lib/api/Std/` layout |
| Create | `crates/fpas-acp/tests/codec/`, `tests/stdlib/acp/types_test.fpas` | Independent wire fixtures and public type behavior |
| Create | `docs/pascal/std/ai/acp/types.md` | Delivered value, nullability, unknown-value and precision contracts |

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
