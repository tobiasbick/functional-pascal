# Standard library gap review

Status: assessment updated on 2026-10-20. Gaps 1, 6, 8 and 9 are implemented;
remaining candidates are unscheduled.

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
These are implemented capabilities.

## Everyday capability gaps

Numbers are stable references within this assessment, not approved work packages.
Gaps 1, 6, 8 and 9 are closed. Their numbers remain reserved;
the historical structured-error direction anchor remains stable.

| ID | Area | Current boundary and missing capability | Comparison |
| --- | --- | --- | --- |
| 2 | Binary files | `Fs` reads and writes UTF-8 text but exposes no byte read/write operations. Existing network/Crypto byte arrays could supply an initial representation without adding language types. | Go `os.ReadFile`/`os.WriteFile`; Python binary files |
| 3 | Incremental file I/O | No public file handles for bounded chunk reads/writes, line-by-line reading or seeking. `ReadText` loads the whole file and rejects files over 64 MiB. HTTP response streaming already exists but does not cover files. | Go `io`/`bufio`; Rust file I/O |
| 4 | Process control | `Run` and `RunCapture` wait for completion. Missing capabilities include supplying stdin, a working directory and environment per invocation, timeout/cancellation, and bounded or incremental captured output. Moving a call into `go` does not add child-process control. | Go `os/exec`; Python `subprocess` |
| 5 | Calendar date and time | `Time` exposes wall-clock/monotonic milliseconds and sleep, but no calendar decomposition, date values, ISO-8601 parsing or date/time formatting. Time-zone handling would be a further design step. | Go `time`; Ada `Calendar.Formatting` |
| 7 | Command-line parsing | `Args` exposes only argument count and indexed lookup. Applications must build typed options, required arguments, help output and consistent parse errors themselves. | Go `flag`; Python `argparse` |
| 10 | Byte codecs | No general Base64/Base64url or hexadecimal encode/decode API for byte arrays. `IntToHex` handles individual numbers. Small codecs can use the existing integer-array byte convention; compact buffers are a separate question. | Go `encoding/base64` and `encoding/hex` |
| 11 | URL components | `Net.Uri` parses absolute HTTP/HTTPS addresses but exposes no query-parameter builder/parser or percent-encoding helpers for query values and path segments. Those contexts need distinct escaping rules. | Go `net/url`; Python `urllib.parse` |

### Evidence in FPAS

- [Console input implementation](../../crates/fpas-std/src/console/input.rs) and
  [public text input](../pascal/std/console/input.md): closed gap 1.
- [Filesystem API](../pascal/std/host/fs.md): gaps 2 and 3.
- [Process API](../pascal/std/host/proc.md) and
  [implementation](../../crates/fpas-std/src/proc.rs): gap 4.
- [Time API](../pascal/std/host/time.md): gap 5.
- [String formatting](../pascal/language/types/string/format-chars.md),
  [implementation](../../crates/fpas-std/src/str/format.rs) and
  [conversion](../pascal/std/text/conv.md): closed gap 6.
- [Arguments](../pascal/std/host/args.md): gap 7.
- [Paths](../pascal/std/host/path.md) and
  [implementation](../../crates/fpas-std/src/path.rs): closed gap 8.
- The Result signatures in the filesystem, process,
  [network](../pascal/std/network/net.md) and
  [parsing](../pascal/std/text/parse.md) APIs: closed gap 9.
- [Conversion](../pascal/std/text/conv.md),
  [Crypto](../pascal/std/cryptography/crypto.md),
  [UTF-8](../pascal/std/network/utf8.md) and
  [URI](../pascal/std/network/uri.md) APIs: gaps 10 and 11.

### Closed gap 1: observable console EOF

`Console.ReadLn()` and `Console.ReadText()` return `Option of string`. A blank
line is `Some('')`; EOF is `None`. Both consume the same text buffer, including
queued and debugger input. Final unterminated lines are returned once, and real
I/O failures remain runtime diagnostics. See the
[Console input contract](../pascal/std/console/input.md).

### Closed gap 6: numeric output formatting

String `Format` and `FormatInPlace` support numeric field width, left alignment,
zero padding, decimal precision and scientific `%e`/`%E` notation. Rounding uses
ties to even; explicit integer precision and scientific notation preserve the
full integer value. Existing bare formats retain their behavior. Field sizes
and the combined output of extended numeric fields are bounded. Formatting is
locale-independent; localized presentation remains a separate design topic.
See the [formatting contract](../pascal/language/types/string/format-chars.md).

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
  existing response-streaming support and from gap 11's URL helpers.

## Suggested FPAS 1 starting order

This is a recommendation for future selection, not an approved implementation plan:

1. Gap 10: add bounded byte codecs over the existing byte representation.
2. Gap 7: add a small, consistent argument parser.

These candidates provide immediate everyday value without requiring new
source-language features. Select one canonical API for each operation rather
than copying every variant from the comparison libraries.

Incremental file I/O and process control need separate designs covering ownership,
bounds and cancellation. Structured errors and the selected Std migrations are
implemented. The remaining candidates should be driven by concrete application
needs.

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
