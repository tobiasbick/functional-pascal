# AP54.3: Integrate UTF-8 and migrate canonical binary APIs

Package: [AP54](README.md). Status: not started; depends on AP54.2 and
implementation authorization. AP54.1 is complete.

## Prerequisites

- [AP54.1](01-contract.md) fixes the signatures, UTF-8 sequence-start diagnostics,
  allocation failures and canonical byte-leaf replacement rule.
- [AP54.2](02-native-storage.md) delivers the buffer and passes its checks.
- Implementation is authorized. Any additional syntax proposal needs its own
  explicit approval before implementation.

## Contract and scope

Make the buffer the one byte representation for existing public binary APIs and
their implementations. Replace signatures and migrate all callers in this part,
so the delivered checkout builds and runs without a second public array API or
temporary conversion wrappers. Ordinary integer collections are unaffected.

| Owner | Required migration |
| --- | --- |
| `Std.Text.Utf8` | Encode strings to the buffer, strictly decode the buffer and provide the agreed efficient text-append path under the existing namespace boundary |
| `Std.Encoding` | Base64/Base64url/hex accept and produce buffers; keep alphabet, padding, canonicality, offsets and consumer limits except for agreed buffer-specific changes |
| `Std.Crypto` | Random-byte production publishes the same buffer type directly from owned native bytes |
| `Std.Io.Fs` | Whole-file and handle byte reads/writes, including cancellation variants, accept/return the canonical type |
| `Std.Net.Sockets` | TCP/TLS byte send/receive and cancellation variants use the same type |
| `Std.Os.Proc` | Interactive pipe probes, writes, pending selections and read/write callbacks share the canonical type |
| `Std.Net.Http` | Body records, streams, wire parsing, server responses and SSE feeding/buffering use the same storage |
| `Std.Net.Uri` | Private percent/query byte helpers use the buffer; public URI text contracts stay intact |

Keep the existing public `Utf8.Encode` and `Utf8.Decode` names, replacing their
integer-array byte side with `Bytes.ByteBuffer` and the exact error/result
signatures fixed in AP54.1. The only combined text-append operation is
`Utf8.AppendInPlace(var Buffer, Text)`, returning Result with Ok(true) on success;
do not duplicate it as a second encoding operation in `Std.Bytes`.

Use Rust's established UTF-8 facilities for the native text boundary, with any
small adapter needed to provide the approved FPAS error identity and offsets.
Do not build a second handwritten Unicode decoder. Preserve raw malformed bytes
in a buffer; decode failure must not normalize, discard or replace them.
NUL, BOM contents, empty strings and multibyte scalars retain their text contents.
Buffer positions remain byte positions; string indexing stays scalar-based.
Malformed and incomplete input errors identify the zero-based sequence start,
as fixed in AP54.1. Update any continuation-byte-offset expectations accordingly.
Preserve error-family identity and distinguish malformed/truncated input from
recoverable string-allocation failure. Utf8.Encode now returns a typed buffer
Result; migrate every caller's propagation/error mapping, including source
helpers, rather than adding a hidden infallible compatibility path.

One-shot decoding may reject a truncated chunk. Existing HTTP/SSE code continues
to buffer incomplete sequences across fragments under its own bounded protocol
state and finalization rules. This does not create a public incremental decoder.

## Native ownership and preserved behavior

Native input and output jobs retain owned stable storage, never borrowed FPAS
registers, mutable caller storage or a VM/domain lock across I/O. Mutating a
caller value after constructing a selectable write cannot change the bytes that
the agreed snapshot contract has already retained. Publish native byte results
without expanding them into tagged integers; dispose of unpublished results
on failure, cancellation or shutdown.

Preserve partial-read/write counts, byte ordering, EOF versus pending, line
lookahead, per-operation limits, cancellation races and single-worker progress.
Protocol/text limits are not replaced by the buffer's general allocation limit.
Already performed host effects remain non-transactional. Inline callbacks and
the all-stop debugger retain the managed host-I/O contract.

Range validation moves to entry points that accept ordinary integers/arrays;
trusted buffers do not need a full byte-domain scan at every handoff. Continue
validating sizes, argument combinations, resource state and protocol content at
their actual boundaries. Remove obsolete invalid-array-byte paths only as
specified in AP54.1; retain distinct malformed-UTF-8 and host errors.

## Affected files and implementation owners

| Action | Path | Purpose |
| --- | --- | --- |
| Modify | `lib/Std/Text/Utf8.fpas`, `crates/fpas-runtime/src/stdlib/str/` | Replace integer-array text boundaries and the old decoder implementation |
| Modify | `crates/fpas-runtime/src/stdlib/encoding/`, `execution/vm/hosted/crypto.rs` | Native codec/random results and buffer-consuming encoders |
| Modify | `crates/fpas-runtime/src/stdlib/fs/`, `execution/vm/hosted/fs/`, `hosted/proc/`, socket/network owners in `hosted/` | Read/write buffers, native job data, cancellation and selection publication |
| Modify | `lib/Std/Net/Http.fpas`, `lib/Std/Net/Http/`, `lib/Std/Net/Uri/` | Source-defined body types, parsers, chunking and percent helpers |
| Modify | `crates/fpas-sema/src/std_registry/loaded/`, `crates/fpas-builtins/src/`, `crates/fpas-code/src/intrinsic/`, runtime layout/dispatch owners | Canonical signatures, callback types, errors and metadata |
| Modify | `apps/`, `examples/`, `tests/`, `tools/`, source units and Rust-embedded fixtures | Migrate every affected consumer with resolved type/name information |
| Modify | Affected `docs/pascal/` pages and `docs/pascal/agents/` guidance | Exact new signatures, errors, examples and byte-versus-text rules |
| Generate | Affected files under `lib/api/Std/` | Replace declarations through the existing exporter, not manual edits |

Paths are relative to the repository root. Before implementation, expand the
socket/network and metadata rows to actual files and check module sizes. Search
beyond `array of integer`: aliases, records, nested arrays and callbacks can
carry byte payloads. Do not rewrite tokenizer token IDs, search result indexes
or other integer arrays just because their spelling matches.

## Acceptance and verification

Each family above accepts/produces the same type and all affected consumers
compile. Verify binary round trips with all 256 byte values and embedded zeros;
strict UTF-8 round trips and malformed/overlong/surrogate/out-of-range/truncated
sequences; Base64/hex vectors and limits; HTTP/SSE fragments split inside
multibyte sequences; URI percent encoding; process EOF/pending/partial writes;
file line/byte mixing and cancellation; TCP/TLS transfer and close behavior.

Reuse owning `tests/stdlib/{bytes,net,encoding,crypto,fs}/` suites and process,
network and native-job Rust regressions. Test dropped select cases and undelivered
native results with actual backing-storage accounting, not only successful reads.
Check one VM worker, all-stop debugging and inline callback paths.

Run full Rust and FPAS suites, affected app/example checks, FPAS/Rust formatting,
strict Clippy, generated API and executable documentation checks. Audit remaining
binary integer-array use and explain intentional general-array conversions.
No final public compatibility family or source-level workaround may remain.
Document delivered behavior and fix confirmed review findings before checking
off AP54.3.
