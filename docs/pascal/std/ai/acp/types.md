# `Std.Acp.Types`

Detached ACP value types and checked conversions. Import
`uses Std.Acp.Types as AcpTypes;`. These values can be copied and sent through
ordinary typed channels. The unit supplies stable v1 protocol DTOs, nominal
string IDs and method-aware payload codecs. Experimental v1 payloads live in
`Std.Acp.Experimental`; v2 has its own `Std.Acp.V2` nominal family.

## Values

| Kind | Name | Meaning |
| --- | --- | --- |
| type | `RawJson` | Record with `Text: string`, containing exactly one validated JSON value. |
| type | `UInt64` | Record with canonical decimal `Digits: string`, covering `0..18446744073709551615`. |
| type | `Field of T` | Data enum with `Missing`, `NullValue`, and `Value(Value: T)`. |
| type | `ProtocolError` | Record with `Code: integer`, `Message: string`, and `Data: Option of RawJson`. |
| type | `AcpError` | Nominal operational error, including structured `Peer(Protocol: ProtocolError)` and `Host(Cause: HostError)`. |

`Field.Missing` means no patch was supplied, `Field.NullValue` means explicitly
cleared, and `Field.Value([])` means an empty replacement. Use enum constructors
and `case` patterns to construct and inspect these states.

## Conversions

| Kind | Name | Meaning |
| --- | --- | --- |
| function | `ParseRawJson(Text: string): Result of (RawJson, AcpError)` | Validate one JSON value and retain its text. |
| function | `RawJsonText(Value: RawJson): Result of (string, AcpError)` | Revalidate a record and return its JSON text. |
| function | `RawJsonMember(Value: RawJson; Name: string): Result of (Option of RawJson, AcpError)` | Read an original object subtree; a missing member returns None, explicit null returns Some of raw null. |
| function | `ParseUInt64(Text: string): Result of (UInt64, AcpError)` | Parse ASCII decimal digits, normalizing leading zeroes. |
| function | `UInt64Text(Value: UInt64): Result of (string, AcpError)` | Validate canonical digits and return decimal text. |
| function | `UInt64FromInteger(Value: integer): Result of (UInt64, AcpError)` | Reject negative integers. |
| function | `UInt64ToInteger(Value: UInt64): Result of (integer, AcpError)` | Reject values above `9223372036854775807`. |

No conversion passes through `real` or `Std.Json.JsonValue`. Raw JSON retains
object order, nested unknown fields, and numeric spelling, including `-0`,
integers beyond u64 and values such as `1e400`. Insignificant whitespace outside
the JSON value is removed. Duplicate decoded member names, unpaired Unicode
surrogates, malformed syntax, and multiple values are rejected. The value
boundary allows at most 64 MiB of UTF-8 input and 128 nested objects/arrays.
Connection message limits, when implemented, apply separately.

## Typed payload codecs

For each root `ClientRequest`, `AgentResponse`, `ClientNotification`,
`AgentRequest`, `ClientResponse`, `AgentNotification`, and
`ProtocolLevelNotification`, the unit supplies two functions:

| Kind | Name | Meaning |
| --- | --- | --- |
| function | `DecodeClientRequest(Method: string; Payload: RawJson): Result of (DecodedClientRequest, AcpError)` | Validate a request body and retain its original JSON. |
| function | `EncodeClientRequest(Method: string; Payload: ClientRequest): Result of (RawJson, AcpError)` | Validate a typed request and encode its body. |

Replace `ClientRequest` with the appropriate root in both names and types.
Each `Decoded*` record contains `Payload`, the typed root enum, and
`RawPayload`, the original input including fields ignored by the typed schema.
The method is the wire spelling, for example `session/prompt`. Encoding checks
that the method agrees with the chosen enum variant. These functions convert
a payload body; they do not send messages or parse a JSON-RPC envelope.

Root enum variants have the SDK payload names, for example
`ClientRequest.PromptRequest(Payload: PromptRequest)`. Supporting tagged enums
have named variants with a `Payload` record when needed; records with flattened
unions have a typed `Kind` field. String IDs such as `SessionId`, `ToolCallId`
and `TerminalId` are distinct strings with explicit constructors.

Optional values use `Option`. Patch fields use `Field`, including the SDK's
missing/null distinction. SDK defaults, invalid optional values and explicitly
skippable malformed list entries apply on decoding. Mandatory malformed fields
fail. Closed known fields are validated again on encoding, including integer
bounds and finite reals.

Open string enums have `Unknown(Value: string)`. Future tagged variants use
`Unknown(Tag: string; Payload: RawJson)` and retain their entire original object.
An unknown variant cannot impersonate a known tag. Arbitrary JSON, including
metadata, MCP data and extension bodies, remains `RawJson`. Extension payloads
retain the complete underscore-prefixed method in `Method`; their raw body
is `Params`, or `ResultValue` for a response.

All current constructors and fields are in the generated
[editor declarations](../../../../../lib/api/Std/Acp.Types.fpas). The declarations
derive from the pinned SDK schema and its method metadata; the SDK's serde
types validate payloads. No host effect occurs during conversion.

These records have public fields. Constructing a record directly does not
validate its contents; every native operation revalidates it. Direct `UInt64`
records must already be canonical: `Digits = '00'` is rejected. Protocol error
codes use signed JSON-RPC i32 values at the wire boundary; the detached record
uses `integer` and does not itself send a response.

## Errors

Conversions return `InvalidArgument(Name, Reason)` for invalid unsigned values,
`InvalidMessage(Line, Column, Reason)` for invalid JSON, and
`LimitExceeded(Name, Limit)` for oversized input. Line and column are optional
parser positions. Ordinary conversion diagnostics contain structural reasons,
not input bodies or object keys.

The family also declares `UnsupportedCapability(Name)`,
`UnsupportedVersion(Version)`, `Busy(Operation)`, `Closed(Operation)`,
`Timeout(Operation)`, `Cancelled(Operation)`, `Host(Cause)`, and `Peer(Protocol)`
for subsequent connection operations. Ordinary ACP error display, including
`.Message()` and failed `.Unwrap()`, shows the variant and fixed label without
associated payloads or nested host causes. Explicit fields retain a peer's
message and data; code that displays them chooses what to disclose.

## Implementation (contributors)

- [`fpas-acp`](../../../../../crates/fpas-acp/src/lib.rs) owns the pinned SDK,
  raw JSON validation and exact unsigned integers.
- [Native type registration](../../../../../crates/fpas-sema/src/std_registry/loaded/acp/mod.rs)
  defines the FPAS declarations; [value operations](../../../../../crates/fpas-runtime/src/stdlib/acp/mod.rs)
  revalidate detached records at native boundaries.
- [Native record admission](../../../../../crates/fpas-runtime/src/execution/vm/layouts/native_records.rs)
  verifies nested record shapes in executable images.

## See also

- [AI clients](../README.md)
- [Ordinary channels](../../concurrency/task.md)
- [Client overview and lifecycle](README.md)
