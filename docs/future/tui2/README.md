# Std.Tui2

This directory records the plan for a new terminal UI library implemented primarily in Functional Pascal source.

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
| [api-surface.md](api-surface.md) | Planned value records, live handles, controls, and rough operations. |
| [layout.md](layout.md) | Size contracts, layout items, allocation, and initial layout types. |
| [event-loop.md](event-loop.md) | `Std.Console` input boundary and event routing order. |
| [events-and-actions.md](events-and-actions.md) | Authoritative event, action, callback, and application-state contract. |
| [actions-and-handlers.md](actions-and-handlers.md) | Temporary forwarding page from the superseded named-handler design. |
| [view-lifecycle.md](view-lifecycle.md) | Application lifecycle and custom-view measure, paint, focus, and close hooks. |
| [source-library.md](source-library.md) | Source manifest, trusted `Std.*` units, exports, and overrides. |
| [geometry.md](geometry.md) | Coordinate spaces, rectangle semantics, clipping, and hit-testing. |
| [handles-and-ownership.md](handles-and-ownership.md) | Generational capabilities, typed conversion, ownership, and destruction. |
| [application-state.md](application-state.md) | Temporary forwarding page from the superseded fixed-state design. |
| [text-and-cells.md](text-and-cells.md) | Grapheme width, cell repair, clipping, and semantic palettes. |
| [runtime-boundary.md](runtime-boundary.md) | Main-task access, posting, errors, and terminal restoration. |
| [testing.md](testing.md) | Deterministic headless and failure-canary requirements. |
| [implementation-phases.md](implementation-phases.md) | Ordered implementation and test milestones. |

## Language prerequisites

The event surface depends on the implemented [capturing closures](../../pascal/language/functions/closures.md),
[record properties](../../pascal/language/types/record-properties.md), and
[events and bound record methods](../events-and-bound-methods.md). Custom layouts, action groups,
richer menu construction, and additional controls remain non-blocking extensions.

## Current implementation status

Phase 0 is complete. Phase 1 geometry is complete: static type-owned construction and focused value-record units are in place. Grapheme-aware `Std.Console.DisplayWidth` is the shared deterministic text-width primitive. The next step in [Phase 1](implementation-phases.md#phase-1--geometry-text-cells-and-canvas) is the Tui2 cell surface and canvas boundary.
