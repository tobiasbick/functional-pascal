# AP54: Native byte buffers and UTF-8 integration

Status: Planned; the contract is complete. Effort: L.
Completion is tracked only in the [central plan](../README.md#package-status).
This package records planning work; implementation is not yet authorized.

## Goal

Deliver one managed, contiguous byte representation under `Std.Bytes`, including
UTF-8 conversion and integration with existing binary APIs. Ordinary FPAS code
continues to calculate with `integer` and `real`; storing a byte does not require
a new scalar type. A native read, codec or random-byte producer should deliver
byte storage without constructing one tagged VM value per byte.

This package owns the selected byte-buffer portion of
[FPAS 2 section 22](../../fpas2-language-directions.md#22-packed-numeric-data-and-binary-buffers).
It does not implement the entire packed-data proposal.

## Current behavior

| Boundary | Current representation and owner |
| --- | --- |
| UTF-8 | `Std.Text.Utf8.Encode` produces `array of integer`; `Decode` validates that array. See [the reference](../../../pascal/std/network/utf8.md) and [source unit](../../../../lib/Std/Text/Utf8.fpas). |
| Files | Whole-file and handle byte reads/writes use integer arrays, with bounded transfers and partial-write rules. See [file streaming](../../../pascal/std/host/fs-streams.md). |
| TCP/TLS | Byte receive/send operations use integer arrays. See [sockets](../../../pascal/std/network/net.md). |
| Processes | Interactive pipe reads, writes and selectable callbacks use integer arrays; pending and EOF have distinct representations. See [processes](../../../pascal/std/host/proc.md). |
| HTTP and URI | Bodies, streams, SSE fragments and private wire/percent helpers retain integer-array bytes. See [HTTP](../../../pascal/std/network/http.md) and [URI source units](../../../../lib/Std/Net/Uri). |
| Binary-to-text codecs | Base64, Base64url and hex encode/decode integer arrays through `Std.Encoding`. See [encoding](../../../pascal/std/text/encoding.md). |
| Randomness | `Std.Crypto.RandomBytes` returns an integer array. See [cryptography](../../../pascal/std/cryptography/crypto.md). |
| Runtime | `Value::Array` holds tagged elements; managed strings, aggregate ownership, task transfer and native-job admission already have their own lifecycle rules. See [values](../../../../crates/fpas-runtime/src/value/mod.rs) and [native-job admission](../../../../crates/fpas-runtime/src/execution/vm/host_value/admission.rs). |

For example, a file chunk currently passes through an integer array before
`Utf8.Decode` can validate it. Under the proposed contract it stays in the same
byte representation from the native read to text decoding. An ordinary
`array of integer`, such as tokenizer token IDs, remains an ordinary array.

## Agreed scope and boundaries

- One public general byte-buffer type, `Std.Bytes.ByteBuffer`, under the already
  reserved `Std.Bytes`. Import aliases use the normal unit rules, for example
  `Bytes.ByteBuffer` after importing `Std.Bytes as Bytes`.
- UTF-8 conversion stays in the existing `Std.Text.Utf8`; Base64 and hex stay in
  `Std.Encoding`. The [namespace specification](../../../specs/std-namespaces.md)
  owns these allocations. No new public child unit is required by this plan.
- Include construction, byte access, append, copy, fill, slices and capacity
  management, with bulk native operations rather than per-byte FPAS loops.
- Include `string` to UTF-8 bytes, efficient text append, and strict decoding
  back to `string`. Malformed UTF-8 is typed error data, without silent repair.
- Buffer positions and lengths count bytes. Existing string operations retain
  their Unicode scalar rules. Embedded zero bytes are ordinary data.
- Buffers have value semantics with copy-on-write storage. Assignment, arguments
  and slices do not create shared mutable handles. Mutation uses explicit
  `InPlace` operations on writable storage under the existing `var` rules;
  changing a copy must not change the original. Equality compares byte contents,
  not storage identity or capacity.
- Invalid direct indices follow existing array runtime-failure behavior.
  Expected conversion and limit failures are typed results. The
  [AP54.1 contract](01-contract.md) fixes every signature, error payload, bounds
  rule and result form; it is the authoritative API detail reference.
- The initial maximum length/capacity is 256 MiB per buffer, independently of
  existing I/O limits. Growth is geometric, newly exposed bytes are zeroed and
  capacity release is explicit through `TrimCapacityInPlace`.
- Proper subrange slices copy their selected bytes; nonempty full-range slices
  may share immutable backing. There are no borrowed views or hidden retention
  of a larger parent by a small slice. Buffers satisfy `Equatable` and can be
  dictionary keys; they do not provide ordering or a public hash operation.
- Migrate existing binary APIs and repository consumers to one canonical byte
  representation. Provide explicit integer-array conversion for interoperability,
  without implicit coercions or duplicate array/buffer I/O API families.
- Include task/channel transfer, asynchronous native ownership, cancellation,
  debugger execution and allocation accounting. Tasks and native jobs retain
  stable byte values; memory is released automatically when its legitimate
  owners disappear. There is no public `Close` operation for a buffer.
- Keep FPAS syntax unchanged. Any implementation that actually requires a syntax
  change needs separate explicit user approval before that change is made.

Excluded: separate `StringBuffer` or `Utf8Buffer` types; a public incremental text
decoder or new text-builder family; additional character encodings; packed
arrays and fixed-width scalar types; `money` and `decimal`; public raw pointers,
borrowed spans or a C ABI; binary number-layout helpers; general stream
abstractions; lexical cleanup syntax; and specialized pixel, vertex, index or
audio buffers. Graphics/audio may reuse the storage later under their own plans.
This AP does not implement GPU upload, media codecs or the general FFI.

## Decisions

| ID | Question | Recorded decision | Affected work packages |
| --- | --- | --- | --- |
| Q72 | Select one native byte-buffer foundation with UTF-8 integration and migration of existing binary APIs? | Agreed. Include text transitions; do not introduce separate string/UTF-8 buffers or new scalar numeric types. Create a dedicated AP; implementation authorization remains separate. | AP54.1–AP54.4 |
| Q73 | What are buffer assignment, mutation, slicing, equality and task-transfer semantics? | Agreed: copy-on-write values, explicit `InPlace` mutation, byte-content equality and stable transferred values. Proper subranges copy, nonempty full-range slices may share. Equatable/dictionary keys and Printable are supported; Comparable, Numeric, public Hash and views are excluded. | AP54.1–AP54.4 |
| Q74 | What is the exact public operation set, error contract and allocation policy? | Agreed: the [single API table](01-contract.md#exact-byte-api), explicit array conversion, array-style range failures, typed BufferError results, atomic buffer mutation, a fixed 256 MiB maximum, geometric growth, zero-initialized new bytes and explicit capacity trimming. No alias/legacy API family. | AP54.1–AP54.4 |
| Q75 | What are the exact UTF-8 conversion and append contracts? | Agreed: the [UTF-8 signatures and errors](01-contract.md#utf-8-api-and-errors), strict Decode, typed Encode/AppendInPlace results, byte offsets at the first invalid/incomplete sequence start, preserved NUL/BOM contents, and no repair or extra text-buffer type. | AP54.1, AP54.3, AP54.4 |
| Q76 | How do native ownership, canonical API migration and artifacts expose the buffer? | Agreed: last-owner automatic release without Close, stable native/task inputs, canonical byte-leaf replacement preserving host-I/O rules, full compiled/debug value support, complete recording/restoration and length plus a bounded hex preview. The [migration/artifact contract](01-contract.md#migration-and-artifacts) fixes the details. | AP54.1–AP54.4 |

Keep the answers here as the single decision record and reflect them in dependent
work packages. The user accepted the recommendations recorded in Q73–Q76; the
signature tables and error payloads in AP54.1 apply the delegated design authority
within that scope. Do not reopen those choices as pending alternatives. FPAS
syntax changes still need explicit user approval. Planning approval does not start
implementation.

## Open decisions

None within the selected contract. AP54.1 records the complete API, errors,
limits, slice/ownership rules, UTF-8 behavior and artifact expectations.

Backing container, module splits, accounting hooks and fast paths remain ordinary
delegated implementation choices. They must honor the contract rather than add
public alternatives. Any required scope or FPAS syntax change needs user input;
none is currently required by this design. Implementation has not started.

## Dependencies and work packages

Prerequisites are existing facilities, not new APs: current namespace allocation,
managed runtime ownership and GC, uniform first-class operations, and the
[worker-releasing host-I/O contract](../../fpas2-language-directions.md#14-host-io-that-releases-vm-workers).
They are present on the selected `codex/std-namespaces` integration branch.
Choose the implementation branch under the [development process](../development-process.md#dependencies-and-branches)
when implementation is authorized; creating this plan does not switch branches.

- [x] [AP54.1: Byte-buffer contract](01-contract.md)
- [ ] [AP54.2: Implement managed byte storage and core operations](02-native-storage.md)
- [ ] [AP54.3: Integrate UTF-8 and migrate canonical binary APIs](03-utf8-and-migration.md)
- [ ] [AP54.4: Verify lifecycle, integration and representation costs](04-verification.md)

Execution order is AP54.1 → AP54.2 → AP54.3 → AP54.4. Each implementation part
includes its own tests and current documentation. AP54.3 deliberately combines
signature replacement and all affected callers so it leaves a usable tree,
without temporary public compatibility APIs. AP54.4 adds distinct cross-boundary
evidence; it is not permission to postpone lifecycle tests from AP54.2/AP54.3.

This is the only work-package checklist for AP54. Completion does not imply a
commit, push or merge; those actions require their own user authorization.

## Acceptance

A program can construct and bulk-fill a buffer, append UTF-8 text, pass it through
a task/channel, round-trip it through file and process/socket byte APIs, and
strictly decode the resulting bytes without an integer-array conversion at each
handoff. Malformed UTF-8 stays available as binary data and produces the agreed
typed error only when text decoding is requested.

All binary API families above use the same type. Transfer limits, partial
progress, EOF/pending distinctions, cancellation, cleanup, protocol parsing and
text contents preserve their documented behavior except for explicitly agreed
API replacements. Token-ID arrays and unrelated integer collections are unchanged.

Buffer storage is reclaimed when the last legitimate owner disappears, including
through cycles, failed calls, abandoned selections and undelivered native results.
Allocation evidence includes backing bytes and capacity, not just VM wrapper
counts or process RSS. There is no buffer-specific retention pool. Compiled
interfaces, debugger values and complete recording/restoration support the type;
a preview never replaces or truncates stored value data.

Use measurements to report storage/copy and runtime costs, not to promise a speed
gain or impose timing-based tests. Store raw evidence under `.temp-data/`, never
as benchmark entries in `history.md`.

## Documentation and regression owners

- Current handbook and generated declarations: update only with implementation;
  owners are listed in AP54.2/AP54.3 and the
  [Std touchpoints](../../../pascal/std/README.md#shared-implementation-touchpoints).
- Runtime storage, roots, transfer, collectors and native-job boundaries: the
  existing `crates/fpas-runtime/src/value/` and `execution/vm/` owners.
- Contract tests: owning crate tests and existing `tests/stdlib/` families;
  add buffer-specific FPAS and native allocation regressions where missing.
- Graphics and audio remain consumers of a later plan, rather than prerequisites
  or acceptance targets for this byte-buffer package.
- Cross-package checks: [shared constraints](../shared-constraints.md).
