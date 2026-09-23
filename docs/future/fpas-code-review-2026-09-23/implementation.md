# FPAS review implementation

Implementation base: `34de88bc`. This follow-up addresses the FPAS review while
preserving the existing language syntax, visibility rules, and intrinsic APIs.
The original review remains the record of `b3d3b84c`.

| Finding | Implementation and regression |
| --- | --- |
| F01 | Escape only the literal Notes directory before appending the note glob. Tests cover bracketed, unmatched-bracket and spaced directories, sibling files, retained paths, and subsequent saves. |
| F02 | A successful save clears the save-failure overlay and error while preserving other overlays. A failure/repair/retry test then selects another note and quits. |
| F03 | All files sharing a persisted ID become visible issues; none enters the editable collection. Tests verify both original files survive selection, saving and archive actions unchanged. |
| F04 | Redirect dot-segment removal preserves empty segments and terminal separators. Loopback tests assert exact request targets, including relative, query and fragment references. |
| F05 | A shared protocol-decimal parser rejects non-ASCII digits, signs, separators and overflow before arithmetic. Client and server loopback tests cover invalid framing and valid boundary handling. |
| F06 | Scheme validation runs independently of port presence. The URI matrix covers HTTP/HTTPS, unsupported schemes, IPv6, explicit empty ports, leading zeros and range boundaries. |
| F07 | Feed processes bounded input slices and clears retained bytes and event fields after any decoding error. Feed and Finish errors make the decoder terminal. Tests cover many events per fragment, split CRLF, repeated calls, and retained state through a fixture-only accessor. |
| P01 | A private runtime encoder emits UTF-8 bytes in one pass. The source decoder preserves validation and joins scalar fragments once. Public APIs are unchanged; malformed byte and scalar-boundary regressions were added. |
| P02 | TUI queues use unread cursors with geometric compaction, release emptied storage, and append through an owned local array. Consumers use unread counts. Burst tests cover FIFO ordering, interleaved enqueue/drain and separate handles. |
| P03 | Display-column lookup splits lines once and traverses grapheme clusters; local prefix checks preserve scalar-index behavior near the requested column. Tests compare with the previous contract for combining marks, wide characters, joined emoji and presentation selectors. |
| P04 | `Notes.Ordering` provides stable merge sorting. Pin/timestamp priority, equal-key stability and selected identity are covered. |
| M01 | `Fractal.Camera` and `Fractal.Explorer` share geometry, input and terminal lifecycle. Burning Ship and Tricorn retain separate renderers, palettes and reset settings. Both consuming projects are checked. |
| M02 | `Notes.Actions` owns control and action identifiers used by Model, Editing, Update and View. Existing headless workflow tests still route real view-produced events. |
| M03 | HTTP setup uses typed `try` bindings before connection ownership. Later cleanup branches remain intact. |
| D01 | Both Markdown math units export `Add`. A Rust integration test extracts and compiles the actual documented projects. |
| D02 | The example index describes lifetime cancellation, the remaining shutdown deadline and watchdog escalation. |

## Performance evidence

The `source-review` and `source-navigation` benchmark groups capture before/after
measurements at doubling input sizes. Inputs are prepared before timing. Each
fixture runs ten timed repetitions; process startup and compilation are excluded.
The queue fixture includes public headless rendering/routing, and navigation
includes layout and rendering. Their deltas measure those complete workloads.

The baseline was saved before changing UTF-8, sorting, queue representation or
caret lookup. Final comparisons measured the following elapsed times (milliseconds):

| Workload | Size | Before | After |
| --- | ---: | ---: | ---: |
| UTF-8 ASCII | 2048 | 25 | 16 |
| UTF-8 ASCII | 4096 | 52 | 32 |
| UTF-8 ASCII | 8192 | 114 | 64 |
| UTF-8 multibyte | 2048 | 103 | 64 |
| UTF-8 multibyte | 4096 | 226 | 132 |
| UTF-8 multibyte | 8192 | 559 | 261 |
| Notes sort | 128 | 52 | 6 |
| Notes sort | 256 | 211 | 14 |
| Notes sort | 512 | 835 | 31 |
| TUI queue | 256 | 111 | 113 |
| TUI queue | 512 | 240 | 234 |
| TUI queue | 1024 | 582 | 445 |
| Navigation, long lines | 512 | 68 | 68 |
| Navigation, long lines | 1024 | 131 | 132 |
| Navigation, long lines | 2048 | 263 | 259 |
| Navigation, many lines | 512 | 15 | 15 |
| Navigation, many lines | 1024 | 25 | 25 |
| Navigation, many lines | 2048 | 44 | 44 |

At the largest sizes, ASCII and multibyte roundtrips improve by 44% and 53%,
sorting by 96%, and queue bursts by 24%. The smallest queue case is 1.8% slower;
no reliable gain is claimed at that size. Navigation timings are effectively
unchanged in these combined workloads. The implementation removes repeated
whole-line prefix measurement, but still checks scalar prefixes inside a cluster
near the target column to preserve existing Unicode caret behavior. It does not
establish linear behavior for arbitrarily long individual grapheme clusters.

All 60 registered benchmark workloads also completed with `cargo bench-fpas run`.
There is no matched before snapshot for the other groups, so this full run checks
execution rather than proving unchanged performance outside the targeted groups.
Separate settled runs of both new groups were recorded in
[`docs/bench/history.md`](../../bench/history.md); small timing differences between
the comparison and recorded runs are expected.

## Verification

Validation runs on Windows, with one Cargo build job and no concurrent Cargo builds.
No interactive terminal session or external network service is needed by the new tests.

- `cargo fmt --all -- --check` and `git diff --check`: passed.
- `cargo build --workspace --locked --offline -j 1`: passed.
- `cargo test --workspace --locked --offline -j 1 --no-fail-fast -- --test-threads=1`:
  3175 tests exercised. Two inventory assertions initially needed updated counts
  for the new benchmark groups and Notes references; all other tests passed.
  Both affected integration targets then passed in full: benchmark CLI (7 tests)
  and language-service repository context (9 tests). No failing tests remain.
- `cargo clippy --workspace --all-targets --all-features --locked --offline -j 1 -- -D warnings`:
  passed.
- `fpas test --jobs 1 --report json tests/suite.fpasprj`: 445 passed, 1 skipped,
  0 failed, 446 total. Nine FPAS regression files and five Rust integration tests
  were added.
- `fpas check`: Notes, Local Chat, Burning Ship and Tricorn projects passed.
- `fpas fmt --check`: all changed FPAS files and `apps examples lib` passed.
  Checking all of `tests` also reports two unchanged debugger fixtures:
  `tests/debugger/fixtures/debugger_target.fpas` and
  `tests/debugger/fixtures/expression_mutation.fpas`.

These are standard-library and application fixes, internal refactors, and
documentation corrections. User-facing updates cover network parsing/UTF-8,
Notes behavior, the project examples, and example shutdown. No language syntax
or semantics changed. Interactive terminal behavior was covered through headless
tests and project checks, without a manual terminal session.
