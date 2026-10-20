# Standard library gap review

Status: assessment updated on 2026-10-20. Gaps 1 through 11 are implemented;
additional candidates are unscheduled.

This review covers the existing standard library, including native string and
collection operations, but excludes `Std.Ai.*`. Findings were checked against
public declarations, the handbook and runtime/source implementations. Go,
Python, Rust and Ada provide comparison points; a feature in another language
is not automatically a requirement for FPAS.

The goal remains a functional Pascal with selected Ada conventions, predictable
APIs, one canonical way to express each operation, and a stable FPAS 1. The
assessment distinguishes everyday capability gaps, optional convenience helpers
and larger extensions. It supplements the [standard library roadmap](std-roadmap.md).

## Existing coverage

FPAS already supplies TCP/TLS clients and listeners, HTTP response streaming and
SSE decoding, task groups, cancellation, mixed-source selection, supervised work,
atomic text publication and a source-level TUI. Native arrays and dictionaries
provide higher-order operations; arrays include stable `SortBy` and `FlatMap`.
Strings support Unicode scalar indexing and casing, while Console exposes
grapheme segmentation and display widths. Native error declarations, typed causes,
automatic output and explicit MapError are delivered through AP47, including both
selected Std cohorts and integrated acceptance. Console text reads return Option,
so EOF is observable without losing empty lines. String formatting supports numeric
field widths, alignment, zero padding, fixed precision and scientific notation.
Pure path helpers detect absolute paths, calculate relative paths and replace
filename extensions under host platform rules.
Binary-to-text codecs provide bounded Base64, Base64url and hexadecimal
conversions over the existing integer-array byte representation. Schema-based
command-line parsing supplies typed options and positions, defaults, generated
help and recoverable errors without reading process state. Bounded binary file
reads and writes preserve raw bytes using the existing integer-array representation.
VM-owned file handles add bounded block and line access, explicit opening modes,
byte positioning and cooperative cancellation without loading a complete file.
Controlled process calls add per-call input, working directories and environment
overrides, bounded captured output, deadlines and cancellation with owned
process-tree cleanup.
URL component codecs and ordered query parsing/building preserve UTF-8 data,
duplicate parameters, flags and explicit empty values with bounded resources.
Pure Gregorian dates and UTC date-times add checked construction, strict
ISO/RFC-3339 conversion and day/millisecond arithmetic over years 1..9999.
These are implemented capabilities.

## Everyday capability gaps

Numbers are stable references within this assessment, not approved work packages.
Gaps 1 through 11 are closed. Their numbers remain reserved; the historical
structured-error direction anchor remains stable.

No everyday capability gaps remain in this assessment. The additional candidates
below are unscheduled.

### Evidence in FPAS

- [Console input implementation](../../crates/fpas-std/src/console/input.rs) and
  [public text input](../pascal/std/console/input.md): closed gap 1.
- [Filesystem API](../pascal/std/host/fs.md) and
  [binary implementation](../../crates/fpas-std/src/fs/binary.rs): closed gap 2.
- [Incremental file contract](../pascal/std/host/fs-streams.md) and
  [VM implementation](../../crates/fpas-vm/src/vm/hosted/fs/mod.rs): closed gap 3.
- [Process API](../pascal/std/host/proc.md) and
  [VM implementation](../../crates/fpas-vm/src/vm/hosted/proc/mod.rs): closed gap 4.
- [Calendar API](../pascal/std/host/calendar.md) and
  [source implementation](../../lib/Std/Calendar.fpas): closed gap 5;
  [Time](../pascal/std/host/time.md) remains the clock and sleep API.
- [String formatting](../pascal/language/types/string/format-chars.md),
  [implementation](../../crates/fpas-std/src/str/format.rs) and
  [conversion](../pascal/std/text/conv.md): closed gap 6.
- [Command-line parsing](../pascal/std/host/cli.md) and
  [source implementation](../../lib/Std/Cli.fpas): closed gap 7;
  [Args](../pascal/std/host/args.md) remains the process-argument accessor.
- [Paths](../pascal/std/host/path.md) and
  [implementation](../../crates/fpas-std/src/path.rs): closed gap 8.
- The Result signatures in the filesystem, process,
  [network](../pascal/std/network/net.md) and
  [parsing](../pascal/std/text/parse.md) APIs: closed gap 9.
- [Byte codecs](../pascal/std/text/encoding.md) and
  [implementation](../../crates/fpas-std/src/encoding): closed gap 10.
- [URI components and queries](../pascal/std/network/uri.md) and
  [source implementation](../../lib/Std/Net/Uri.fpas): closed gap 11.

### Closed gap 1: observable console EOF

`Console.ReadLn()` and `Console.ReadText()` return `Option of string`. A blank
line is `Some('')`; EOF is `None`. Both consume the same text buffer, including
queued and debugger input. Final unterminated lines are returned once, and real
I/O failures remain runtime diagnostics. See the
[Console input contract](../pascal/std/console/input.md).

### Closed gap 2: bounded binary files

`Fs.ReadBytes` reads raw file bytes into an integer array and `Fs.WriteBytes`
creates or replaces a file from integer bytes in `0..255`. Both operations are
limited to 1 MiB per call. Writes validate size and every byte before opening
the destination; validation failures leave existing files unchanged and create
no new files. Empty arrays and files are valid. Reads bound the actual stream
as well as checking its reported size. Typed `FsError` failures preserve host
causes, and `InvalidByte(Offset, Value)` identifies the first invalid element.
Normal writes retain the existing `WriteText` behavior for I/O failures.
See the [filesystem contract](../pascal/std/host/fs.md). Incremental access is
covered by gap 3; compact byte buffers remain a separate topic.

### Closed gap 3: incremental regular-file I/O

`Std.Fs.File` is an opaque VM-owned handle with explicit opening modes and
`Close`. Copies share a byte position and close state. Block reads and writes,
strict UTF-8 line reads, `Seek`, `Tell` and `Flush` operate without loading the
whole file. Byte transfers and line contents are bounded at 1 MiB; a VM can hold at most 1,024
open files. Line reads distinguish empty lines from EOF and retain buffered
prefixes after cancellation or a size failure so callers can recover through
byte reads or seeking. Writes validate the complete byte array before I/O and
return the actual transferred count. Cancellation variants observe tokens before
block I/O and between line chunks; they cannot interrupt a running OS call.
Typed `FsError` failures retain host causes, and VM/session termination closes
remaining handles. See the [incremental file contract](../pascal/std/host/fs-streams.md).

### Closed gap 4: controlled process execution

`Proc.DefaultOptions`, `RunWithOptions` and `RunCaptureWithOptions` add UTF-8 stdin,
per-call working directories and environment overrides without changing host
process state. Captured output defaults to 1 MiB per stream and can be configured
up to 64 MiB. Output limits, monotonic deadlines and cancellation return typed
errors with captured prefixes. Input and both output pipes progress concurrently;
EOF is explicit. VM-owned process groups on Unix and Jobs on Windows terminate
remaining descendants and reap the direct child during completion or shutdown.
Existing call signatures and nonzero exit codes remain supported. Interactive
process handles, detached processes and incremental public output are outside
this API. See the [process contract](../pascal/std/host/proc.md).

### Closed gap 5: Gregorian calendar dates and UTC date-times

`Std.Calendar` provides immutable date and UTC date-time values with checked
construction and read-only component methods. The proleptic Gregorian calendar
covers years 1..9999, including negative Unix milliseconds before 1970.
Strict date and RFC-3339 parsers accept up to three fractional digits and known
numeric offsets, normalize to UTC and reject unknown offsets or precision loss.
Canonical formatting emits fixed-width ISO dates and UTC milliseconds. Day and
millisecond addition checks bounds before arithmetic, and signed differences
fit an integer. Invalid components, syntax, offsets and range failures return
native `CalendarError` values. The implementation is a pure source unit;
clock/sleep APIs are unchanged. Month/year addition, named time zones,
daylight-saving rules and localization remain separate extensions. See the
[calendar contract](../pascal/std/host/calendar.md).

### Closed gap 6: numeric output formatting

String `Format` and `FormatInPlace` support numeric field width, left alignment,
zero padding, decimal precision and scientific `%e`/`%E` notation. Rounding uses
ties to even; explicit integer precision and scientific notation preserve the
full integer value. Existing bare formats retain their behavior. Field sizes
and the combined output of extended numeric fields are bounded. Formatting is
locale-independent; localized presentation remains a separate design topic.
See the [formatting contract](../pascal/language/types/string/format-chars.md).

### Closed gap 7: typed command-line parsing

`Std.Cli` parses an explicit argument array using a schema of string, integer,
real and Boolean options and positions. It supports long and short options,
negative Boolean aliases, the `--` separator, required values and typed defaults.
Successful parsing returns either typed values or generated help; schema, token,
value and resource failures use `CliError`, preserving `ParseError` causes.
The parser is source-defined, bounded and independent of process state, output
and exit status. Subcommands, repeated options and short bundles remain outside
the initial API. See the [CLI parsing contract](../pascal/std/host/cli.md).

### Closed gap 8: pure path helpers

`Path.IsAbsolute` tests whether a path is fully absolute under host rules.
`RelativeTo(Target, Base)` calculates a lexical relative path and returns `None`
when the relationship is indeterminate. `ChangeExtension(Path, Extension)`
replaces or removes the final suffix, returning `None` for invalid extensions or
paths without a filename. These operations do not access the current directory,
filesystem or symlinks. Effectful real-path resolution remains a separate `Fs`
concern. See the [Path contract](../pascal/std/host/path.md).

### Direction for gap 9: structured errors

The selected [FPAS 2 direction](fpas2-language-directions.md#1-typed-operational-errors)
is a native `type T = error ... end error` declaration. Standard-library and
application error families retain concrete variants and typed payloads in Result.
Literal message labels, automatic output and a designated typed Cause remove
per-type reporting functions and preserve the original error under wrappers.

The implemented contract is documented in the
[language handbook](../pascal/language/types/errors.md) and
[Std reference](../pascal/std/README.md), including nested callback Results,
a [shared native HostError leaf](../pascal/std/host/errors.md) and explicit domain
mappings. Native conditions are classified from structured outcomes, never by
parsing message strings.

MapError is the explicit general Result error transformation. Ok preserves its
payload without calling the mapper; Error calls it once. Constructor values make
wrapping concise, for example `try Fs.ReadText(Path).MapError(ConfigError.FileFailure)`.
Try still requires compatible error types after mapping and does not choose a
wrapper automatically. Option, runtime failures, contracts and cleanup retain
their existing behavior.

Selected public signatures are replaced in place with all affected consumers;
there is no parallel string/typed wrapper family. Existing successes, limits,
EOF and lifecycle distinctions remain specified at their current owner. No new
I/O capability is bundled into the error migration. Native error declarations,
constraints, Message(), constructor values and MapError are implemented, along
with host/parsing, task/network and layered HTTP families. SDK and TUI boundaries
that retain strings use explicit Message() mappings.

### Closed gap 10: bounded byte codecs

`Std.Encoding` encodes and decodes standard padded Base64, unpadded Base64url
and hexadecimal text. Encoders validate integer bytes in `0..255`; decoders
reject whitespace, invalid alphabets, padding and noncanonical unused bits.
Hexadecimal output is uppercase, while decoding accepts either case. All six
functions return Result with a native `EncodingError` and preserve empty inputs.
Binary inputs and outputs are limited to 1 MiB, with matching encoded-text bounds
checked before output allocation. See the
[encoding contract](../pascal/std/text/encoding.md). Compact buffers remain separate.

### Closed gap 11: URL components and ordered queries

`Std.Net.Uri` provides UTF-8 percent encoding and strict decoding for path
segments and query components. Encoders leave only RFC 3986 unreserved ASCII
characters literal and emit uppercase escapes; space is `%20`, slash is `%2F`,
and decoding preserves a literal plus sign. Query helpers retain parameter order,
duplicate names and the difference between a flag and an explicit empty value.
They split separators before decoding and return typed `ComponentError` failures,
including original `Utf8Error` causes. The helpers cap decoded text at 64 KiB,
encoded text at 192 KiB and query entries at 4,096. Existing HTTP/HTTPS URI
parsing retains its grammar, nominal `UriError` and raw request-target behavior.
See the [URI contract](../pascal/std/network/uri.md).

## Additional candidates

These have more application-specific value or can already be expressed using
existing language/library operations. Their absence alone does not justify an API.

| ID | Area | Missing capability | Comparison |
| --- | --- | --- | --- |
| 12 | Logging and stderr | No general logging API with severity and structured fields; the public Console API also lacks direct stderr output. Debugger diagnostics serve a different purpose. | Go `log/slog`; Python `logging` |
| 13 | CSV | No parser/writer handling quoting, embedded delimiters and multiline fields. String `Split` does not implement CSV. | Python `csv`; Go `encoding/csv` |
| 14 | Unicode helpers | Normalization, case folding and character classification are absent. Scalar operations, Unicode casing and grapheme segmentation already exist. | Python `unicodedata`; Go `unicode` |
| 15 | Pattern matching | No regular-expression search, capture groups or replacement. Existing string operations cover literal searches. | Python `re`; Go `regexp` |
| 16 | Collection helpers | No standard `Zip`, grouping, duplicate removal or binary search. Many such operations can be expressed with loops and `Reduce`; add a helper only for repeated, concrete needs. | Rust iterators/slices; Python `itertools` |
| 17 | Test comparisons | `AssertEquals` supports scalars only. Approximate real comparisons and direct array/record comparisons with useful difference reports are absent. Record equality can already be checked with `AssertTrue`. | Python `unittest` |
| 18 | Cryptography | Secure randomness exists; hashes, HMAC, password hashing/verification and signatures do not. These are already deferred in the [cryptography plan](networked-applications/cryptography.md). | Go `crypto/*`; Python `hashlib`/`hmac` |

Relevant FPAS references are the [Console API](../pascal/std/console/README.md),
[text units](../pascal/std/text/README.md),
[string operations](../pascal/language/types/string/README.md),
[collection operations](../pascal/std/collections/README.md),
[test assertions](../pascal/std/testing/test.md) and
[Crypto](../pascal/std/cryptography/crypto.md).

## Larger extensions and existing plans

- Compression and archive reading/writing are absent, for example gzip, ZIP and
  TAR. Python's `gzip`, `zipfile` and `tarfile` show the application use cases.
  Resource limits and streaming behavior need a design before adding an adapter.
- Embedded transactional storage is covered by the existing
  [SQLite plan](networked-applications/sqlite.md).
- Bidirectional message transport and datagrams are covered by the existing
  [WebSocket](networked-applications/websocket.md) and
  [UDP](networked-applications/udp.md) plans.
- Dedicated compact byte buffers and Decimal/money types involve representation
  or language/runtime decisions. They belong with the
  [FPAS 2 foundations](fpas2-language-directions.md); they are not prerequisites
  for every helper that can use current byte arrays.
- Broader HTTP support is also absent: compression, proxies and persistent
  connections are explicitly outside the current
  [HTTP contract](../pascal/std/network/http.md). These are separate from the
  existing response-streaming support and the implemented URL helpers.

## Selecting further FPAS 1 work

The reviewed everyday capabilities and selected error migrations are implemented.
Further work should be driven by concrete application needs among the additional
candidates and larger extensions. This assessment does not schedule further
implementation work.

## Comparison sources

Representative official source references used for the comparison:

- Go: [I/O](https://github.com/golang/go/blob/go1.26.0/src/io/io.go),
  [process execution](https://github.com/golang/go/blob/go1.26.0/src/os/exec/exec.go),
  [time formatting/parsing](https://github.com/golang/go/blob/go1.26.0/src/time/format.go),
  [flags](https://github.com/golang/go/blob/go1.26.0/src/flag/flag.go),
  [URL helpers](https://github.com/golang/go/blob/go1.26.0/src/net/url/url.go) and
  [structured logging](https://github.com/golang/go/blob/go1.26.0/src/log/slog/doc.go).
- Python: [Unicode database](https://github.com/python/cpython/blob/v3.14.0/Doc/library/unicodedata.rst)
  and [CSV](https://github.com/python/cpython/blob/v3.14.0/Doc/library/csv.rst).
- Rust: [I/O error kinds](https://github.com/rust-lang/rust/blob/1.90.0/library/std/src/io/error.rs).
- Ada: [Calendar.Formatting](https://github.com/gcc-mirror/gcc/blob/releases/gcc-15/gcc/ada/libgnat/a-calfor.ads).
