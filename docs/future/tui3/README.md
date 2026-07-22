# Std.Tui3

This directory plans the FPAS-native terminal UI with a **functional programming model**
and a **Turbo Vision look**. It replaces the retained direction in
[`docs/future/tui2/`](../tui2/README.md).

This directory remains the plan for work beyond the completed Phase 0 feasibility slice. Current
implemented behavior is documented under [`docs/pascal/std/tui3/`](../../pascal/std/tui3/README.md).
The temporary implementation remains `Std.Tui3` under `lib/Std/Tui3/` until promotion.

## Decisions

- `Std.Tui3` is a new API. It does not preserve `Std.Tui` or `Std.Tui2` compatibility.
- The programming model is **Elm / MVU**: `Model`, `TuiMsg`, `Update`, `View`, `TuiCmd`.
- Turbo Vision supplies **chrome and interaction metaphors only** (desktop, window frames,
  dialogs, menus, status line, modal overlays). It does not supply object ownership,
  `Create`/`Add`/`Destroy`, or widget event properties.
- The public API uses the `Tui` prefix. The temporary unit name is `Std.Tui3`.
- `View(Model)` returns a `TuiElement` **data tree**. There are no public live view handles.
- Every interactive element has a unique **`TuiControlId`** for focus and routing. It may
  additionally carry a repeatable **`TuiAction`** id for application intent. FPAS has generic
  routines but not generic element types, so the tree does not store app-defined message enums.
- After each update the runtime rebuilds the element tree, lays it out, and paints (frame
  rebuild). Phase 0 must prove that the current VM value representation can do this without
  repeated deep tree or cell-grid clones. Keyed reconcile is not a substitute for passing that
  gate.
- Focus, input text/caret, check state, selection, and scroll offsets live in the application
  `Model` (or focused subrecords). They are not hidden registry object state.
- Geometry, cells, palette, surface, and canvas ideas are **salvaged from Tui2** as pure geometry
  and paint contracts. The mutable working surface is host-owned; tests receive an explicit
  immutable snapshot. Live layout handles, view registries, and widget callbacks are not.
- `Std.Console` remains the only terminal I/O boundary. Rust stays thin: events and cells.
- UI work is main-task-only. Tui3 v1 commands are closed data values for runtime control; v1 does
  not embed arbitrary closures or claim a general asynchronous effect system.
- When Tui3 is accepted: delete `Std.Tui` and `Std.Tui2`, rename `Std.Tui3` to `Std.Tui`,
  and remove this future plan.

## Mandatory feasibility gate

Before value types are ported wholesale, Phase 0 must produce one compiling headless vertical
slice with the exact FPAS signatures used by the runtime. It must cover:

- nested element builders containing a label, controlled input, button, and modal dialog in a
  data-carrying enum with recursive `Children: array of TuiElement` payloads;
- a generic `TModel` host routine with concrete `Update` and `View` function parameters;
- unique control ids, repeatable action ids, focus movement, text editing, and activation;
- initial render before input, deterministic message queue order, and quit order;
- repeated `View` → layout → paint frames at representative tree and terminal sizes;
- evidence that tree traversal and surface painting do not repeatedly deep-clone full values.

If the current `Value` representation fails the performance gate, shared or copy-on-write runtime
storage for the affected aggregate values is a prerequisite. Do not work around the problem with a
public retained widget registry.

## Endgame

```text
docs/future/tui3/     → plan (this tree)
lib/Std/Tui3/         → temporary implementation
docs/pascal/std/tui3/ → temporary user-facing docs when implemented

pass feasibility and promotion gates
  delete Std.Tui (turbo-vision bridge) and Std.Tui2 (retained)
  rename Std.Tui3 → Std.Tui
  rename docs/tests accordingly
  delete docs/future/tui3 and freeze notes for tui2
```

## Salvage from Tui2

| Take | Leave |
| --- | --- |
| `TuiPoint` / `TuiSize` / `TuiRect`, zero-based, exclusive edges | Generational live handles, `AsView` |
| Cells, styles, palette, surface/canvas contracts | `OnClick` / view lifecycle event properties |
| Size policies, margins, alignment, stretch as **pure layout inputs** | Live `TuiLayout` registry and `Add` ownership |
| Headless surface inspection and injected-input testing idea | `Dialog.OpenModal` as object API |
| Console terminal boundary | turbo-vision / `Std.Tui` bridge |

## Documents

| Document | Purpose |
| --- | --- |
| [architecture.md](architecture.md) | Units, names, layers, public vs internal. |
| [mvu.md](mvu.md) | Model, message, update, view, commands, purity. |
| [elements.md](elements.md) | `TuiElement` data tree and TV chrome constructors. |
| [api-surface.md](api-surface.md) | Planned values, messages, elements, and operations. |
| [layout.md](layout.md) | Pure measurement and allocation over element trees. |
| [geometry.md](geometry.md) | Coordinate spaces, clipping, hit-testing. |
| [text-and-cells.md](text-and-cells.md) | Graphemes, surface, palette, canvas. |
| [event-loop.md](event-loop.md) | Console → message → update → view → layout → paint. |
| [runtime-boundary.md](runtime-boundary.md) | Main task, commands, errors, terminal cleanup. |
| [testing.md](testing.md) | Headless contracts and assertion levels. |
| [production-inventory.md](production-inventory.md) | Phase 6.1 classification of retained and migration-owned TUI paths. |
| [promotion-manifest.md](promotion-manifest.md) | Exact Phase 7 deletion, modification, and rename targets. |
| [implementation-phases.md](implementation-phases.md) | Execution rules, current baseline, phase index, and gates. |
| [`phases/`](phases/) | Small file-scoped task cards for Phases 1–7. |

## Success criterion

A headless confirm-dialog demo (controlled input + button + modal chrome) is writable with **no**
`Create` / `Add` / `Destroy` / `AsView` / `OnClick` in application code — only `Model`,
`TuiMsg`, `Update`, and `View` — while the surface still looks like a Turbo Vision window and
dialog. The same slice must pass the Phase 0 clone/performance gate. Promotion additionally requires
an explicit feature-gap audit against the production `Std.Tui` applications that would be removed.
