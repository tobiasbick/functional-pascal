# Std.Tui2

This directory records the plan for a new terminal UI library implemented primarily in Functional Pascal source.

## Decisions

- `Std.Tui2` is a new API. It does not preserve `Std.Tui` compatibility.
- The terminal backend remains a small Rust responsibility: read terminal events and render terminal cells.
- `Std.Tui2` owns the event loop, view registry, focus, modal routing, commands, and widgets in FPAS source.
- The public API uses the `Tui` prefix consistently to avoid short-name conflicts with `Std.Console` and `Std.Graph`.
- Static record functions are a required language prerequisite. Value records use `Type.Create(...)` and distinct `Type.From...(...)` conversions instead of free factory functions or overloads.
- FPAS source files own one concern and, for Tui2 value types, normally one record definition. Geometry therefore uses separate `TuiPoint.fpas`, `TuiSize.fpas`, and `TuiRect.fpas` implementation units.
- The design takes interaction concepts from Turbo Vision, but does not reproduce its class hierarchy or Free Vision implementation.
- Containers use a nested layout system based on size hints, per-axis size policies, stretch factors, spacers, margins, spacing, and alignment.
- User intent is represented by reusable `TuiAction` handles with one synchronous handler; typed control changes use one handler per event.
- The core does not expose a general multicast publish/subscribe bus.
- Views follow a defined lifecycle; public lifecycle hooks are primarily a `TuiCustomView` extension contract.
- TUI coordinates are zero-based and rectangles use exclusive right and bottom edges.
- Live objects use application-scoped generational handles with explicit typed conversions.
- Domain state lives in the application's private FPAS unit state; TUI handles expose integer association tags only.
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
| [actions-and-handlers.md](actions-and-handlers.md) | Programmable actions, typed change handlers, and callback rules. |
| [view-lifecycle.md](view-lifecycle.md) | Application lifecycle and custom-view measure, paint, focus, and close hooks. |
| [source-library.md](source-library.md) | Source manifest, trusted `Std.*` units, exports, and overrides. |
| [geometry.md](geometry.md) | Coordinate spaces, rectangle semantics, clipping, and hit-testing. |
| [handles-and-ownership.md](handles-and-ownership.md) | Generational capabilities, typed conversion, ownership, and destruction. |
| [application-state.md](application-state.md) | Application unit state, tags, and fixed handler signatures. |
| [text-and-cells.md](text-and-cells.md) | Grapheme width, cell repair, clipping, and semantic palettes. |
| [runtime-boundary.md](runtime-boundary.md) | Main-task access, posting, errors, and terminal restoration. |
| [testing.md](testing.md) | Deterministic headless and failure-canary requirements. |
| [implementation-phases.md](implementation-phases.md) | Ordered implementation and test milestones. |

## Non-blocking extensions

The core contracts required through the first controls are decided. Custom layouts, capturing closures, action groups, richer menu construction, and additional controls may be designed later without changing the foundation.

## Current implementation status

Phase 0 is complete. Phase 1 geometry is complete: static type-owned construction and focused value-record units are in place. The next step in [Phase 1](implementation-phases.md#phase-1--geometry-text-cells-and-canvas) is deterministic text width and cell primitives.
