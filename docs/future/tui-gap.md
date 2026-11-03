# CLR TUI library gaps

This document records the missing FPAS-owned terminal UI library for the CLR
toolchain. It is a gap inventory, not an implementation plan. It does not
authorize implementation, schedule work, fix public names/signatures, or approve
language changes.

The intended solution is part of FPAS, with its TUI core written in FPAS and
usable from other CLR languages. The target is a separate CLR library owned by
the FPAS project, rather than a facade over another TUI framework. Ordinary .NET APIs and
a narrowly scoped platform adapter remain possible implementation building blocks.
The exact proportion of FPAS and C# code is unverified.

The Rust references are the
[MVU application contract](https://github.com/tobiasbick/functional-pascal/blob/0a4bc134ee71b5f922ebd0b1157ab55244bef657/docs/pascal/std/tui/application.md),
[element model](https://github.com/tobiasbick/functional-pascal/blob/0a4bc134ee71b5f922ebd0b1157ab55244bef657/docs/pascal/std/tui/elements.md),
and [cached subviews](https://github.com/tobiasbick/functional-pascal/blob/0a4bc134ee71b5f922ebd0b1157ab55244bef657/docs/pascal/std/tui/subviews.md).
Their ideas are relevant; their VM handles, runtime intrinsics, source syntax,
and public names do not establish the CLR library's contract.

## Current foundations and missing library

The CLR branch has no equivalent TUI library, terminal host, or TUI test driver.
The implemented language already supplies useful foundations:

- Records, unions, enums, generics, delegates, and closures for models,
  messages, declarative trees, and callbacks.
- Actual CLR arrays/collections, Task/ValueTask consumption, cancellation,
  channels, exceptions, and synchronous/asynchronous resource cleanup.
- Public FPAS types and methods consumable from C# and F# through ordinary CLR
  metadata and project references.
- The project SDK's library output, dependency handling, portable symbols,
  and optional XML API documentation.

These foundations are described by the [CLR mapping](../specs/clr-mapping.md),
[record semantics](../pascal/language/types/records.md), and
[async/cancellation rules](../pascal/language/concurrency/cancellation-and-coordination.md).
Existing [record](../../tests/Fpas.IntegrationTests/Build/MsBuildTests.Records.cs),
[delegate](../../tests/Fpas.IntegrationTests/Build/MsBuildTests.Callables.cs),
[union](../../tests/Fpas.IntegrationTests/Build/MsBuildTests.Unions.cs), and
[mixed-language](../../tests/Fpas.IntegrationTests/Build/MsBuildTests.Baseline.cs)
tests exercise general interoperability. They do not verify a TUI implementation.

No new source syntax appears necessary for the proposed core capabilities.
Compilation, terminal behavior, performance, and the eventual public library
contract still need actual verification.

## Library ownership and CLR consumers

The missing library would have its own FPAS library project, assembly, and
distribution identity within the repository. `Fpas.Tui` is a possible working
name, not a selected namespace or package contract. TUI applications would
reference that library. Compiler components would retain their existing
responsibilities, and Fpas.Core would remain support for emitted language features.

Open gaps include:

- A public CLR interface expressed through ordinary types, methods, delegates,
  Task results, and CancellationToken parameters.
- Generic model/message handling so a C# or F# caller can supply its own model
  and message types and pass Update/View callbacks.
- Public element factories, styles, geometry, commands, application lifetime,
  and inspection values usable without FPAS-specific tooling at runtime.
- Deliberate naming for public types nested in FPAS unit containers, following
  the current mapping rather than assuming namespace-level source types.
- Assembly/package dependencies, XML documentation, SDK bootstrap, and
  cross-language consumer verification through real project builds.

FPAS cannot yet author arbitrary interfaces or user-defined class hierarchies.
Existing CLR interfaces can be implemented within the supported contract.
Delegates, records, and composition can express functional extension points
without adding source interface syntax. Extensibility and any need for separate
CLR contract types remain design questions, not approved compiler extensions.

## MVU and application lifetime

| Capability | Missing CLR library behavior |
| --- | --- |
| Model and typed messages | A generic application model and typed framework/application events with a defined ownership contract. |
| Update | Serial message handling that returns the next model and describes requested effects. |
| View | Declarative element-tree construction from the current model. View does not own terminal handles or start background work. |
| Effect execution | Host-owned asynchronous work returning outcomes through the application message queue. |
| Event ordering | Defined routing, Update, command handling, View, layout, and presentation order. Input and background work must remain responsive. |
| Lifetime | Explicit startup, stop, cancellation, joining of owned work, failure observation, and terminal restoration. |
| Timers and subscriptions | Explicit scheduling and cancellation/replacement of sources without blocking the UI loop. |

Model changes, routing, layout, and rendering belong to the serial application
loop. Background work communicates through messages rather than mutating the
live model or rendering buffers directly. Channels and CancellationTokens are
available building blocks; the host's ownership and failure rules are missing.

An immutable-model contract needs attention to CLR reference semantics. FPAS
records have read-only properties, but referenced arrays and objects may remain
mutable. Assignment and record updates do not deep-clone data. Const alone does
not make an application model or dependency snapshot immutable.

UI-owned work is part of the TUI application's lifetime. This does not reopen
the discarded proposal for a general FPAS task-group/supervision library or a
new `Std.*` wrapper catalog.

## Elements, layout, and rendering

The Rust library describes immutable elements and keeps runtime focus, layout,
and painting in its host. The CLR library lacks corresponding support for:

- Text, buttons, inputs, multiline editors, checkboxes, lists, tabs, scrolling,
  menus, status lines, panels, overlays, gauges, and custom cell grids.
- Stable control identities, application actions, focus scopes, keyboard
  navigation, pointer routing, and model-controlled editing/selection state.
- Measurement, arrangement, alignment, clipping, modal layout, and resize.
- Semantic styles/themes, palettes, and terminal color/attribute presentation.
- A working cell surface, frame snapshots, and incremental layout/painting/output.
- Unicode-aware text geometry, including graphemes, terminal-cell width,
  surrogate pairs, and continuation cells for wide glyphs.

The CLR string representation is UTF-16. String length and character indexing
are not terminal-cell width. The text/layout contract needs its own evidence.

FPAS records are CLR reference types. Cell buffers and hot rendering paths need
a representation that avoids unnecessary per-cell/per-frame object creation.
Buffered arrays are a possible building block; no performance result or final
representation has been established.

## Cached subviews

The desired subview behavior uses explicit typed dependencies, a subtree
builder, and an equivalence comparison. The cached subtree is reused while its
selected dependencies remain equivalent. The builder runs on first use and
when those dependencies change. This is distinct from diffing a fully rebuilt
tree after every View call.

Missing contracts include:

- Selecting every input that affects a subtree, including content, identities,
  actions, controlled state, styles, and model-dependent geometry.
- Pure builder/comparison behavior, with no I/O or required per-update effects
  hidden in cached callbacks.
- Per-application cache ownership, bounded retention, removal of unused entries,
  and cleanup when an application closes.
- Safe handling of nested subviews, reentrancy, resize, and dependency changes.
- Reuse that preserves routing and interaction behavior as well as appearance.

Mutable referenced dependency data cannot stand in for an immutable previous
snapshot. Explicit revisions or appropriate value projections/comparisons are
possible approaches. Cache correctness and lifecycle are unverified.

## Headless testing

The intended headless host uses the same Update, input routing, View, layout,
and painting implementation as an interactive application. Its presentation
target is a cell surface in memory. A second simplified UI implementation would
not establish the real library's behavior.

| Test level | Missing capability and evidence |
| --- | --- |
| Model behavior | Supply a model/message and assert the resulting model and requested effects independently of a terminal. |
| Headless UI | Process keyboard, pointer, resize, and background messages through real routing; assert focus, control state, layout, text, colors, and cells. |
| Timing and effects | Control clocks and external results for bounded deterministic scenarios, including cancellation and failure. |
| Terminal transport | Verify emitted sequences and actual terminal interaction in a controlled PTY/ConPTY session where appropriate. |

The headless host lacks explicit stepping or condition-based waits, message/frame
completion signals, isolated application state, and detached read-only snapshots.
Tests need independent expected behavior and clear failures, rather than
assertions coupled to private implementation calls.

Logical surface assertions establish the library's layout/painting result.
They do not prove that emitted terminal sequences reproduce that result in a
real terminal. Terminal transport tests address that separate risk.

## Live inspection and automation

Inspection and control of a running application is a desired extension. It is
not implemented CLR behavior. A local, explicitly enabled diagnostic connection
could allow a human, test client, or agent to observe and operate the same live
application.

Missing capabilities include:

- Consistent snapshots after completed updates/frames, carrying model/frame
  versions so callers can identify the state they observed.
- A semantic tree with stable control IDs, types/roles, labels, bounds, focus,
  enabled state, and selected/editing values where exposed.
- The last rendered cell surface, with dimensions, styles, and text.
- Summaries of pending work, relevant failures, and a bounded event history.
- Input requests queued through the ordinary application input path.
- Selection of a control by stable identity followed by actual routed input,
  rather than silently bypassing focus, hit testing, or disabled-state checks.
- Condition-based waits and deadlines tied to processed messages/frames, so
  automation does not depend on guessed sleep durations.
- Defined concurrent-input ownership and ordering when a human and an
  automation client are both operating the application.
- A CLR-friendly inspection transport and values with clear lifecycle,
  cancellation, queue limits, and disconnected-client behavior.

Inspection is an explicit library function. It does not require arbitrary CLR
memory access or assume that debugger evaluation can safely read the model.
Snapshots must not expose writable references to the host's working data.
Application-specific model summaries remain separate from generic UI inspection.

Live operation uses real external effects and scheduling. It does not supply
deterministic replay. Controlled headless tests remain necessary. Inspecting a
logical cell surface also remains distinct from observing actual terminal output.

## Terminal and platform adapter

The terminal host needs managed streams and platform-dependent setup/cleanup:
raw input, keyboard/mouse events, resize, alternate screen, cursor state,
terminal capabilities, and restoration on all supported exit paths.

System.Console alone does not establish the complete behavior of the Rust
terminal host. Native mode configuration and PTY/ConPTY hosting may need a small
C# platform adapter or narrowly scoped interop dependency. Relevant Windows
mechanisms are documented in
[virtual terminal sequences](https://learn.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences)
and [pseudoconsole sessions](https://learn.microsoft.com/en-us/windows/console/creating-a-pseudoconsole-session).

The exact Windows/Linux adapter, public managed signatures, and native-resource
ownership are open. A claim that every layer can be written in today's FPAS is
not yet established. A platform adapter would serve the FPAS-owned TUI library;
MVU, widgets, layout, rendering policy, subviews, and headless behavior can still
be FPAS implementation responsibilities.

The interactive terminal, memory presentation, and live diagnostics need to
share the application's real processing interfaces. Their different transports
do not justify different UI semantics.

## Scope and verification

The public CLR interface remains unselected and is governed by the existing
mapping/interop specifications. Any newly required source syntax needs separate
user approval. This inventory does not create projects or packages, choose a
library directory, or modify compiler behavior.

The missing proof includes real FPAS library compilation, C#/F# consumers,
independent application instances, MVU/routing behavior, subview cache lifecycle,
background cancellation/cleanup, Unicode geometry, terminal restoration,
headless assertions, and live-connection/frame consistency on Windows and Linux.

[CLI gaps](cli-gap.md) own command/debugger protocols, and
[editor gaps](editor-gap.md) own IDE integration. The TUI's optional live
inspection is an application/library capability and is not DAP.
Narrow this inventory as its owning implementation and relevant verification
become available, without treating general CLR interoperability tests as TUI
acceptance.
