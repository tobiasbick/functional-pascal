# CLR TUI library gaps

Build an FPAS-owned CLR library with its TUI core written in FPAS and usable from
C# and F#. Ordinary .NET APIs and a small platform adapter are possible building
blocks. Public names, package layout, adapter dependencies and exact managed
signatures remain open. Follow [CLR mapping](../specs/clr-mapping.md) and
[interface rules](../pascal/interop/interfaces.md); new syntax needs approval.

## Library and application lifetime

- Define a library project, assembly/package identity, XML documentation,
  dependency/bootstrap rules and real C#/F# consumer verification.
- Expose generic model/message handling, Update/View callbacks, element factories,
  styles, geometry, effects, cancellation and inspection through ordinary CLR APIs.
- Define serial message routing, Update, effects, View, layout and presentation
  order without blocking input or background completion.
- Own asynchronous effects, timers and subscriptions. Route outcomes as messages;
  define replacement, cancellation, joining, failure observation and terminal
  restoration on every exit path.
- Specify model/dependency ownership. Read-only record properties and `const`
  do not freeze referenced arrays/objects or deep-clone updates.

View constructs a declarative tree; it does not own terminal handles or launch
background work. Background work must not mutate the live model or render buffers.

## Elements, layout and rendering

Provide text, buttons, inputs, multiline editors, checkboxes, lists, tabs,
scrolling, menus, status lines, panels, overlays, gauges and custom cell grids.
Define stable identities, actions, focus scopes, keyboard/pointer routing and
model-controlled editing/selection.

Specify measurement, arrangement, alignment, clipping, modal layout, resize,
semantic styles/themes and incremental layout/painting/output. Unicode geometry
must account for graphemes, terminal-cell width, surrogate pairs and wide-glyph
continuation cells. UTF-16 length is not display width.

Choose cell-buffer/hot-path representations with measured allocation and frame
costs. FPAS records are reference types; do not assume per-cell objects are cheap.

## Cached subviews

Use explicit typed dependencies, a subtree builder and equivalence comparison.
Build on first use or dependency change; reuse otherwise. Define:

- Complete dependencies including identities, actions, controlled state, styles
  and model-dependent geometry.
- Builders/comparisons without I/O or required per-update side effects.
- Per-application ownership, bounded retention, unused-entry removal and cleanup.
- Nested subviews, reentrancy, resize and dependency invalidation.
- Reuse preserving routing and interaction as well as appearance.

Mutable references cannot represent an immutable previous dependency snapshot.
Select revisions or value projections/comparisons and verify cache lifecycle.

## Headless testing

Use the interactive application's real Update, routing, View, layout and painting
with an in-memory cell target. Provide explicit stepping/condition waits,
message/frame completion and detached snapshots. Keep instances isolated.

| Level | Required evidence |
| --- | --- |
| Model | Independent model/effect expectations for messages. |
| Headless UI | Real keyboard/pointer/resize/background routing; focus, state, layout, text, colors and cells. |
| Timing/effects | Controlled clocks/results, deadlines, cancellation and failures. |
| Terminal | Emitted sequences and actual interaction in controlled PTY/ConPTY sessions. |

Logical cell assertions do not establish actual terminal output. Both need proof.

## Live inspection and automation

An optional, explicitly enabled local connection should expose versioned snapshots
after processed updates/frames, a semantic control tree, the rendered cells,
pending-work/failure summaries and bounded event history. Control descriptors
need stable IDs, roles, labels, bounds, focus, enabled state and exposed values.

Queue automation through real input routing. Selecting a control must preserve
focus, hit testing and disabled-state checks. Define concurrent human/client input
ordering, condition waits, deadlines, queue bounds, cancellation and disconnect
behavior. Snapshots must not expose writable live state. Keep application-model
summaries separate from generic UI inspection.

Live operation does not establish deterministic replay or actual terminal display;
retain controlled headless and terminal tests.

## Terminal adapter and acceptance

Define raw input, keyboard/mouse events, resize, alternate screen, cursor state,
capabilities and restoration. Select Windows/Linux adapters for library input
modes and restoration, with explicit native-resource ownership. Reuse the
[controlled terminal runner](../pascal/tools/terminal-testing.md) and
Fpas.Processes.Terminal for private PTY/ConPTY hosting.

Verify FPAS library builds, C#/F# consumers, independent instances, MVU/routing,
cache lifecycle, effect cleanup, Unicode geometry, terminal restoration, headless
assertions and live snapshot/frame consistency on Windows and Linux.

The [CLI reference](../pascal/program-structure/cli.md) owns command workflows. The [debugger reference](../pascal/tools/debugging.md)
owns the DAP and batch protocols. The [VS Code extension](../../src/VSCode/README.md)
owns its editor workflows.
TUI live inspection is a library capability, separate from DAP.
