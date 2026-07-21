# Std.Tui3

This directory plans the FPAS-native terminal UI with a **functional programming model**
and a **Turbo Vision look**. It replaces the retained direction in
[`docs/future/tui2/`](../tui2/README.md).

Nothing here is user-facing yet. Implemented behavior belongs under `docs/pascal/` only
after it exists. Until then the temporary unit is `Std.Tui3` under `lib/Std/Tui3/`.

## Decisions

- `Std.Tui3` is a new API. It does not preserve `Std.Tui` or `Std.Tui2` compatibility.
- The programming model is **Elm / MVU**: `Model`, `TuiMsg`, `Update`, `View`, `TuiCmd`.
- Turbo Vision supplies **chrome and interaction metaphors only** (desktop, window frames,
  dialogs, menus, status line, modal overlays). It does not supply object ownership,
  `Create`/`Add`/`Destroy`, or widget event properties.
- The public API uses the `Tui` prefix. The temporary unit name is `Std.Tui3`.
- `View(Model)` returns a `TuiElement` **data tree**. There are no public live view handles.
- Interactive elements carry **`TuiAction` ids** (positive integers). The runtime turns
  activation into `TuiMsg.Action(Id)`. Applications match ids in `Update`. FPAS has
  generic routines but not generic element types, so the tree does not store app-defined
  message enums.
- After each update the runtime rebuilds the element tree, lays it out, and paints
  (frame rebuild). Keyed reconcile is optional later; it is not required for v1.
- Focus, selection, and scroll offsets live in the application `Model` (or an explicit
  focus record the app stores in the model). They are not hidden registry object state.
- Geometry, cells, palette, surface, and canvas ideas are **salvaged from Tui2** as pure
  values. Live layout handles, view registries, and widget callbacks are not.
- `Std.Console` remains the only terminal I/O boundary. Rust stays thin: events and cells.
- UI work is main-task-only. Effects leave `Update` as `TuiCmd` values.
- When Tui3 is accepted: delete `Std.Tui` and `Std.Tui2`, rename `Std.Tui3` to `Std.Tui`,
  and remove this future plan.

## Endgame

```text
docs/future/tui3/     → plan (this tree)
lib/Std/Tui3/         → temporary implementation
docs/pascal/std/tui3/ → temporary user-facing docs when implemented

accept model
  delete Std.Tui (turbo-vision bridge) and Std.Tui2 (retained)
  rename Std.Tui3 → Std.Tui
  rename docs/tests accordingly
  delete docs/future/tui3 and freeze notes for tui2
```

## Salvage from Tui2

| Take | Leave |
| --- | --- |
| `TuiPoint` / `TuiSize` / `TuiRect`, zero-based, exclusive edges | Generational live handles, `AsView` |
| Cells, styles, palette, surface, canvas | `OnClick` / view lifecycle event properties |
| Size policies, margins, alignment, stretch as **pure layout inputs** | Live `TuiLayout` registry and `Add` ownership |
| Headless surface inspection and injected-input testing idea | `Dialog.OpenModal` as object API |
| Console terminal boundary, main-task posting idea | turbo-vision / `Std.Tui` bridge |

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
| [implementation-phases.md](implementation-phases.md) | Ordered milestones through promote-to-`Std.Tui`. |

## Success criterion

A headless confirm-dialog demo (input + button + modal chrome) is writable with **no**
`Create` / `Add` / `Destroy` / `AsView` / `OnClick` in application code — only `Model`,
`TuiMsg`, `Update`, and `View` — while the surface still looks like a Turbo Vision window
and dialog.
