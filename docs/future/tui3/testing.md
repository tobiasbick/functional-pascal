# Std.Tui3 testing contract

Std.Tui3 is designed for deterministic headless verification from the first implementation
phase.

## Headless host

Tests create a host with `OpenForTest`, supply `Update` and `View`, inject `TuiMsg`
values, and advance with `RunIterations` or single-step injection. Terminal modes stay
unchanged.

Headless and interactive paths share layout, paint, and message rules.

## Phase 0 feasibility suite

Implementation may not proceed to the value-porting phase until one FPAS vertical slice proves:

- the exact generic host, `Update`, `TuiCmdOutput`, and `View` signatures compile;
- nested builders preserve a controlled input, button, and modal dialog as a recursive child tree;
- the initial surface exists before the first injected key or pointer is routed;
- focus change, text change, activation, and quit messages run in the documented FIFO order;
- duplicate `TuiControlId` values fail while duplicate `TuiAction` values remain valid;
- repeated frames at representative tree and terminal sizes avoid repeated full-tree and full-grid
  clones during ordinary traversal and paint.

Record the tree sizes, terminal sizes, iteration count, timings, and any allocation/clone evidence
with the implementation. Compare increasing tree sizes to catch accidental superlinear copying. If
aggregate cloning dominates, shared or copy-on-write VM values become a prerequisite rather than a
Tui3-local retained-view workaround.

Phase 0 baseline (2026-07-21, Windows development build):

| Tree | Terminal sizes | Iterations | End-to-end time |
| --- | --- | ---: | ---: |
| 43 nodes (32 labels, 8 buttons, column, window, desktop) | 40×12 and 120×40 | 100 each | 14.74 s |

The measurement uses `tests/stdlib/tui3/repeated_frames_test.fpas` and includes parsing, compilation,
VM startup, and both runs. The element walker contains no explicit full-tree clone. The working
surface uses direct nested `array of TuiCell` rows with parallel continuation rows. VM arrays use
copy-on-write storage, and global indexed writes update the owned grid without rebuilding it; the
full glyph array is constructed only by explicit `Snapshot` calls.

## Phase 6.3 checkpoint

Checkpoint (2026-07-22, Windows development build; retired IDE excluded):

| Evidence | Result |
| --- | --- |
| Repeated-frame canary | 43 nodes (32 labels, 8 buttons, column, window, desktop); 100 frames each at 40×12 and 120×40; 32.20 s end to end. |
| Complete Tui3 FPAS suite | 45 passed, 0 failed in 37.52 s. |
| Terminal restoration | `cargo test -p fpas-std console::tests::interactive`: 3 passed. |
| Rust workspace checkpoint | `cargo test --workspace`: passed in 109.8 s. |
| Formatting | `cargo fmt --check` and `fpas fmt --check tests/stdlib/tui3 examples/pascal/tui3`: passed. |

The repeated-frame canary traverses and paints new element trees without requesting a surface
snapshot during the frame loop. It retains the Phase 0 ownership boundary: element traversal has
no explicit full-tree clone, the working surface is copy-on-write storage, and `Snapshot` remains
the only public full-grid copy. The current Tui3 documentation covers every public value, element,
host operation, and interactive-terminal operation exported by `lib/Std/Tui3.fpas`.

## Determinism

- Same display-width and clipping rules headless and live.
- Stable leftover-cell allocation order.
- Initial render happens before input and outside the iteration budget.
- Injected messages are FIFO.
- Messages produced by one physical input are FIFO; pending messages precede new input.
- Tests use a virtual monotonic clock for ticks and timeouts.
- No test depends on wall-clock sleeps or an interactive terminal.

## Assertion levels

| Level | Examples |
| --- | --- |
| Pure values | Geometry, policies, palette resolution, control/action ids. |
| Update | Given model + message → expected model and command. |
| View | Given model → expected element shape (title, actions, conditionals). |
| Layout | Minimum/preferred allocation, nesting, clipping, resize. |
| Routing | Focus target, modal subtree, action message from key/pointer. |
| Screen | Explicit snapshots, exact cells, Unicode width, colors, wide-glyph repair. |
| End to end | Small FPAS programs driven only by injected messages. |

Prefer testing `Update` and `View` as pure functions before full host loops.

Every public interactive element needs at least one keyboard test, one pointer test when
applicable, one layout/resize test, and one screen-output test.

## Failure canaries

- Worker-task host calls report that worker delivery is unsupported in v1.
- Panic inside `Update` / `View` / paint leaves a clear diagnostic.
- Terminal-open rollback on failure.
- Invalid control/action id construction and duplicate control ids.
- Quit stops before another `View`, paint, or flush.
- Snapshot-free frames do not copy the full working surface.

## What not to test as public API

- Generational handle reuse (Tui2 concept — absent).
- Widget event-property assignment order.
- `OpenModal` object state.

## Suite placement

FPAS regression tests live under `tests/stdlib/tui3/` once code exists, bundled through
[`tests/suite.fpasprj`](../../../tests/suite.fpasprj). Examples, if any, go under
`examples/` and are not named `*_test.fpas`.
