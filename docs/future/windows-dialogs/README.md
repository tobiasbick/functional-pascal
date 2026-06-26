# TUI frames: windows and dialogs (Turbo Vision direction)

Implementation plan for host-managed **frame widgets** — Turbo Vision–style windows and dialogs with integrated chrome (title bar, close/zoom buttons, scroll bars) and a separate content **view** area for child views.

**Status:** **partial** — frame-root geometry, window-manager interaction, and painted
`FrameWidget` chrome, inner viewport clipping, and public `ShowFramedDialog` are implemented;
frame-integrated scrolling and close handling are not. **Live
tracker:** [TUI-CODE-REVIEW.md](TUI-CODE-REVIEW.md#remaining-work-2026-06-22) (phases 0–5 done;
phase 6+ open). **Current spec:** [`docs/pascal/std/tui/app/frames.md`](../../pascal/std/tui/app/frames.md).

**Prerequisites:** Phase 7 TUI host (view tree, modal stack, host widgets, damage tracking). See [`tui-application-framework.md`](../tui-application-framework.md).

**Canonical spec target (when implemented):** [`docs/pascal/std/tui/app/README.md`](../../pascal/std/tui/app/README.md) and [`vm-bridge.md`](../../pascal/std/tui/app/vm-bridge.md).

---

## Goals

1. Replace hand-painted dialog boxes (see [`show_dialog.fpas`](../../../examples/pascal/tui/show_dialog.fpas)) with a declarative Rust-hosted frame widget.
2. Align visually and structurally with **Turbo Vision**: title in the frame, close on the left, zoom on the right, blue windows, gray dialogs.
3. Support **scrolling content** in both windows and dialogs; scroll bars are **frame chrome**, not views.
4. Reuse the existing modal lifecycle (`ShowModal`, `CloseModal`) and its focus routing. Framed-dialog creation adds an atomic owned-root operation because the existing `ShowDialog` always creates a different root view.

---

## Principles

- **Heavy lifting in Rust** — border, title bar, buttons, scroll bars, clipping, scroll offset, wheel/scrollbar hit-testing.
- **One frame implementation, two presets** — `FrameKind.Window` (blue) and `FrameKind.Dialog` (gray); not two separate paint paths.
- **Frame vs. view** — frame owns all chrome; the view is only the inner content viewport where children live.
- **Scroll bars live in the frame** — never separate `ViewId`s, never child views, never Pascal paint handlers for `▲█▼`.
- **Modal behavior stays on the modal API** — `ShowModal` / `CloseModal` provide focus scope and routing; `ShowFramedDialog` creates an owned modal root atomically. Frame flags control appearance and interaction only.

---

## Frame vs. view

```text
┌─ Frame (chrome) ─────────────────────────────┐
│ ■ Title                           ▲▼ (zoom) │  title bar row
├───────────────────────────────────────┬──▲──┤
│                                       │  █  │  vertical scroll (frame column)
│           View (content)              │  ▼  │
│           child views live here       │     │
├───────────────────────────────────────┴──┬──┤
│ ◄████████ horizontal track ████████████ ►│  │  horizontal scroll (frame row)
└──────────────────────────────────────────┘
```

| Layer | Owns | Painted / input routed by |
| ----- | ---- | ------------------------- |
| **Frame** | Border, title bar, close (`■`), zoom (`▲`/`▼`), scroll bars | `FrameWidget` in Rust |
| **View** | Content viewport only | Existing view tree + `HostRegisterOnViewPaint`; scroll **offset** applied by the frame host |

### Geometry (`frame/geometry.rs`)

| Rect | Meaning |
| ---- | ------- |
| `outer_rect` | Total widget bounds passed to `HostCreateFrameView` |
| `view_rect` | Inner content viewport; child coordinates are relative to this rect |
| `frame_insets` | Border + title bar + scroll bar column/row when visible |

When a scroll bar becomes visible, the **frame** reserves a chrome column or row inside `outer_rect`. The **view** rect shrinks accordingly, but scroll bars remain frame-owned chrome — not part of view content.

Scroll-bar visibility is solved to a fixed point:

1. Compute the viewport without scroll bars.
2. Add each bar whose content axis exceeds the current viewport axis.
3. Recompute the viewport because either bar can make the other necessary.
4. Repeat until visibility is unchanged (at most two recomputations for two axes).

Creation and resize validate minimum geometry before mutating the view tree. A non-scrollable frame must be at least `4 x 3`; a scrollable frame must be at least `6 x 6`, leaving room for both bars, two arrow cells, and a thumb cell. Invalid geometry returns a runtime error with the required minimum and leaves the existing frame unchanged.

---

## Title bar layout (Turbo Vision)

Title text sits **in** the top frame row, not above the border.

```text
┌─■─ Title text ─────────────────────▲▼─┐
```

| Slot | Position | Default |
| ---- | -------- | ------- |
| Close | Left | `■` when `Closable` is available and enabled; emits `FrameAction.Close` when clicked |
| Title | After close | Single line, truncated with `…` when too long |
| Zoom | Right | `▲` / `▼` when `Zoomable` is available and enabled; hidden for dialog preset |

---

## Frame kinds and color presets

Two built-in presets selected by `FrameKind`. Colors use CRT indices (`0..=15`, same as `Std.Console`). Custom overrides via `FrameStyle`.

### `FrameKind.Window` — blue

| Region | CRT | Notes |
| ------ | --- | ----- |
| Title bar background | `LightBlue` (9) | Active window |
| Title bar foreground | `White` (15) | Title, border top |
| Client background | `LightGray` (7) | Content area |
| Client foreground | `Black` (0) | Default content text |
| Border | Title bar colors | Frame lines match title bar |

Aligns with [`apps/ide/src/theme.fpas`](../../../apps/ide/src/theme.fpas) desktop blue + gray work areas.

### `FrameKind.Dialog` — gray

| Region | CRT | Notes |
| ------ | --- | ----- |
| Title bar background | `LightGray` (7) | Flat modal look |
| Title bar foreground | `Black` (0) | |
| Client background | `LightGray` (7) | Uniform gray |
| Client foreground | `Black` (0) | |
| Border | Title bar colors | |
| Zoom buttons | Hidden | `Zoomable` defaults to `false` |

### Inactive window (later)

Title bar `Blue` (4), text `LightGray` — deferred; document preset only, not phase 1.

---

## Capabilities

Behavior flags separate from kind and from modal lifecycle.

```pascal
type
  FrameKind = enum
    Window;
    Dialog;
  end;

  FrameCapabilities = record
    Closable: boolean;
    Zoomable: boolean;
    Movable: boolean;
    Resizable: boolean;
    Scrollable: boolean;
  end;
```

| Capability | Window default | Dialog default | Available |
| ---------- | -------------- | -------------- | --------- |
| Closable | false until Phase 3, then true | false until Phase 3, then true | Phase 3 |
| Zoomable | false until Phase 4, then true | false | Phase 4 |
| Movable | false until Phase 4, then true | false | Phase 4 |
| Resizable | false until Phase 4, then true | false | Phase 4 |
| Scrollable | false until Phase 2, then true | false until Phase 2, then true | Phase 2 |

Modal scope, owned root cleanup, and input blocking outside the modal remain modal-stack responsibilities. `Application.ShowModal` handles an existing non-owned frame, `Application.ShowDialog` retains its current create-and-own behavior for unframed dialogs, and `Application.CloseModal` closes either form.

`DefaultFrameCapabilities` only enables behavior implemented in its release phase. Passing an unavailable capability as `true` is rejected instead of displaying a control that cannot perform its action.

---

## Scrolling

Both windows and dialogs can scroll their **view** content. Scroll state lives in `FrameWidget`; scroll bars are **frame chrome**.

### Scroll state (Rust)

```rust
struct FrameScroll {
    offset_x: i64,
    offset_y: i64,
    content_width: i64,
    content_height: i64,
}
```

- `content_*` — logical content size (set from Pascal or derived later from child bounds).
- `offset_*` — host-managed, clamped to `[0 .. content - view_rect.size]`.
- Vertical scroll bar — right **frame** column (one cell wide) when `content_height > view_rect.height`.
- Horizontal scroll bar — bottom **frame** row (one cell high) when `content_width > view_rect.width`.
- Corner cell — frame chrome when both bars are visible.

### Child coordinates

Children parent to the frame root view. Local `(x, y)` are **view coordinates** (origin top-left of `view_rect`). The view registry resolves a transform for every view; a frame contributes its viewport origin, scroll offset, and viewport clip:

```text
absolute = view_rect.origin + child.local - (offset_x, offset_y)
```

The transform composes through descendants and nested frames. The same resolved screen rectangle and effective clip must be used by:

- widget and Pascal-handler painting;
- mouse hit-testing and modal scope checks;
- `QueryViewRect`, which returns the translated screen rectangle before clipping;
- damage calculation, including focus redraw hints;
- reparenting that preserves a view's screen position.

This is a view-registry contract, not a paint-only adjustment. Descendants fully outside the effective clip do not paint or receive pointer input. No scroll viewport wrapper view is introduced.

### Input

| Source | Behavior |
| ------ | -------- |
| Mouse wheel inside `view_rect` | Scroll vertically; Shift+wheel → horizontal (when supported) |
| Click `▲` / `▼` / `◄` / `►` | Scroll by one line/column |
| Click in track above/below thumb | Scroll by one page (viewport height/width) |
| `Up` / `Down` / `PgUp` / `PgDn` / `Home` / `End` | Scroll only when focus is in the frame subtree and the focused descendant did not consume the key |

Pointer targeting checks title-bar buttons and scroll bars before view content. Direct chrome hits never reach child views. Events over `view_rect` target the topmost child first and bubble through ancestors only while unconsumed; this lets a frame consume an unhandled wheel event.

Keyboard handling starts at the focused descendant. Focused widget handling and view-local command bindings run before ancestor widget fallbacks; only then may the containing frame consume scrolling keys. Modal/global command bindings and `OnKeyPressed` remain later fallbacks. `Home`, `End`, and arrow keys therefore remain available to focused controls that implement them.

Mouse wheel uses existing `Std.Console.Event` scroll actions (`ScrollUp`, `ScrollDown`, …).

### Paint order

The redraw dispatcher changes from separate global widget and Pascal-handler passes to one depth-first view traversal with per-view phases:

1. Paint the widget underlay; for a frame, fill `view_rect` with `ClientBg`.
2. Invoke that view's Pascal paint handler with its resolved transform and effective clip.
3. Paint child subtrees in sibling z-order.
4. Paint the widget overlay; for a frame, paint border, title bar, buttons, and scroll bars.

Widgets that do not need an overlay keep their existing single paint operation in the underlay phase. Menu popups retain their explicit top-level overlay pass. This ordering guarantees that a frame's Pascal handler and descendants cannot overwrite frame chrome.

Title bar and outer border do not scroll.

---

## Window vs. dialog lifecycle

| | Window | Dialog |
| --- | ------ | ------ |
| Creation | `HostCreateFrameView(..., FrameKind.Window, ...)` | `ShowFramedDialog(...)` or frame + `ShowModal` |
| Modal | No | Yes (`ShowFramedDialog` / `ShowModal`) |
| Colors | Blue preset | Gray preset |
| Zoom | Default on from Phase 4 | Default off |
| Owned cleanup on close | No; caller unregisters it | Yes after the action handler calls `CloseModal` when created via `ShowFramedDialog` |
| Scrolling | Yes from Phase 2 (frame chrome) | Yes from Phase 2 (frame chrome) |

---

## Pascal API (planned)

Types:

```pascal
type
  FrameKind = enum Window; Dialog; end;

  FrameStyle = record
    TitleBg: integer;
    TitleFg: integer;
    ClientBg: integer;
    ClientFg: integer;
    BorderFg: integer;
    ScrollFg: integer;
    ScrollBg: integer;
  end;

  FrameCapabilities = record ... end;

  FrameAction = enum
    Close;
    Zoom;
    ZoomBack;
  end;

  FrameActionHandler = procedure(
    App: Application;
    FrameView: ViewId;
    Action: FrameAction
  );

  FrameScrollState = record
    OffsetX: integer;
    OffsetY: integer;
    ContentWidth: integer;
    ContentHeight: integer;
  end;
```

Functions (names tentative until spec lands in `docs/pascal/`):

```pascal
function DefaultFrameStyle(Kind: FrameKind): FrameStyle;
function DefaultFrameCapabilities(Kind: FrameKind): FrameCapabilities;

function HostCreateFrameView(
  App: Application;
  X, Y, Width, Height: integer;
  Title: string;
  Kind: FrameKind;
  Capabilities: FrameCapabilities;
  Style: FrameStyle
): ViewId;

function ShowFramedDialog(
  App: Application;
  ModalId: integer;
  X, Y, Width, Height: integer;
  Title: string;
  Style: FrameStyle;
  Capabilities: FrameCapabilities
): ViewId;

procedure HostRegisterOnFrameAction(
  App: Application;
  FrameView: ViewId;
  OnFrameAction: FrameActionHandler
);

procedure HostSetFrameContentSize(
  App: Application; FrameView: ViewId;
  ContentWidth, ContentHeight: integer
);

procedure HostScrollFrame(
  App: Application; FrameView: ViewId;
  DeltaX, DeltaY: integer
);

procedure HostSetFrameScrollOffset(
  App: Application; FrameView: ViewId;
  OffsetX, OffsetY: integer
);

function QueryFrameScrollState(
  App: Application; FrameView: ViewId
): FrameScrollState;
```

Frame chrome emits `OnFrameAction(App, FrameView, Action)`. Carrying `FrameView` identifies the source when several frames exist, and a dedicated action type avoids collisions with application-defined `OnCommand` ids. The handler decides policy: a framed dialog normally handles `Close` with `Application.CloseModal`, while a non-modal window may hide, unregister, or retain its model before unregistering the view. Zoom actions update host-managed geometry once Phase 4 is available. A frame with no registered handler consumes the chrome click without invoking Pascal. Unregistering a frame removes its handler registration.

Escape-to-close remains app-defined via `HostBindCommandToActiveModal` for dialogs.

---

## Rust module layout

```text
crates/fpas-std/src/tui/widget/frame/
 ├── mod.rs         — FrameWidget, ViewWidget::Frame, input orchestration
 ├── kind.rs        — FrameKind presets (Window blue, Dialog gray)
 ├── style.rs       — FrameStyle, scroll bar colors
 ├── geometry.rs    — outer / view_rect / scroll bar slots
 ├── chrome.rs      — border, title bar, ■, ▲▼
 ├── action.rs      — FrameAction and chrome hit results (Pascal callbacks remain VM-owned)
 └── scroll.rs      — offset state, bar paint, wheel/click, clamp
```

Extend `ViewWidget` enum in [`widget/mod.rs`](../../../crates/fpas-std/src/tui/widget/mod.rs) with `Frame(FrameWidget)`.

VM bridge: new intrinsics in `fpas-bytecode`, sema in `loaded/tui/`, compiler lowering in `std_calls/tui/`, execution in `fpas-vm/.../tui/`. The view registry gains composable content transforms and clips; the redraw dispatcher gains widget underlay/overlay phases. Keep those changes in focused files under the existing `view/`, `widget/`, and `host/` subdirectories rather than growing the current bridge entrypoints.

---

## Implementation phases

> **Note (2026-06-21):** The checklist below is the original design breakdown. Progress is tracked in
> [TUI-CODE-REVIEW.md](TUI-CODE-REVIEW.md). Phases 0–5 there are complete; frame **painting** and
> items in [Remaining work](TUI-CODE-REVIEW.md#remaining-work-2026-06-22) are still open.

### Phase 1 — Geometry and chrome

- [ ] Add composable view-registry transforms/clips and use them consistently for resolved rectangles, `QueryViewRect`, hit-testing, damage, and reparenting.
- [x] Replace the two global paint passes with depth-first underlay → handler → children → overlay traversal.
- [x] `geometry.rs` — compute `view_rect` from `outer_rect`, title bar height, border, scroll bar slots.
- [x] Define and test fixed-point scroll-bar visibility and minimum frame dimensions.
- [x] `chrome.rs` — double-line border, title layout, and reserved capability slots; unavailable controls remain hidden.
- [x] `kind.rs` + `style.rs` — Window (blue) and Dialog (gray) defaults.
- [x] `FrameWidget::paint` — view background + chrome; no scrolling yet.
- [x] `HostCreateFrameView` intrinsic + sema + compiler.
- [x] Example: `examples/pascal/tui/framed_window.fpas` (static content, no scroll).

### Phase 2 — Scrolling

- [x] `scroll.rs` — state, vertical/horizontal bar paint (`▲█▼`, `◄█►`), auto-show when content exceeds view.
- [x] Apply scroll offset through the Phase 1 registry transform for paint, query, hit-test, damage, and nested descendants.
- [x] Mouse wheel bubbles child-first; direct scroll bar clicks target frame chrome.
- [x] Keyboard scrolling runs after focused descendant handling and before modal/global fallback handling.
- [x] `HostSetFrameContentSize`, `HostScrollFrame`, `HostSetFrameScrollOffset`, `QueryFrameScrollState`.
- [ ] Enable `Scrollable` in both default capability presets only after the complete input and rendering path is available.
- [ ] Examples: long text in window; list taller than dialog viewport.
- [ ] FPAS tests under `tests/tui/` (headless scroll offset + bar visibility queries if exposed).

### Phase 3 — Dialog integration and actions

- [ ] Add an internal modal-stack operation that marks an existing root as owned.
- [x] `ShowFramedDialog` atomically creates the frame and pushes that same root as owned; invalid geometry leaves both registries unchanged.
- [ ] `FrameAction` + `HostRegisterOnFrameAction`; chrome actions include the source `ViewId` and do not use `OnCommand` ids.
- [ ] Remove per-frame action handlers when a frame is unregistered or its owned modal closes.
- [ ] Close-button hit-test emits `FrameAction.Close`; the example handler calls `CloseModal`.
- [ ] Enable `Closable` in both default capability presets.
- [x] Example: `examples/pascal/tui/framed_dialog.fpas` (modal gray dialog, Escape closes).
- [ ] Update IDE shell when ready to replace ad-hoc panels.

### Phase 4 — Interaction polish (deferred)

- [ ] Movable frames (drag title bar), then enable the Window default.
- [ ] Resizable frames (drag border / zoom state), then enable Window `Resizable` and `Zoomable` defaults.
- [ ] Inactive window color preset.
- [ ] Thumb drag on scroll track.
- [ ] Auto `content_size` from child bounds.

---

## Testing and verification

When implemented, follow [`.agents/skills/fpas-change-checklist/SKILL.md`](../../../.agents/skills/fpas-change-checklist/SKILL.md):

1. **Docs** — move API descriptions from this plan into `docs/pascal/std/tui/app/` (`README.md`, `vm-bridge.md`, new `frames.md` if needed).
2. **Rust tests** — unit tests for geometry, fixed-point bar visibility, minimum sizes, scroll clamp, transformed descendant rectangles/clips, damage, paint phase order, child-first input bubbling, action source ids, and owned-modal rollback/cleanup.
3. **FPAS tests** — `tests/tui/frames/tui_frame_*_test.fpas` using `OpenForTest` / `TestPump` / screen queries, including two-frame action-source dispatch and a child control that consumes a scrolling key.
4. **Examples** — `framed_window.fpas`, `framed_dialog.fpas`; run `scripts/format-fpas-sources.sh` on touched `.fpas`.
5. **Verify** — `cargo fmt`, `cargo build`, `cargo test --workspace`, `fpas test tests/tui/`.

---

## Out of scope (this plan)

- Separate scroll bar **views** or a generic `TScroller` view type exposed to Pascal (frame-owned
  scroll **chrome** is in scope for Phase 6).
- Full Turbo Vision desktop, drag-drop between windows, MDI window list.
- Automatic layout manager inside frames (anchor/grow on the view tree is tracked separately).
- Memo/text editor (Phase 7 in [TUI-CODE-REVIEW.md](TUI-CODE-REVIEW.md)).

---

## Related documentation

| Document | Relevance |
| -------- | --------- |
| [`tui-application-framework.md`](../tui-application-framework.md) | Host architecture history; Phase 7 complete |
| [`docs/pascal/std/tui/app/README.md`](../../pascal/std/tui/app/README.md) | Current dispatch-mode spec |
| [`docs/pascal/std/tui/app/vm-bridge.md`](../../pascal/std/tui/app/vm-bridge.md) | Intrinsics and modal/view APIs |
| [`apps/ide/src/theme.fpas`](../../../apps/ide/src/theme.fpas) | Turbo Pascal / IDE color reference |
| [`examples/pascal/tui/show_dialog.fpas`](../../../examples/pascal/tui/show_dialog.fpas) | Current manual dialog to replace |
