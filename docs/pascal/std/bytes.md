# `Std.Bytes`

Managed contiguous byte buffers with copy-on-write values, checked operations and
automatic memory release. Import `uses Std.Bytes as Bytes;`.

`Bytes.ByteBuffer` stores bytes without a tagged FPAS integer for each element.
Reads return ordinary `integer` values in `0..255`. Positions and lengths count
bytes. Arbitrary bytes, including zeros and malformed UTF-8, are valid contents.
The buffer itself does not interpret text.

## Factories and errors

| Kind | Name | Notes |
| --- | --- | --- |
| type | `ByteBuffer` | Managed byte value; construction uses Create or FromArray |
| type | `BufferError` | Expected byte-conversion, limit and allocation failures |
| function | `Create(Length: integer): Result of (Bytes.ByteBuffer, Bytes.BufferError)` | Length zero bytes, with initial capacity equal to length |
| function | `FromArray(Values: array of integer): Result of (Bytes.ByteBuffer, Bytes.BufferError)` | Validate all values before copying into exact-capacity storage |

Handle expected failures with `case`, propagate compatible errors with `try`, or
deliberately abort with `.Unwrap()`. Every InPlace function returns `Ok(true)` on
success, including no-op calls. It never returns `Ok(false)` as another failure
channel. Consume that Result explicitly.

## Receiver operations

Each operation has only its receiver form. Positional and fully named arguments
follow normal FPAS rules. `Length()` and `IsEmpty()` use the same names as arrays,
strings and dictionaries; ranges consistently use Start and Len.

| Operation | Meaning |
| --- | --- |
| `Buffer.Length(): integer` | Logical byte count |
| `Buffer.IsEmpty(): boolean` | Length is zero |
| `Buffer.Capacity(): integer` | Reserved total byte capacity |
| `Buffer.ToArray(): Result of (array of integer, Bytes.BufferError)` | Copy bytes to ordinary integer elements |
| `Buffer.Slice(Start: integer; Len: integer): Result of (Bytes.ByteBuffer, Bytes.BufferError)` | Checked selected range |
| `Buffer.Concat(B: Bytes.ByteBuffer): Result of (Bytes.ByteBuffer, Bytes.BufferError)` | Receiver followed by B |
| `Buffer.SetAt(Index: integer; Value: integer): Result of (Bytes.ByteBuffer, Bytes.BufferError)` | Replace one byte in a new value |
| `Buffer.Fill(Value: integer; Start: integer; Len: integer): Result of (Bytes.ByteBuffer, Bytes.BufferError)` | Fill an existing range in a new value |
| `Buffer.CopyFrom(Source: Bytes.ByteBuffer; SourceStart: integer; TargetStart: integer; Len: integer): Result of (Bytes.ByteBuffer, Bytes.BufferError)` | Overwrite a target range from the original source bytes |
| `Buffer.Resize(Length: integer): Result of (Bytes.ByteBuffer, Bytes.BufferError)` | Preserve prefix, retain capacity and zero newly exposed bytes |
| `Buffer.SliceInPlace(Start: integer; Len: integer): Result of (boolean, Bytes.BufferError)` | Replace writable storage with its selected range |
| `Buffer.ConcatInPlace(B: Bytes.ByteBuffer): Result of (boolean, Bytes.BufferError)` | Append B to writable storage |
| `Buffer.SetAtInPlace(Index: integer; Value: integer): Result of (boolean, Bytes.BufferError)` | Replace one checked byte |
| `Buffer.FillInPlace(Value: integer; Start: integer; Len: integer): Result of (boolean, Bytes.BufferError)` | Fill an existing range |
| `Buffer.CopyFromInPlace(Source: Bytes.ByteBuffer; SourceStart: integer; TargetStart: integer; Len: integer): Result of (boolean, Bytes.BufferError)` | Overwrite a checked range, including overlapping aliases |
| `Buffer.ResizeInPlace(Length: integer): Result of (boolean, Bytes.BufferError)` | Change logical length without automatically shrinking capacity |
| `Buffer.ReserveInPlace(Capacity: integer): Result of (boolean, Bytes.BufferError)` | Reserve at least this total capacity, without changing length |
| `Buffer.TrimCapacityInPlace(): Result of (boolean, Bytes.BufferError)` | Reduce capacity to current length |

Read a byte with `Buffer[Index]`. Byte slots are read-only: use SetAt or
SetAtInPlace to replace a byte. A `for-in` loop visits byte integers in order.
Its writable source root is protected against changes throughout the body, as
for arrays; an independent copied value can still change.

## Values, mutation and tasks

Assignment, arguments, return values and collection entries retain value
semantics. Copies may share immutable storage. An InPlace call changes writable
caller storage, detaching when another value needs the original bytes. Constants,
read-only parameters and computed receivers are not writable. There are no free
routine aliases of receiver operations, optional arguments or implicit array
conversions. Value versus InPlace denotes a new result versus explicit mutation.

On failure the operation commits neither a new byte value nor a new capacity.
Completed argument effects remain visible. Overlapping CopyFrom and self-Concat
use stable source bytes. Native InPlace calls cannot capture a writable receiver
as a function value or cross `go`. Value operations and factories can be passed
as ordinary first-class operations.

Buffers satisfy `Equatable` and `Printable`, support `=` and `<>`, and can be
dictionary keys. Equality ignores capacity and backing identity. Mutating an
original value does not change a key already inserted into a dictionary.
There is no byte-buffer ordering or public hash operation.

Buffers are transferable data. Channels, task arguments/results, aggregates and
read-only closures retain stable values. Surrounding task-bound captures and host
resources keep their normal restrictions. Memory is released when its last
legitimate owner disappears, including through cycle collection; no Close is
needed. A debugger snapshot is also a legitimate owner until it is discarded.

## Bounds, limits and capacity

The maximum length and capacity are **268435456 bytes (256 MiB)**. This limit
does not replace a consumer's separate I/O or protocol limit.

Direct indices require `0 <= Index < Length`. Ranges require a nonnegative Start
and Len, with `Start <= Length` and `Len <= Length - Start`. A zero-length range
at the end is valid. Invalid indices/ranges and negative lengths/capacities are
runtime errors. Nonnegative size requests above the maximum are typed failures.

| BufferError variant | Meaning |
| --- | --- |
| `InvalidByte(Offset: integer; Value: integer)` | Value is outside `0..255`; Offset is the first invalid FromArray element, SetAt's Index, or Fill's Start |
| `LimitExceeded(Limit: integer)` | Buffer length/capacity would exceed Limit |
| `AllocationFailed(RequestedBytes: integer)` | A recoverable native data-storage allocation failed |

Size preflight precedes conversion input scanning and allocation. Ranges are
validated before scalar byte values and mutation. Fill validates Value even
when Len is zero. ToArray expands native bytes into tagged integer elements and
can require substantially more memory than the byte buffer.

Automatic capacity growth doubles from at least 64 bytes, bounded by the limit.
ReserveInPlace takes a minimum **total** capacity. Shrinking through Resize keeps
capacity; growing later exposes zeros, not discarded bytes. TrimCapacityInPlace
on an empty value drops its backing reference and makes capacity zero. Other
copies can still retain the original allocation. A later copy-on-write mutation
can allocate even when capacity is sufficient.

An empty Slice has zero capacity. Proper subranges copy only the selected bytes
and do not keep a larger parent allocation alive. A nonempty full-range Slice
may share backing and capacity. Printable/debugger summaries show length and at
most 32 bytes in uppercase hex with an explicit truncation marker. Debugger
inspection also exposes capacity; display limits do not truncate retained data.

## Example

```pascal id=byte-buffer-values run
program BufferValues;

uses Std.Bytes as Bytes, Std.Test as Test;

begin
  var Data := Bytes.FromArray([0, 1, 255]).Unwrap();
  const Original := Data;
  discard Data.SetAtInPlace(1, 9).Unwrap();
  Test.AssertEquals(1, Original[1]);
  Test.AssertEquals(9, Data[1]);
  const Part := Data.Slice(Start := 1, Len := 2).Unwrap();
  Test.AssertTrue(Part = Bytes.FromArray([9, 255]).Unwrap());

  discard Data.ResizeInPlace(0).Unwrap();
  discard Data.TrimCapacityInPlace().Unwrap();
  Test.AssertEquals(0, Data.Capacity());
end.
```

File, socket, process, codec, random-byte and HTTP byte interfaces share this
buffer type. `Std.Text.Utf8.Encode` returns a buffer Result; Decode validates a
buffer, and AppendInPlace appends encoded text. FromArray and ToArray provide
explicit interoperability with ordinary integer collections.

## Implementation (contributors)

| Concern | Location |
| --- | --- |
| Native backing and fallible copy-on-write | [`value/bytes.rs`](../../../crates/fpas-runtime/src/value/bytes.rs) |
| Runtime operations | [`stdlib/bytes.rs`](../../../crates/fpas-runtime/src/stdlib/bytes.rs) |
| Native receiver signatures | [`native/bytes.rs`](../../../crates/fpas-sema/src/std_registry/native/bytes.rs) |
| Factory/type registration | [`loaded/bytes.rs`](../../../crates/fpas-sema/src/std_registry/loaded/bytes.rs) |
| Intrinsic identities | [`intrinsic/bytes.rs`](../../../crates/fpas-code/src/intrinsic/bytes.rs) |
| FPAS regressions | [`tests/stdlib/bytes/`](../../../tests/stdlib/bytes) |

## See also

- [Collection mutation](../language/types/collection-mutation.md)
- [First-class operations](../language/functions/first-class.md)
- [UTF-8](network/utf8.md)
- [Binary encoding](text/encoding.md)
