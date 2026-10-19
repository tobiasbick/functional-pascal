# Standard library gap review

Status: assessment only, reviewed on 2026-10-19. No implementation is scheduled.

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
grapheme segmentation and display widths. These are not missing capabilities.

## Everyday capability gaps

Numbers are stable references within this assessment, not approved work packages.

| ID | Area | Current boundary and missing capability | Comparison |
| --- | --- | --- | --- |
| 1 | End of text input | `Console.ReadLn()` cannot return EOF explicitly. The ordinary stdin implementation ignores the zero-byte read indication, so EOF and an empty line are indistinguishable to FPAS code. This prevents a reliable read-until-EOF loop over a pipe. | Go `io.EOF`; Rust `BufRead` |
| 2 | Binary files | `Fs` reads and writes UTF-8 text but exposes no byte read/write operations. Existing network/Crypto byte arrays could supply an initial representation without adding language types. | Go `os.ReadFile`/`os.WriteFile`; Python binary files |
| 3 | Incremental file I/O | No public file handles for bounded chunk reads/writes, line-by-line reading or seeking. `ReadText` loads the whole file and rejects files over 64 MiB. HTTP response streaming already exists but does not cover files. | Go `io`/`bufio`; Rust file I/O |
| 4 | Process control | `Run` and `RunCapture` wait for completion. Missing capabilities include supplying stdin, a working directory and environment per invocation, timeout/cancellation, and bounded or incremental captured output. Moving a call into `go` does not add child-process control. | Go `os/exec`; Python `subprocess` |
| 5 | Calendar date and time | `Time` exposes wall-clock/monotonic milliseconds and sleep, but no calendar decomposition, date values, ISO-8601 parsing or date/time formatting. Time-zone handling would be a further design step. | Go `time`; Ada `Calendar.Formatting` |
| 6 | Numeric output formatting | String `Format` supports `%d`, `%f`, `%s` and `%%`, without selectable decimal precision, scientific notation or numeric field width. `RealToStr` provides exact round-trip text rather than presentation controls. | Go `fmt`; Python formatting |
| 7 | Command-line parsing | `Args` exposes only argument count and indexed lookup. Applications must build typed options, required arguments, help output and consistent parse errors themselves. | Go `flag`; Python `argparse` |
| 8 | Path helpers | `Path` has five basic operations. Absolute-path checks, relative-path calculation and extension replacement are absent. Resolving a real path through filesystem/symlink state belongs in an effectful API such as `Fs`, preserving the pure `Path` boundary. | Go `path/filepath`; Rust `Path` |
| 9 | Structured errors | Many filesystem, network, process and parsing APIs return `Error(string)`. Programs cannot reliably branch on a stable error kind such as missing file, permission denied, timeout or cancellation. The [agreed FPAS 2 direction](fpas2-language-directions.md#1-typed-operational-errors) combines a shared error contract with composable concrete error types, explicit `MapError` conversion and unchanged `try`; shared error APIs and concrete variants remain open. | Rust `io::ErrorKind`; Go typed errors |
| 10 | Byte codecs | No general Base64/Base64url or hexadecimal encode/decode API for byte arrays. `IntToHex` handles individual numbers. Small codecs can use the existing integer-array byte convention; compact buffers are a separate question. | Go `encoding/base64` and `encoding/hex` |
| 11 | URL components | `Net.Uri` parses absolute HTTP/HTTPS addresses but exposes no query-parameter builder/parser or percent-encoding helpers for query values and path segments. Those contexts need distinct escaping rules. | Go `net/url`; Python `urllib.parse` |

### Evidence in FPAS

- [Console input implementation](../../crates/fpas-std/src/console/input.rs) and
  [public text input](../pascal/std/console/input.md): gap 1.
- [Filesystem API](../pascal/std/host/fs.md): gaps 2 and 3.
- [Process API](../pascal/std/host/proc.md) and
  [implementation](../../crates/fpas-std/src/proc.rs): gap 4.
- [Time API](../pascal/std/host/time.md): gap 5.
- [String formatting](../pascal/language/types/string/format-chars.md),
  [implementation](../../crates/fpas-std/src/str/format.rs) and
  [conversion](../pascal/std/text/conv.md): gap 6.
- [Arguments](../pascal/std/host/args.md) and
  [paths](../pascal/std/host/path.md): gaps 7 and 8.
- The Result signatures in the filesystem, process,
  [network](../pascal/std/network/net.md) and
  [parsing](../pascal/std/text/parse.md) APIs: gap 9.
- [Conversion](../pascal/std/text/conv.md),
  [Crypto](../pascal/std/cryptography/crypto.md),
  [UTF-8](../pascal/std/network/utf8.md) and
  [URI](../pascal/std/network/uri.md) APIs: gaps 10 and 11.

### Direction for gap 9: structured errors

The agreed direction is a shared error contract for context and reporting,
combined with concrete error types defined by standard-library units and
applications. Composition and variant payloads let an application wrap a
filesystem error alongside its own configuration errors while preserving the
original machine-readable case. Public Result signatures retain their concrete
error types; callers can branch exhaustively without inspecting diagnostic text.

Include `MapError` in the same design package. For `Result of (T, E)`, a mapper
from `E` to `E2` produces `Result of (T, E2)`: Ok preserves its payload without
calling the mapper, while Error calls it exactly once and wraps the returned
error value. The receiver is evaluated once. Unlike recovery through `OrElse`,
the mapper returns an error value rather than another Result.

Keep `try` unchanged: it extracts success or immediately propagates an error
through a compatible enclosing Result error type. Cross-type propagation uses
explicit `MapError` conversion before `try`, for example
`try Fs.ReadText(Path).MapError(WrapFileError)` with a function wrapping the
filesystem cause in an application error. No implicit conversion or automatic
wrapper selection is added. Option propagation, panic, contracts and cleanup
retain their existing behavior.

The [FPAS 2 error direction](fpas2-language-directions.md#1-typed-operational-errors)
owns this design and the
[MapError/try contract](fpas2-language-directions.md#agreed-maperror-and-try-contract).
Shared fields, unit-specific variants, per-API domain mappings and migration
remain open. This agreement does not schedule an implementation or introduce
object inheritance.

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

1. Gap 1: make EOF observable without treating a blank line as end of input.
2. Gap 6: add predictable numeric presentation controls.
3. Gap 8: add the small path helpers required by real tools.
4. Gap 10: add bounded byte codecs over the existing byte representation.
5. Gap 7: add a small, consistent argument parser.

These candidates provide immediate everyday value without requiring new
source-language features. Select one canonical API for each operation rather
than copying every variant from the comparison libraries.

Incremental file I/O, process control and structured errors deserve separate
designs covering ownership, bounds, cancellation and the public Result types.
The remaining candidates should be driven by concrete application needs.

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
