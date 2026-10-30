# Plan: Shared UI for Terminal and Graphics

Status: planned direction; design and implementation remain open. This document
records the intended scope and sequence, not permission to start implementation.
The user accepts rewriting all existing TUI applications, including the dungeon
client and server console, for the new API. No new FPAS syntax is assumed;
syntax changes require explicit user approval.

The binding [namespace specification](../specs/std-namespaces.md) allocates
`Std.Ui` for the common interface and `Std.Ui.Terminal` / `Std.Ui.Graphical`
for the hosts. The existing terminal interface is `Std.Ui.Terminal`.
The common interface and graphical host remain future work under this plan.

## Sequence and relationship to other plans

The intended order is:

1. Continue and finish the intended [FPAS 2 work](fpas2-language-directions.md),
   with each topic retaining its own status and decision process.
2. Proceed with graphics and multimedia foundations as described in the
   [graphics application case](graphics.md) and the
   [alternative foundations](graphics-alternative.md).
3. During that graphics work, design and implement this shared UI and migrate
   the existing terminal applications. A working graphical host is part of the
   UI plan, not a later assumption.

This order does not mean every proposal in FPAS 2 has been approved, nor that
all open graphics questions already have answers. FPAS 2 remains the source of
truth for shared language and runtime decisions; the graphics documents own
native rendering, audio, platform and delivery concerns. This document owns the
UI and migration plan. Record changes to shared foundations in their owning
document and link them here instead of creating competing designs.

**Coordination before graphics:** while working on FPAS 2, consult this plan
before settling decisions that affect UI hosts, particularly
[stores and selectors](fpas2-language-directions.md#19-application-stores-and-selectors),
[selective updates](fpas2-language-directions.md#20-selective-tui-updates),
[buffers](fpas2-language-directions.md#22-packed-numeric-data-and-binary-buffers),
[clocks](fpas2-language-directions.md#10-distinct-duration-and-clock-types),
and [native event loops](fpas2-language-directions.md#25-native-event-loops-and-selectable-host-events).
The terminal host's [cached subviews](../pascal/std/tui/subviews.md) use explicit
typed dependency values and host-owned cache lifetimes. Carry that ownership and
dependency contract into the common UI design; it does not require automatic
tracking or a particular rendering backend.

This is a design review point, not a request to implement the UI early or to
make every listed proposal a prerequisite. If a concrete FPAS 2 decision makes
the planned UI impractical, record the conflict and revisit the sequence before
locking in that decision.

## Goal and boundaries

An FPAS application can describe a semantic UI tree and run it with a terminal
host or a graphical host. Model, Update, View, actions, controlled inputs and
background messages provide the common application shape. Applications may
share a View where useful or provide different Views over the same model.

The intended common layer contains semantic elements, layout descriptions,
validation and interaction contracts. Each host supplies its own measurement,
input integration, drawing, presentation and lifecycle. Share layout algorithms
where evidence supports it; a common API does not require forcing identical
internal representations onto both hosts.

The shared UI is not a browser, a complete CSS implementation, a game engine,
an audio system or a 3D renderer. GUI-specific images, canvas and 3D viewports,
and terminal-specific cell grids, need explicit host-specific extensions. A
cross-host View must provide an explicit fallback where an extension has no
equivalent; the host must not silently pretend it supports that capability.

## API direction

Explore an immutable element tree in `Std.Ui`, using the allocated host units.
Member names, signatures and host selection are open. Use the existing FPAS
language and evaluate the design against real application Views before freezing it.

- Provide text, buttons, inputs, text areas, check boxes, lists, menus, panels,
  scroll views and dialogs with explicit identities and actions where needed.
- Retain controlled values and serial updates. Define focus, modal routing,
  selection and keyboard/pointer behavior across hosts and document differences.
- Use CSS-inspired flex layout: rows and columns, gap, padding, margin,
  alignment, grow, shrink, basis and minimum/maximum sizing. Add grid for
  forms and other two-dimensional layouts. Specify overflow and scrolling.
- Use semantic styles and spacing tokens that each host's theme resolves.
  Specify units for exact terminal and graphical dimensions; do not silently
  equate a terminal cell with a pixel. Define behavior when a View uses an
  unsupported host-specific unit or element.
- Define intrinsic measurement, constraint propagation, text measurement and
  deterministic terminal-cell rounding and remainder distribution. Portability
  means preserved semantics, not identical geometry or appearance.
- Keep background commands, bounded message delivery, cancellation, ticks,
  shutdown and headless operation explicit. Preserve the existing terminal
  host's behavior unless an intentional replacement is documented and tested.

GPUI is an inspiration for a composable, CSS-like API, not a commitment to its
implementation or to CSS compatibility. A CSS parser, selectors and cascade
are not part of the initial scope.

## Work packages

### 1. Inventory and contract

Inventory terminal UI features, public interfaces and callers, including
examples, tests, benchmarks, documentation and tools in both repositories.
Use `Std.Ui.Terminal`; its pre-migration name was `Std.Tui`.
Cover menus, modal and focus routing, pointer and keyboard input, text editing,
themes, cells, background work, resize and incremental rendering. Map each to a
shared contract, a host extension or an intentional replacement. Design example
Views using Notes and the dungeon client to expose missing capabilities early.

### 2. Layout and common core

Define and test flex, grid, measurement, styles, host-specific units and the
element, message and host contracts. Separate application work, View creation,
layout, painting and presentation so later selective updates can follow the
FPAS 2 direction without assuming automatic dependency tracking. Keep geometry
and paint invalidation correct for content, focus, pointer, resize, modal and
theme changes. Measure representative workloads before making performance claims.

### 3. Terminal host

Implement the terminal host using the new contracts, including cell-grid custom
drawing, terminal themes, input and headless tests. Preserve application-visible
behavior and terminal cleanup, cancellation, queue ordering and responsiveness;
verify intentional layout changes against actual use. Retain efficient layout
reuse and damaged-cell presentation where justified by measurements.

### 4. Graphics foundation and graphical host

Choose the native foundation through the graphics work, comparing the pure Rust
route and SDL3 against the intended platforms, text, 2D, 3D, audio and delivery
requirements. The current preference for pure Rust remains a preference, not a
backend decision. Integrate with the selected FPAS 2 event and resource
contracts; do not assume a general FFI or new numeric scalar types is required.

Deliver a real first graphical host with window lifecycle, text measurement and
rendering, 2D drawing and clipping, keyboard and pointer input, focus, scrolling,
dialogs and resize. Handle native event-loop ownership, wakeups, background work,
resource cleanup and presentation. Address optional graphics capabilities,
runner selection and asset packaging in coordination with the graphics plan.

Run the same small form, list and dialog View on both hosts before migrating all
applications. Check text, sizing, keyboard, pointer, focus and resize. If common
abstractions become awkward or require excessive exceptions, revise the contract
before proceeding; separate Views over a shared model remain valid.

### 5. Application migration

Migrate examples and headless fixtures first, then Notes, Local Chat, Fpastris
and Mandelbrot. Include application tests, demos, documentation, benchmarks,
project manifests and tools that use the old API.

Migrate `fpas-into-the-dungeon` as well: server console and client, including
menus, world management, character sheet, inventory, journal, trade, dialogue,
maps, raycasting, animation and associated tests. First preserve the playable
terminal experience using terminal-specific extensions where needed. A new
graphical dungeon presentation or 3D game is separate graphics/game work; this
plan does not imply that replacing the UI API creates one automatically.

Backwards compatibility with the old TUI API is not required. Temporary migration
support may be used if useful, but remove it and the old public API after all
callers have moved and the replacement is verified.

### 6. Acceptance and documentation

- Exercise flex/grid constraints, intrinsic sizing, clipping, scrolling,
  rounding, narrow terminals and graphical resize.
- Test keyboard and pointer routing, focus, menus, dialogs, text editing,
  selection and supported host-specific extensions.
- Verify message ordering, bounded delivery, fairness, background work,
  cancellation, failure handling, shutdown and resource cleanup.
- Compare headless results and interactive behavior on both hosts; exercise a
  common View and explicit host-specific Views or fallbacks.
- Run relevant FPAS and dungeon suites, including dungeon client and server
  workflows, network behavior and generation fingerprint tests. Preserve game
  behavior except for intentional, reviewed UI changes.
- Compare representative performance against the existing TUI and investigate
  regressions; do not promise an improvement without measurements.
- Update current manuals only for delivered behavior and reconcile this plan,
  graphics notes and FPAS 2 status as work is completed.

## Open decisions

Public member names and builders; supported flex/grid subset and default sizing;
precise units and rounding; accessibility and input-method behavior; supported
platforms; graphical text and renderer; backend choice; host capabilities and
fallbacks; shared versus host-specific input and layout implementation; theme
mapping; and packaging all need concrete decisions during the work. No schedule
or backend selection is implied by this plan.
