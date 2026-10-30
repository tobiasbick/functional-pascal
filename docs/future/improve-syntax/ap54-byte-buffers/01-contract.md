# AP54.1: Byte-buffer contract

Package: [AP54](README.md). Status: Complete. The accepted contract below is
implemented by AP54.2 and AP54.3. The package README owns the decision record
and completion checklist.

## Public model

`Std.Bytes` exports the native managed value type `ByteBuffer` and nominal error
family `BufferError`. The tables use `uses Std.Bytes as Bytes;`. Every buffer
contains arbitrary bytes; scalar reads return existing `integer` values in
`0..255`. Assignment shares immutable storage until a writer needs independent
storage. There are no shared-mutable buffer handles, pointers, views or `Close`.

Factories and array import belong to `Bytes`. Instance operations have only the
receiver form: do not add free-routine aliases. Text conversion belongs only to
`Std.Text.Utf8`. Every operation has one case-insensitive name and one signature;
all arguments are required. Existing positional/fully named argument rules apply.
Byte operations have no `Empty`, `Clone`, `Clear`, `Append`, `Extend`, `AddRange`
or `Try...` aliases. Assignment copies a value; `Create(0)` constructs an empty
buffer. The combined text operation Utf8.AppendInPlace has its own encoding role.

Value operations preserve their receiver; an `InPlace` operation changes selected
writable caller storage. Both retain their different existing FPAS roles. Query
operations do not allocate buffer storage. Operations that can fail recoverably
return `result`; there are no parallel throwing/result-returning variants.

## Exact byte API

### Factories, conversion and queries

| Operation | Result |
| --- | --- |
| `Bytes.Create(Length: integer)` | `result of (Bytes.ByteBuffer, Bytes.BufferError)`; zero-initialized bytes |
| `Bytes.FromArray(Values: array of integer)` | `result of (Bytes.ByteBuffer, Bytes.BufferError)`; validate then copy |
| `Buffer.ToArray()` | `result of (array of integer, Bytes.BufferError)`; independent integer elements |
| `Buffer.Length()` | `integer`; logical byte count |
| `Buffer.IsEmpty()` | `boolean`; whether length is zero |
| `Buffer.Capacity()` | `integer`; accessible backing capacity, excluding allocator bookkeeping |
| `Buffer[Index]` | `integer`; checked, read-only byte slot |

Byte slots are read-only designators, like string scalar slots. Use `SetAt` or
`SetAtInPlace` below; do not also introduce indexed assignment or a separate Get
operation. Existing `for-in` iterates byte integers in order over a value
snapshot, with the same writable-source protection as arrays. It adds no new
loop syntax or iterator API.

### Value transformations

Every operation below returns `result of (Bytes.ByteBuffer, Bytes.BufferError)`.
The receiver remains unchanged. Transformations are bulk native operations,
without executing an FPAS call for each byte.

| Operation | Meaning |
| --- | --- |
| `Buffer.Slice(Start: integer; Len: integer)` | Selected checked byte range |
| `Buffer.Concat(B: Bytes.ByteBuffer)` | Receiver followed by all bytes of B; matches array Concat's argument label |
| `Buffer.SetAt(Index: integer; Value: integer)` | Replace one byte |
| `Buffer.Fill(Value: integer; Start: integer; Len: integer)` | Replace a range with repeated Value; length stays unchanged |
| `Buffer.CopyFrom(Source: Bytes.ByteBuffer; SourceStart: integer; TargetStart: integer; Len: integer)` | Overwrite an existing target range with the selected source bytes; length stays unchanged |
| `Buffer.Resize(Length: integer)` | Change logical length; preserve the prefix and initialize newly exposed bytes to zero |

### Explicit mutation and capacity

| Operation | Result |
| --- | --- |
| `Buffer.SliceInPlace(Start: integer; Len: integer)` | `result of (boolean, Bytes.BufferError)` |
| `Buffer.ConcatInPlace(B: Bytes.ByteBuffer)` | `result of (boolean, Bytes.BufferError)` |
| `Buffer.SetAtInPlace(Index: integer; Value: integer)` | `result of (boolean, Bytes.BufferError)` |
| `Buffer.FillInPlace(Value: integer; Start: integer; Len: integer)` | `result of (boolean, Bytes.BufferError)` |
| `Buffer.CopyFromInPlace(Source: Bytes.ByteBuffer; SourceStart: integer; TargetStart: integer; Len: integer)` | `result of (boolean, Bytes.BufferError)` |
| `Buffer.ResizeInPlace(Length: integer)` | `result of (boolean, Bytes.BufferError)` |
| `Buffer.ReserveInPlace(Capacity: integer)` | `result of (boolean, Bytes.BufferError)` |
| `Buffer.TrimCapacityInPlace()` | `result of (boolean, Bytes.BufferError)` |

Successful InPlace calls return `Ok(true)`, including a no-op; `false` is not a
second success/failure channel. This buffer API explicitly exposes recoverable
allocation/limit failures. It does not change existing array/dictionary/string
InPlace signatures, which generally return no value.

Native receiver mutation uses the existing writable-storage rules, without an
explicit receiver `var` marker. Constants and computed receivers are rejected.
Native InPlace operations cannot bind a receiver by value or cross `go`; value
operations/factories remain first-class under the existing rules. Ordinary
unit functions with explicit `var` arguments, such as UTF-8 append, retain that
marker. Do not add a second free call form for native receiver operations.

## Bounds, errors and commit rules

Direct reads and all explicit ranges use existing array-style checked bounds.
Use `0 <= Start <= Length` and `0 <= Len <= Length - Start`; a zero-length range
at the end is valid. A direct index instead requires `0 <= Index < Length`.
Negative lengths/capacities, invalid indices and invalid ranges are programmer
errors reported through the existing array/range runtime-failure mechanism.
Validate ranges without overflowing `Start + Len`.

`BufferError` has exactly these variants; every payload field is an `integer`:

| Variant | Meaning |
| --- | --- |
| `InvalidByte(Offset, Value)` | Value is outside `0..255`; FromArray names the first invalid input element, SetAt names Index, and Fill names Start |
| `LimitExceeded(Limit)` | A nonnegative requested byte length/capacity exceeds the buffer maximum; Limit is `268435456` |
| `AllocationFailed(RequestedBytes)` | A recoverable native allocation request failed; RequestedBytes is that request's data-storage byte count |

Check byte values even for an empty fill range. An invalid fill value reports
Start, including when Start is the valid end position. FromArray validates
elements in order. Constructors and growth/capacity requests preflight their
nonnegative requested size and buffer limit before scanning bytes or allocating.
Reads, slices and range overwrites validate their existing ranges first, then
scalar byte values, then any required allocation. Thus an invalid slice range
is a runtime range failure even if its supplied Len also exceeds the buffer
maximum. Ordinary argument expressions still evaluate once in written order.

Check concatenation/growth against the remaining allowed length before adding
sizes; a request beyond the maximum returns LimitExceeded, not an integer
overflow or wrapped allocation. ToArray checks expanded tagged-array storage
arithmetic and reports recoverable allocation failure rather than confusing
array storage bytes with the logical byte-buffer limit.

On Error or a failed operation's range/runtime check, do not commit a new target
value or capacity. Completed argument side effects are not rolled back. Snapshot
Source before writes so overlapping CopyFrom and self-Concat use original input
bytes. InPlace root/index evaluation and the timing of argument writes follow
the existing collection-mutation rules.

## Size, growth and retention

The fixed initial maximum is **268435456 bytes (256 MiB)** for both logical
length and accessible capacity. It is a central library policy, not a per-call
option or a new CLI flag. Existing I/O transfer and protocol limits remain
independent and unchanged. The invariant is `0 <= Length <= Capacity <= Limit`.

Create and FromArray start with capacity equal to length. An empty new value has
zero capacity. UTF-8 Encode starts with capacity equal to its encoded byte length.
Native I/O/codec producers may adopt their owned result allocation and report
its accessible capacity, subject to the same invariant; they need not copy it
just to remove unused capacity. When automatic growth is necessary, start from
the greater of current capacity and 64 bytes,
double until the requested length fits, and cap capacity at the maximum. The
new byte region is initialized to zero before it becomes readable. Value
transformations follow the corresponding mutation's rules on an independent
copy; capacity never participates in equality.

ReserveInPlace takes a **minimum total capacity**, not an additional-byte count.
It does not change length or shrink storage. ResizeInPlace to a smaller length,
including zero, retains capacity; growing later exposes zero bytes, not the old
discarded contents. TrimCapacityInPlace sets this value's capacity to its length;
on an empty value it drops the backing reference and sets capacity to zero.
Other copies/native owners may still legitimately keep the old allocation.
Detaching shared storage may allocate even when capacity is already sufficient;
Reserve is not an allocation-free guarantee for later copy-on-write mutation.

An empty Slice produces a zero-capacity empty value. A nonempty proper subrange
copies just those bytes into capacity equal to Len; it never retains a larger
parent allocation. A nonempty full-range slice may share the original storage
and capacity through copy-on-write. SliceInPlace replaces the target with that
same result. No additional view/detach API is exposed.

## Equality, display and lifetime

ByteBuffer satisfies `Equatable` and `Printable`, not `Comparable` or `Numeric`.
Equality compares length and byte contents; dictionaries can use buffer keys
as value snapshots. Mutating a caller's original value does not change an
inserted key. There is no ordering, Sort support, or public Hash operation. Any
internal hash must agree with byte-content equality.

Automatic release follows the last legitimate owner, including ownership through
cycles, task/channel payloads, native requests/results and retained debugger values.
Native input snapshots are fixed at handoff; selectable writes retain the bytes
provided at case construction. Native jobs use owned Rust storage, never borrowed
FPAS registers or a VM/domain lock held across I/O. ByteBuffer alone is
transferable data, not a VM-local resource handle; surrounding mutable captures
and resources keep their existing restrictions.

Account for backing bytes/capacity and shared owners, counting one shared
allocation once. Do not introduce a buffer-specific retention pool in this AP;
release backing storage after its last legitimate owner disappears. The system
allocator's own retained pages are not live buffer allocations.

## UTF-8 API and errors

With `uses Std.Text.Utf8 as Utf8;`, the complete text boundary is:

| Operation | Result |
| --- | --- |
| `Utf8.Encode(Text: string)` | `result of (Bytes.ByteBuffer, Bytes.BufferError)` |
| `Utf8.Decode(Bytes: Bytes.ByteBuffer)` | `result of (string, Utf8.Utf8Error)` |
| `Utf8.AppendInPlace(var Buffer: Bytes.ByteBuffer; Text: string)` | `result of (boolean, Bytes.BufferError)` |

Decode keeps its existing public argument label Bytes. There is no second
Encode/Decode or text-append operation in Std.Bytes. UTF-8 append encodes and
appends in one native bulk operation, with the same atomic commit, limit and
growth rules as byte ConcatInPlace. Success is Ok(true).

UTF-8 encoding is semantically valid for every string, but buffer limits and
recoverable allocation failure make its result explicit. Decode is strict and
never modifies input, replaces invalid bytes, strips BOM, normalizes text or
treats NUL specially. Empty text/bytes round-trip without content changes.

Utf8Error retains `InvalidByte(Offset: integer; Detail: string)` and
`IncompleteSequence(Offset: integer)`, and adds
`AllocationFailed(RequestedBytes: integer)` for recoverable string allocation.
Offset always identifies the **zero-based start of the first invalid or
incomplete sequence** in the supplied buffer. Detail is explanatory text, not
a machine-matched contract. Use Rust's established UTF-8 validation and its valid
prefix boundary; update fixtures that expected a later bad continuation byte.

One-shot Decode rejects an incomplete tail. HTTP/SSE retains bounded fragments
until its existing finalization boundary; it does not call one-shot Decode on
arbitrary incomplete fragments. Additional encodings and a public incremental
decoder remain excluded.

## Migration, artifacts and existing debugger capabilities

Use one mechanical type rule for the existing binary families listed in the
package README: replace each **byte-valued** `array of integer` leaf with
ByteBuffer, including record bodies, optional pending reads and callback
signatures. Their operations keep their canonical names and their domain error
families; retain EOF/pending, partial counts, cancellation and host-effect rules.
Domain error paths caused solely by an invalid integer byte disappear once their
input is a trusted buffer. Preserve malformed text/protocol and host failures.
Use explicit array conversions at real application interoperability boundaries,
not inside every I/O handoff. Token IDs and unrelated integer collections stay
arrays. Source helpers and every public caller migrate together in AP54.3.

Utf8.Encode's newly explicit Result requires caller error handling. Propagate or
map expected errors to the caller's existing error family rather than adding
compatibility wrappers or unconditional Unwrap calls to library code. Decode's
argument remains named Bytes; existing public byte-I/O parameter labels remain
unchanged when only their payload type changes.

The nominal type must work in separately compiled interfaces, program images,
ordinary values, first-class operations and the existing debugger's inspection,
retained values, detached calls and bounded event capture. Retaining a live
buffer value preserves its complete bytes, length, capacity and shared backing;
presentation clipping never changes that value. Invalid/corrupt type artifacts
must fail with a diagnostic. Current format versions may change; compatibility
shims are unnecessary.

The user explicitly keeps general VM snapshots, reverse execution, recording
to disk and state restoration/replay separate from AP54. Those capabilities are
not implemented by the current debugger. ByteBuffer must work in its existing
capabilities without a buffer-specific rejection, and must not advertise or
implement a general snapshot/replay engine as part of this work.

Debugger/Printable presentation shows ByteBuffer, logical length and at most
32 bytes in uppercase hex, with an explicit truncation indication. Inspection
also exposes capacity. Preview truncation never truncates a retained live value;
raw bytes are not implicitly decoded as text.

## API examples

These examples illustrate the implemented API without adding syntax.

```pascal
program BufferValues;

uses Std.Bytes as Bytes, Std.Test as Test;

begin
  var Working := Bytes.FromArray([1, 2, 3]).Unwrap();
  const Original := Working;
  discard Working.SetAtInPlace(1, 9).Unwrap();
  Test.AssertEquals(2, Original[1]);
  Test.AssertEquals(9, Working[1]);
  const Part := Working.Slice(Start := 1, Len := 2).Unwrap();
  Test.AssertTrue(Part = Bytes.FromArray([9, 3]).Unwrap());
  Test.AssertTrue(Working.Slice(3, 0).Unwrap().IsEmpty());
  discard Working.ConcatInPlace(Working).Unwrap();
  Test.AssertEquals(6, Working.Length());

  discard Working.ResizeInPlace(0).Unwrap();
  discard Working.TrimCapacityInPlace().Unwrap();
  Test.AssertEquals(0, Working.Capacity());
end.
```

```pascal
program BufferText;

uses Std.Bytes as Bytes, Std.Text.Utf8 as Utf8, Std.Test as Test;

begin
  var Data := Utf8.Encode('Grüße').Unwrap();
  discard Utf8.AppendInPlace(var Data, #0 + '🌍').Unwrap();
  Test.AssertEquals('Grüße' + #0 + '🌍', Utf8.Decode(Data).Unwrap());
  const Invalid := Bytes.FromArray([65, 226, 40]).Unwrap();
  Test.AssertTrue(Utf8.Decode(Invalid) is Error(Utf8.Utf8Error.InvalidByte(1, _)));
  const Incomplete := Bytes.FromArray([65, 226, 130]).Unwrap();
  Test.AssertTrue(Utf8.Decode(Incomplete) is Error(Utf8.Utf8Error.IncompleteSequence(1)));
  Test.AssertTrue(Bytes.FromArray([256]) is Error(Bytes.BufferError.InvalidByte(0, 256)));
  Test.AssertTrue(Bytes.Create(268435457) is Error(Bytes.BufferError.LimitExceeded(268435456)));
end.
```

## Delivery and implementation boundaries

Q72–Q76 are recorded in the package README with no unresolved design question.
The examples specify value/copy independence, empty ranges, self-concatenation,
capacity release, Unicode/NUL contents, invalid/truncated UTF-8 and limit/byte
errors. Regression coverage also includes failed allocations,
overlapping copies, dictionary keys, protected iteration, task/native snapshots,
partial I/O, compiled-artifact reuse and the existing debugger capabilities.

AP54.2 owns native storage/type/catalog/error/metadata and its tests; AP54.3 owns
the atomic byte-API/caller migration and UTF-8 implementation. Their affected-file
tables identify the implementation owners. The namespace specification includes
Std.Bytes in the current catalogue; the handbooks and generated declarations
document the implemented signatures.

Internal container choice, module layout, allocation-failure test injection and
copy-on-write fast paths remain delegated implementation work. They must honor
this contract, not introduce user-visible alternatives. A necessary FPAS syntax
change still needs explicit user approval. Commit/push and implementation
authorization are separate from completion of this decision work package.
