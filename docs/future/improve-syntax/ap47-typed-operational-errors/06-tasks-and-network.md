# AP47.6: Tasks, networking and layered consumers

Status: Complete. Authorized by the AP47.6 implementation request; AP47.5 dependency satisfied.
Selected interfaces: [task/network catalog](standard-errors.md#task-and-networking-cohort)
and [explicit mappings](standard-errors.md#source-boundary-mappings-and-migration-inventory).
Completion: [AP47](README.md#dependencies-and-work-packages).

## Deliverable

Switch current Result-returning Tasks, Net, Server, Uri, Utf8 and HTTP interfaces
to the selected typed families, including hidden callback/selection Outcomes,
streaming helpers and every affected caller. Native failure and task lifecycle
mechanisms retain their existing scheduling, ownership and cleanup behavior.

## Implementation plan

- Define native TaskError, NetError and ServerError families in their existing
  registry owners. Extend shared HostError construction from AP47.5 where a real
  native failure exists. Classify closure, expiry and cancellation at the owners
  that already decide the outcome, before formatting.
- Change Send/Receive, their Try/WithTimeout/WithCancellation forms, selection
  callback Outcomes and generic WaitAny functions consistently. Do not stop at
  the top-level return type: types inside procedure parameters and WaitCase
  metadata must agree with runtime callback values.
- Preserve ready/timeout/cancellation precedence, draining, try-operation success
  values, task-group ownership after timeout, single result consumption, blocked
  worker behavior and existing TaskFailure records. An invalid native task or
  compiler/runtime invariant is not made recoverable by wrapping it in TaskError.
- Source Uri/Utf8 errors are declared next to their data/decoder operations.
  Utf8.Decode maps its existing truncated-sequence check to IncompleteSequence
  and its other byte/sequence checks to InvalidByte. There is no Utf8.Finish
  operation to migrate. Preserve current EOF successes and byte conventions;
  do not reimplement the parsers or add a UTF-8 streaming API.
- Define HttpError in the existing Http.Types leaf and re-export it through
  Std.Http. Migrate Client, Stream, Wire, Head, Io, Fields, BodyFraming, Redirect,
  Sse, Server and ServerLoop together. Map lower-level typed causes at their
  actual ownership seam; protocol errors use their own selected cases.
  Sse.Feed/Finish stay HTTP operations: map failures from their existing
  Utf8.Decode calls to HttpError.Encoding with the original typed Utf8Error.
  Preserve buffering across Feed chunks and final-line processing at Finish.
- Update Server lifetime behavior and listener ownership for NetError while
  preserving shutdown budgeting, signals and escalation. No new process/network
  feature or hard cancellation guarantee is introduced.
- Migrate all transitive source libraries, apps, tests, embedded scripts and
  fixtures in the same delivery. SDK/app boundaries that intentionally retain
  strings convert explicitly there; selected Std APIs never ship both variants.

## Implementation owners

| Path | Responsibility |
| --- | --- |
| `crates/fpas-sema/src/std_registry/loaded/channel_task.rs`, `net.rs`, `server.rs` | Families, exact signature shapes and nested callbacks |
| `crates/fpas-std/src/std_units/symbols/std_symbols/` | Task, network and server symbol inventories |
| `crates/fpas-vm/src/vm/tasks/`, `src/vm/hosted/net/`, `src/vm/hosted/server/`, `src/vm/layouts.rs` | Outcome detection, native factories and signal/listener lifetime ownership |
| `crates/fpas-vm/src/vm/debug/`, `crates/fpas-debug/` | Seeded scheduling, trace/failure-capture integration with unchanged runtime semantics |
| `lib/Std/Net/Uri.fpas`, `lib/Std/Net/Utf8.fpas`, `lib/Std/Http/` | Source error declarations, wrappers and protocol classification |
| `lib/Std/Http.fpas`, `lib/api/Std/`, consumers under `lib/`, `apps/`, `examples/`, `tests/` | Public re-exports, generated declarations and consumer migration |
| `docs/pascal/std/network/`, `std/concurrency/`, associated examples | Delivered domain cases, successes, EOF and lifecycle distinctions |

Do not add per-instruction error/report polling to implement failure propagation
or callback formatting. Formatting is requested only by output consumers;
normal task dispatch continues to move ordinary Result/error values.

## Acceptance and verification

Extend the actual native channel/wait/selection owner fixtures to cover closed,
timeout, cancelled and invalid inputs by typed cases, including callback payloads.
Use deterministic readiness/control hooks already owned by the test harness;
avoid arbitrary sleeps. Separate one targeted fixture for each genuine lifecycle
path: source callback, scheduled callback, runtime supervision and owned-group
cleanup. Retain debugger trace bounds and write-task identity coverage.

Network integration covers TCP round trip/EOF, refused/reset connections where
the fixture can force them, cancellation, timeouts and TLS verification failure.
Assert original HostError details for native failures without asserting portable
numeric codes across OSs. URI and UTF-8 fixtures retain grammar and byte offsets.
HTTP retains ordinary non-2xx response success, partial reads, body limits,
redirect validation and SSE end-state behavior. UTF-8 truncation through Decode
is IncompleteSequence; the same condition reached by Sse.Finish is
HttpError.Encoding with that typed cause. Reuse incremental SSE fixtures to show
a valid multibyte sequence split across Feed chunks still succeeds and a final
truncated line fails at its existing boundary. Server shutdown remains owned
and reports the selected typed outcome under the existing shared budget.

Tests must inspect selected variants and typed causes rather than classify text.
No direct test of a fake reporter supplies the behavior being asserted. Run
the relevant crate and full language/Std suites, affected apps/workspaces,
generated-declaration checks, editor queries and the appropriate VM/concurrency/
TUI/network benchmark comparisons from AP47.7. Local sockets and TLS fixtures
need real permitted access before sandbox failures can be interpreted.


## Delivered behavior and proof

The native catalog, semantic signatures, generated APIs, cached factories and
image admission now include TaskError, NetError and ServerError. Runtime layout
retention covers values constructed without source constructor references,
including ShutdownErrors rendering of retained listener failures. Server causes
retain NetError and HostError dependencies. Invalid native task identities and
runtime invariants remain diagnostics; TaskFailure records remain unchanged.

UriError and Utf8Error are source families at their parser/decoder owners.
Http.Types owns HttpError, re-exported by Std.Http; all selected HTTP helpers and
callbacks retain typed causes. SDK and TUI string contracts map Message()
explicitly. Apps, standalone examples, project examples, source fixtures and
Rust-embedded programs are migrated together.

Owner tests cover channel cancellation/timeout/closure, selection outcomes,
large controlled waits and group ownership. Local TCP/TLS, listener state,
actual signal-handler conflict, URI/UTF-8/SSE boundaries, portable images and
cached/source callers exercise production paths. The subagent's missing final
zero-chunk finding is repaired as UnexpectedEnd and verified through the real
private Wire owner in source and portable-image routes. Review also checked the
consumer migration and the event-collection repair in debugger protocol tests.

Verification includes the workspace build, tests, warning-free all-feature
Clippy, Rust/Pascal formatting, API regeneration reproducibility, all affected
app/project manifests, executable handbook examples and the editor grammar.
The final workspace run passed 4894 tests with three existing ignored tests;
the Pascal suite passed 535 with one intentional skip and includes all 520
executable documentation examples. All 24 app/project checks and the standalone
network example directory passed.
The [original 72-workload comparison](../../../bench/ap47-tasks-network.md)
records three unchanged before/after samples per row and any isolated controls.
AP47.7 completes [final cross-package acceptance](07-acceptance-and-performance.md#integrated-delivery-record)
and registers the new errors benchmark group after this original-suite comparison.
