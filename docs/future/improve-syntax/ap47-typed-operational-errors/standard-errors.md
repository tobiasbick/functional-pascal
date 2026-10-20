# AP47 standard-library error contract

Status: Host/parsing families delivered in AP47.5; task/network families and
layered consumers delivered in AP47.6. AP47.7 final acceptance remains open.
Owns Q33/Q34 details for [AP47.5](05-host-and-parsing.md) and
[AP47.6](06-tasks-and-network.md). Language rules belong to [AP47.1](01-decision.md).
These are error declarations, not a hierarchy or new reporting records.

## Shared native leaf

Add the intrinsic type-only module Std.Errors with one ordinary native error:

```pascal
public type HostError = error
  Failure(Operation: string; Resources: array of string;
          Domain: Option of string; Code: Option of integer; Detail: string)
    = 'Host operation failed';
end error;
```

Operation is the qualified attempted operation. Resources retains supplied
resource identifiers in parameter order, for example source then target; an
operation without resources uses an empty array. Do not resolve paths, query
resources or recover additional data while constructing/displaying the error.
Domain/Code are both Some when a raw native code exists and both None otherwise.
Use 'win32' or 'errno' for their respective host code spaces; no numeric code
is portable by itself. Detail retains the host explanation without classifying
it by string contents. Native metadata is stored once in this leaf.

HostError names a failed hosted operation, not necessarily an OS error. A host
deadline expiry retains its actual attempted operation/resources with Domain/Code
both None and its detected reason in Detail. Never invent a native code. This
also covers backends that supply a diagnostic but no raw code; a backend-specific
code space must not be mislabeled as errno or win32.

Known native outcomes select a typed outer domain case. Even a classified
failure retains its HostError cause; unknown I/O classifications use the domain's
IoFailure case where provided. Crypto entropy failures use EntropyUnavailable
with that same original leaf. Input validation failures have no
invented OS cause. Internal compiler/VM invariants still fail through runtime
diagnostics. No HostError is synthesized from a panic or task failure report.

Runtime builders resolve the declared nominal layouts from the admitted
executable once, alongside other hosted layouts. Rust adapters construct those
shared layouts; they do not invent variant IDs, fresh nominal types or per-call
string-name lookups. Failure to find the declared layout is an internal image
contract error, never a fallback operational string.
Resolve only families required by that image's used native operations. A program
that never uses Fs must not require FsError merely because another Std function
exists. Preserve the dependency closure of each required native error family.

## Host and parsing cohort

The table lists each complete public family selected by AP47.5. Parameter groups
after the table are part of the contract. Fixed message labels are selected
below; authors do not provide custom formatting functions.

| Module / type | Variants and payloads | Applies to |
| --- | --- | --- |
| Std.Fs.FsError | FileNotFound(Path, Cause: HostError); AccessDenied(Path, Cause: HostError); AlreadyExists(Path, Cause: HostError); NotDirectory(Path, Cause: HostError); IsDirectory(Path, Cause: HostError); NotEmpty(Path, Cause: HostError); ReadOnly(Path, Cause: HostError); IoFailure(Cause: HostError); InvalidArgument(Name, Reason); InvalidPattern(Pattern, Reason); InvalidEncoding(Path, Detail); LimitExceeded(Operation, Limit) | Every currently Result-returning Fs operation, including globbing, metadata and temp operations |
| Std.Proc.ProcError | StartFailed(Cause: HostError); WaitFailed(Cause: HostError); InvalidArgument(Name, Reason); IoFailure(Cause: HostError) | Run, RunCapture, CurrentExecutable |
| Std.Parse.ParseError | InvalidSyntax(Target, Detail); OutOfRange(Target, Detail) | Int, HexInt, Real, Bool |
| Std.Json.JsonError | InvalidSyntax(Line, Column, Detail) | Json.Parse |
| Std.Toml.TomlError | InvalidSyntax(Line, Column, Detail) | Toml.Parse |
| Std.Crypto.CryptoError | InvalidArgument(Name, Reason); EntropyUnavailable(Cause: HostError) | RandomBytes, RandomInt |
| Std.Json.Fields.FieldError | ParseFailure(Cause: JsonError); ExpectedObject; MissingField(Name); WrongType(Name, Expected); InvalidValue(Name, Reason); UnknownField(Name) | All Result-returning public JSON field helpers |
| Std.Toml.Fields.FieldError | ParseFailure(Cause: TomlError); ExpectedTable; MissingField(Name); WrongType(Name, Expected); InvalidValue(Name, Reason); UnknownField(Name) | All Result-returning public TOML field helpers |

Unqualified Name, Reason, Path, Pattern, Operation, Target, Expected and Detail
payloads above are string; Limit is integer; Line and Column are Option of
integer. All Cause fields refer to the indicated concrete native error family,
using imported qualifiers in actual declarations. Type names belong to their
defining module; aliases do not produce another type.

Preserve existing successes and validation boundaries. Nonzero child exit status
remains a successful Run/RunCapture outcome. Parsing keeps existing grammar,
numeric overflow/underflow rejection and accepted boolean spellings. Empty text,
malformed grammar and invalid tokens are InvalidSyntax; valid numeric syntax
outside the supported finite/range domain is OutOfRange. Json/Toml use parser
line/column data when supplied, otherwise None; do not fabricate positions.
Field helper issue names are checked structurally at their owner; unknown-field
selection follows the existing iteration order rather than reporting a new
unordered set. Fs limits retain their current values and ownership behavior.

Native classification uses structured Rust/backend categories at the failure
site. Standard NotFound, PermissionDenied, AlreadyExists, NotADirectory,
IsADirectory and DirectoryNotEmpty categories map to their matching Fs cases;
read-only filesystems map to ReadOnly where the backend identifies that category.
Unavailable classifications use IoFailure, retaining the HostError. Do not guess
which path failed in a multi-resource operation: use a known failing Path when
the owner identifies it, otherwise the attempted primary path, while HostError
retains every supplied resource. No preflight Exists/access test replaces the
actual operation.

Use exact message labels 'File not found' and 'Access denied' for those Fs cases.
Other labels are fixed English descriptions of the selected variant, not the
host diagnostic text. Wording changes never select or change a variant.

## Task and networking cohort

AP47.6 selects the following families. Operation, Name, Reason, Detail, Address,
Field and Scheme are string; Limit, Offset and Status are integer.

| Module / type | Variants and payloads | Applies to |
| --- | --- | --- |
| Std.Tasks.TaskError | Closed(Operation); Timeout(Operation); Cancelled(Operation); InvalidArgument(Name, Reason); InvalidGroup(Operation) | Channel Send/Receive/Try operations, timeout/cancellation waits, CloseTaskGroupWithTimeout and matching selection callback Outcomes |
| Std.Net.NetError | Closed(Operation); Timeout(Cause: HostError); Cancelled(Operation); InvalidArgument(Name, Reason); ResolveFailed(Cause: HostError); ConnectionRefused(Cause: HostError); ConnectionLost(Cause: HostError); AddressInUse(Cause: HostError); AccessDenied(Cause: HostError); TlsFailure(Detail); IoFailure(Cause: HostError) | Every Result-returning native Net operation |
| Std.Net.Uri.UriError | InvalidSyntax(Detail); UnsupportedScheme(Scheme); InvalidPort(Detail) | URI parsing and related private fallible helpers |
| Std.Net.Utf8.Utf8Error | InvalidByte(Offset, Detail); IncompleteSequence(Offset) | Decode, including its existing truncated-sequence check; existing nonfallible Encode remains nonfallible |
| Std.Server.ServerError | InvalidArgument(Name, Reason); InvalidState(Operation); ListenerFailure(Cause: NetError); SignalFailure(Cause: HostError); Timeout(Operation); IoFailure(Cause: HostError) | Result-returning lifetime/signal/listener/shutdown operations |
| Std.Http.Types.HttpError | Transport(Cause: NetError); Address(Cause: UriError); Encoding(Cause: Utf8Error); InvalidMessage(Detail); InvalidArgument(Name, Reason); LimitExceeded(Operation, Limit); UnexpectedEnd(Operation); RedirectFailure(Detail) | Public and fallible internal HTTP client/server, streaming, framing, redirects and SSE helpers |

Std.Http re-exports HttpError using its existing public alias style. Defining the
family in the existing leaf Types module avoids an import cycle with Client or
Server. Reuse that family across Http.*; do not create one incompatible error
for each helper. A process result, HTTP non-success status, EOF, a failed task
and the failure of a wait operation remain distinct concepts:

- HTTP non-2xx responses remain successful Response/StreamResponse data.
- A closed channel retains current draining and then Closed behavior. A full or
  empty nonblocking channel retains its existing false/None success outcomes.
- Negative/otherwise invalid timeout arguments are InvalidArgument. Timeout and
  cancellation remain distinct operational cases and preserve current race
  precedence. Timed-out work is still owned; this plan does not kill it.
- Net Timeout wraps HostError for backend timeouts and host deadline expiry,
  retaining native metadata when present and using None when absent. Task/Server
  policy timeouts retain their own Operation payload; they do not invent a host
  failure when no host operation was involved.
- Wait on a failed child and existing TaskFailure/TaskFailureKind reporting keep
  their established runtime semantics. Do not turn caught panic or failure-capture
  reports into TaskError. The public TaskFailure record is not replaced merely
  because its name contains Failure.
- Existing Net EOF/empty byte results remain successes where documented. No new
  EOF variant changes an existing successful end-of-input contract. Reset/refused/
  unavailable resources are classified at their owner, not from text.
- TLS verification/protocol failures are TlsFailure; native transport failures
  inside TLS retain the corresponding Net case with HostError. Keep original
  diagnostic Detail when the backend has no more specific portable category.
- Utf8.Decode uses InvalidByte for byte-domain and malformed-sequence failures,
  and IncompleteSequence at its existing truncated-sequence check. Offsets retain
  the current byte-offset convention. Std.Net.Utf8 has no incremental Finish API;
  do not add one for this migration or replace the decoding algorithm.
- Std.Http.Sse owns the incremental decoder and its Feed/Finish operations. Both
  return their existing success types with HttpError. Where their existing line
  processing calls Utf8.Decode, map its error to HttpError.Encoding(Cause),
  including a truncated final line processed by Finish. Preserve buffering,
  byte-offset origin and decoder end-state behavior; do not report incompleteness
  merely because a Feed chunk ends inside an otherwise valid sequence.

### Fixed labels

Every selected variant has one static label. Use the family default below except
for its listed overrides; these are literal declaration metadata, not a runtime
name-splitting algorithm. Wording is for people and never drives classification.

| Family | Default label | Overrides |
| --- | --- | --- |
| FsError | Filesystem operation failed | FileNotFound: File not found; AccessDenied: Access denied |
| ProcError | Process operation failed | None |
| ParseError | Invalid value syntax | OutOfRange: Value outside supported range |
| JsonError | Invalid JSON | None |
| TomlError | Invalid TOML | None |
| CryptoError | Secure random operation failed | None |
| Json.Fields.FieldError | Invalid JSON field | None |
| Toml.Fields.FieldError | Invalid TOML field | None |
| TaskError | Task operation failed | Closed: Channel is closed; Timeout: Operation timed out; Cancelled: Operation cancelled |
| NetError | Network operation failed | Closed: Connection is closed; Timeout: Operation timed out; Cancelled: Operation cancelled |
| UriError | Invalid URI | None |
| Utf8Error | Invalid UTF-8 | None |
| ServerError | Server operation failed | Timeout: Operation timed out |
| HttpError | HTTP operation failed | None |

No new ExpectedStatus error, channel policy, process timeout, file handle,
network protocol or decoder capability is included. Existing resource shutdown,
partial-I/O and worker behavior stays as documented.

## Source-boundary mappings and migration inventory

Within each public domain, private helpers return the same family or a concrete
lower-level family that is explicitly mapped before propagation. Use MapError
with a one-Cause constructor for pure wrappers, or an ordinary closure to add
context. Recovery still uses OrElse/case. For semantic protocol errors use the
local declared variant at the detection site, not a fake lower-level cause.

| Caller | Selected mapping |
| --- | --- |
| Application ConfigError | FileFailure(Cause: Fs.FsError); MissingSetting(Name); InvalidSetting(Name, Reason); no registered formatter |
| Json.Fields / Toml.Fields | Native parse errors -> ParseFailure; structural field failures -> own variants |
| HTTP transport/address/UTF-8 paths | NetError -> Transport; UriError -> Address; Utf8Error -> Encoding |
| Http.Sse.Feed / Finish | Existing Utf8.Decode failures -> HttpError.Encoding(Cause); SSE protocol/limit/state failures stay in their selected HttpError cases |
| Server listener ownership | NetError -> ListenerFailure; backend signal/native failures -> SignalFailure or IoFailure |
| HTTP integer/chunk grammar | Detect InvalidMessage at the protocol parser; do not expose a primitive ParseError as transport failure |
| Application or SDK that intentionally exposes Error(string) | Explicitly map Cause.Message() at that owned public boundary; keep its documented contract, not an implicit cast |

Native catalogs are owned by sema/std registry and generated declarations under
lib/api/Std; source error families live with their existing source modules.
Generate native declarations rather than editing generated APIs by hand.

Before each cohort implementation, inventory all Result signatures and callback
parameters in its generated and source modules, including multiline declarations
and imported aliases. Apply the table to every matching declaration, not only
to the examples named above. No selected public return or nested callback
Outcome may retain a string error after its cohort is complete. Existing
infallible operations and unselected return shapes do not become Result.

The gap review excluded Std.Ai.* and did not select a TUI error redesign. AP47
therefore performs required caller migrations in those modules and apps but
does not independently change their published domain contracts. These domains
may intentionally retain string errors at explicit owned boundaries. This is
not an alternate typed/legacy wrapper family for any selected function.

## Completion evidence

Public catalog and source declarations agree on exact variants, payload names
and types, success types and callback Outcomes. Selected conditions are tested
by constructing/detecting the actual condition, not by matching `.Message()`.
Use native-category fixtures for portable classification and platform-specific
integration for real error/code preservation. Retain true diagnostics tests
where they own wording. Admission/invariant failures are never accepted as
IoFailure fallbacks. No core taxonomy or mapping question remains delegated to
future implementation; adding capabilities beyond these current operations
requires its own design.
