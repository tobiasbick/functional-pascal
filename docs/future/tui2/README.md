# Std.Tui2

This directory records the remaining plan for the FPAS-native terminal UI library.

## Decisions

- `Std.Tui2` is a new API. It does not preserve `Std.Tui` compatibility.
- The terminal backend remains a small Rust responsibility: read terminal events and render terminal cells.
- `Std.Tui2` owns the event loop, view registry, focus, modal routing, commands, and widgets in FPAS source.
- The public API uses the `Tui` prefix consistently to avoid short-name conflicts with `Std.Console` and `Std.Graph`.
- Static record functions are a required language prerequisite. Value records use `Type.Create(...)` and distinct `Type.From...(...)` conversions instead of free factory functions or overloads.
- Live-handle state uses computed record properties; events are specialized properties backed by the live registry rather than fields copied inside handle values.
- FPAS source files own one concern and, for Tui2 value types, normally one record definition. Stable groups have matching directories and namespace segments; geometry uses `Geometry/Point.fpas`, `Geometry/Size.fpas`, and `Geometry/Rect.fpas`.
- The design takes interaction concepts from Turbo Vision, but does not reproduce its class hierarchy or Free Vision implementation.
- Containers use a nested layout system based on size hints, per-axis size policies, stretch factors, spacers, margins, spacing, and alignment.
- Controls expose Pascal-style, typed, single-handler events. Reusable `TuiAction` handles represent shared user intent.
- The core does not expose a general multicast publish/subscribe bus.
- Views follow a defined lifecycle; public lifecycle hooks are primarily a `TuiCustomView` extension contract.
- TUI coordinates are zero-based and rectangles use exclusive right and bottom edges.
- Live objects use application-scoped generational handles with explicit typed conversions.
- Capturing closures normally own local application state; private unit state and integer association tags remain optional.
- Text layout uses grapheme clusters, deterministic display width, continuation cells, and semantic palette roles.
- UI mutation is main-task-only; worker tasks use a typed FIFO post queue to schedule main-task callbacks.
- Terminal acquisition is transactional and VM teardown provides a mode-restoration safety net.
- Current user-facing documentation remains under `docs/pascal/`; these files describe only future work.

## Documents

| Document | Purpose |
| --- | --- |
| [architecture.md](architecture.md) | Units, names, geometry, and the view model. |
| [api-surface.md](api-surface.md) | Remaining value records, live handles, controls, and rough operations. |
| [layout.md](layout.md) | Size contracts, layout items, allocation, and initial layout types. |
| [event-loop.md](event-loop.md) | `Std.Console` input boundary and event routing order. |
| [events-and-actions.md](events-and-actions.md) | Remaining event, action, callback, and posting contract. |
| [view-lifecycle.md](view-lifecycle.md) | Application lifecycle and custom-view measure, paint, focus, and close hooks. |
| [geometry.md](geometry.md) | Future coordinate spaces, clipping, and hit-testing. |
| [handles-and-ownership.md](handles-and-ownership.md) | Generational capabilities, typed conversion, ownership, and destruction. |
| [text-and-cells.md](text-and-cells.md) | Remaining grapheme surface, cell repair, clipping, and canvas rules. |
| [runtime-boundary.md](runtime-boundary.md) | Main-task access, posting, errors, and terminal restoration. |
| [testing.md](testing.md) | Remaining deterministic headless and failure-canary requirements. |
| [implementation-phases.md](implementation-phases.md) | Ordered implementation and test milestones. |

## Current implementation status

Implemented behavior is documented in the current
[`Std.Tui2` reference](../../pascal/std/tui2/README.md). The next ordered work is the Phase 1 cell
surface and canvas boundary in [implementation-phases.md](implementation-phases.md).
