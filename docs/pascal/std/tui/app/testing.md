# Native testing

## Native TUI testing API

**Status:** implemented. Run under `fpas test` with programs in [`tests/tui/`](../../../../tests/tui/) (`tui_*_test.fpas`, grouped by theme in subdirectories below).

Goal: test hosted `Std.Tui` entirely from FPAS under `fpas test` — headless session, stepwise event pump, input injection, and read-only introspection of screen, views, and widget state. No real terminal, Rust integration test, or TUI sidecar file is required.

### Regression layout (`tests/tui/`)

Programs are grouped by concern. [`tests/suite.fpasprj`](../../../../tests/suite.fpasprj) discovers all of them via `tui/**/*_test.fpas`.

| Subdirectory | Focus | Representative tests |
| ------------ | ----- | -------------------- |
| [`host/`](../../../../tests/tui/host/) | Headless lifecycle, pump, input injection, screen queries | [`tui_pump_test.fpas`](../../../../tests/tui/host/tui_pump_test.fpas), [`tui_escape_test.fpas`](../../../../tests/tui/host/tui_escape_test.fpas), [`tui_screen_query_test.fpas`](../../../../tests/tui/host/tui_screen_query_test.fpas) |
| [`scene/`](../../../../tests/tui/scene/) | View tree, layout, clip, scene graph, stale-query runtime errors | [`tui_view_query_test.fpas`](../../../../tests/tui/scene/tui_view_query_test.fpas), [`tui_view_clip_test.fpas`](../../../../tests/tui/scene/tui_view_clip_test.fpas), [`tui_scene_graph_query_test.fpas`](../../../../tests/tui/scene/tui_scene_graph_query_test.fpas) |
| [`controls/`](../../../../tests/tui/controls/) | Host widgets, scroll bars/views, Tab focus, cell width | [`tui_controls_test.fpas`](../../../../tests/tui/controls/tui_controls_test.fpas), [`tui_tab_focus_test.fpas`](../../../../tests/tui/controls/tui_tab_focus_test.fpas), [`tui_cell_width_test.fpas`](../../../../tests/tui/controls/tui_cell_width_test.fpas) |
| [`menu/`](../../../../tests/tui/menu/) | Menu bar hover, keyboard focus, pull-down overlay compositor | [`tui_menu_hover_test.fpas`](../../../../tests/tui/menu/tui_menu_hover_test.fpas), [`tui_menu_overlay_frame_test.fpas`](../../../../tests/tui/menu/tui_menu_overlay_frame_test.fpas) |
| [`modals/`](../../../../tests/tui/modals/) | `ShowDialog`, `ShowFramedDialog`, modal commands and cleanup | [`tui_show_dialog_test.fpas`](../../../../tests/tui/modals/tui_show_dialog_test.fpas), [`tui_framed_dialog_controls_test.fpas`](../../../../tests/tui/modals/tui_framed_dialog_controls_test.fpas) |
| [`frames/`](../../../../tests/tui/frames/) | Frame chrome, windows, scroll, occlusion repair, reserved commands | [`tui_frame_occlusion_test.fpas`](../../../../tests/tui/frames/tui_frame_occlusion_test.fpas), [`tui_frame_scroll_clip_test.fpas`](../../../../tests/tui/frames/tui_frame_scroll_clip_test.fpas), [`tui_nested_frame_clip_test.fpas`](../../../../tests/tui/frames/tui_nested_frame_clip_test.fpas) |

Contributor placement rule: put new FPAS TUI regressions in the narrowest matching subdirectory.
Use `host/` for lifecycle, pump, injected input, and screen-query behavior; `scene/` for retained
view-tree, layout, clip, focus, and scene-graph behavior; `controls/` for standalone widgets and
cell-width behavior; `menu/` for menu bars and pull-down overlays; `modals/` for dialog/modal scope
and cleanup; and `frames/` for frame chrome, windows, scrolling, occlusion, and frame commands.

Run one theme: `fpas test tests/tui/frames/`. Run a single file: `fpas test tests/tui/host/tui_escape_test.fpas`. Filter by path substring: `fpas test --filter tui_escape`.

For real-terminal behavior (alternate screen, cursor restore, resize flicker), use the manual checklist in [Terminal checklist](../terminal-checklist.md).

### Naming convention (decided)

`Application.*` routines use **one prefix per role**. Do not mix roles under the same prefix.

| Prefix | Role | Examples |
| ------ | ---- | -------- |
| **`Test*`** | Headless test lifecycle, event pump, and input injection | `OpenForTest`, `TestPump`, `TestSendKey`, `TestMoveMouse`, `CloseForTest` |
| **`Query*`** | Read-only host introspection (no mutation) | `QueryScreenCell`, `QueryViewRect`, `QueryFocusedViewId`, `QueryModalDepth`, `QueryMenuBarState` |
| **`Host*`** | Host mutators: register handlers, create/set views and widgets, bind commands, modal stack writes | `HostRegisterView`, `HostCreateMenuBarView`, `HostSetViewRect`, `HostBindCommand` |
| *(none)* | Application-level entry points unchanged | `Open`, `Configure`, `Run`, `ShowModal`, `CloseModal`, `Close` |

Rules:

- **`Query*`** never changes host or screen state. Screen reads reflect the CRT back buffer after the last completed pump/redraw.
- **`Test*`** injectors enqueue input for the next `TestPump` / `TestPumpUntilIdle`; they do not run the host loop themselves.
- **`Host*`** keeps its current meaning for production app setup. New read APIs use **`Query*`**, not `HostQuery*`.
- Intrinsic Rust names mirror Pascal: `TuiTestPump`, `TuiQueryScreenCell`, `TuiHostRegisterView`.

### ViewId type (implemented)

Introduce **`Std.Tui.ViewId`** as a real opaque FPAS type. Do **not** use bare `integer` for host view handles in new or migrated APIs.

```pascal
type ViewId = record end;  { opaque host-owned handle — same pattern as Application }
```

Sema registers `Std.Tui.ViewId` as an empty record; only host routines may produce values. Authors cannot write record literals for `ViewId`.

**Rationale**

- **Type safety:** view handles cannot be confused with coordinates, command ids, or modal ids.
- **Test readability:** `var Bar: ViewId := Application.HostCreateMenuBarView(...)` documents intent at call sites.
- **Remove magic sentinels:** replace integer `-1` with `Option of ViewId` where “no view” is needed.
- **Matches the Rust host:** the VM already stores `ViewId(u32)` in [`ViewRegistry`](../../../../../crates/fpas-std/src/tui/view/mod.rs); the FPAS type is the language-facing wrapper around that token.

**Rules**

| Rule | Detail |
| ---- | ------ |
| Construction | Only host routines return `ViewId` (`HostRegisterView`, `HostCreateMenuBarView`, `ShowDialog`, …). |
| User literals | Sema rejects `42 as ViewId`, integer variables passed where `ViewId` is expected, and arithmetic on `ViewId`. |
| Equality | `ViewId = ViewId` is allowed when comparing handles returned by the same session. |
| Missing view | Use `Option of ViewId` / `None`, not `-1`. |
| Detach to roots | `Application.HostSetViewParent(App, Child, None)` replaces `ParentViewId := -1`. |
| Focus query | `Application.QueryFocusedViewId(App): Option of ViewId` (replaces the former `-1` sentinel). |
| View-local paint | The view-local handler keeps `(App, ViewId, Rect)` — the middle argument is typed `ViewId`. |

**Status:** `ViewId` migration and `Query*` renames for focus/modal depth are **implemented** (sema, VM, tests, and examples).

### Headless lifecycle and pump (Phase 1 — implemented)

| Pascal call | Returns | Role |
| ----------- | ------- | ---- |
| `Application.OpenForTest(Width, Height)` | `Application` | Virtual screen, no terminal writer |
| `Application.TestPump(App)` | `()` | Ingest queued input, flush coalesced resize, process one event, redraw |
| `Application.TestPumpUntilIdle(App)` | `()` | Repeat pump until idle |
| `Application.CloseForTest(App)` | `()` | Deterministic teardown |

### Input injection (`Test*` — Phase 2, implemented)

| Pascal call | Notes |
| ----------- | ----- |
| `Application.TestSendKey(App, Key)` | Full `KeyEvent`; enqueues for next pump |
| `Application.TestSendMouse(App, Event)` | Full `Std.Console.Event` (typically mouse) |
| `Application.TestMoveMouse(App, X, Y)` | Convenience: `Move`, one-based coords |
| `Application.TestClickMouse(App, X, Y)` | `Down` then `Up` |
| `Application.TestResize(App, Width, Height)` | Terminal resize |
| `Application.TestPaste(App, Text)` | Bracketed paste |
| `Application.TestFocus(App, Gained: boolean)` | Focus gained/lost |

Use `Std.Console.EventKind` (not bare `EventKind`) when both `Std.Console` and `Std.Tui` are in scope.

**Pump rules**

- `Test*` injectors only enqueue events; call `TestPump` or `TestPumpUntilIdle` to run the host loop.
- `TestPump` processes **one** queued event (plus coalesced resize flush) and settles redraws before returning.
- `TestClickMouse` enqueues **two** events (`Down` then `Up`). After a click, use `TestPumpUntilIdle` (or two `TestPump` calls) before the next assertion or injection.
- Call `RequestRedraw` after `Configure` when the first paint must run before any input.
- Screen and widget queries reflect state **after** the most recent completed pump.

### Screen and view introspection (`Query*` — implemented)

Screen reads:

| Pascal call | Returns |
| ----------- | ------- |
| `Application.QueryScreenSize(App)` | `Size` |
| `Application.QueryScreenLine(App, Y)` | `string` |
| `Application.QueryScreenCell(App, X, Y)` | `ScreenCell` (`ch`, `fg`, `bg`) |

View and widget reads:

| Pascal call | Returns |
| ----------- | ------- |
| `Application.QueryModalDepth(App)` | `integer` |
| `Application.QueryFocusedViewId(App)` | `Option of ViewId` |
| `Application.QueryRootViews(App)` | `array of ViewId` |
| `Application.QueryViewRect(App, ViewId)` | `Rect` |
| `Application.QueryViewParent(App, ViewId)` | `Option of ViewId` |
| `Application.QueryViewChildren(App, ViewId)` | `array of ViewId` |
| `Application.QueryViewState(App, ViewId)` | `ViewState` |
| `Application.QueryViewOptions(App, ViewId)` | `ViewOptions` |
| `Application.QueryResolvedView(App, ViewId)` | `ResolvedView` |
| `Application.QueryViewKind(App, ViewId)` | `ViewKind` |
| `Application.QuerySceneGraph(App)` | `array of ViewSnapshot` |
| `Application.QueryMenuBarState(App, ViewId)` | `MenuBarState` |
| `Application.QueryInputLineState(App, ViewId)` | `InputLineState` |
| `Application.QueryCheckBoxState(App, ViewId)` | `CheckBoxState` |
| `Application.QueryRadioGroupState(App, ViewId)` | `RadioGroupState` |
| `Application.QueryListBoxState(App, ViewId)` | `ListBoxState` |

Retained state controls used by headless tests:

| Pascal call | Effect |
| ----------- | ------ |
| `Application.HostSetViewVisible(App, ViewId, Visible)` | Controls resolved visibility, clipping, painting, and focus eligibility |
| `Application.HostSetViewEnabled(App, ViewId, Enabled)` | Controls input and focus eligibility for one view |

See [ScreenCell type](#screencell-type-decided) and [MenuBarState type](#menubarstate-type) below.

### Native testing bytecode discriminants

Native test lifecycle and basic queries use **356..=374** in
[`TuiIntrinsic`](../../../../../crates/fpas-bytecode/src/intrinsic/tui.rs). Scene-graph state APIs
use **382..=403**. **348..=355**, **375..=378** belong to `Std.Test` (see
[`test.md`](../../testing/test.md)).

| Discriminant | Pascal surface | Notes |
| ------------ | -------------- | ----- |
| **356** | `OpenForTest` | Virtual CRT, no terminal writer |
| **357** | `TestPump` | One event + redraw settle |
| **358** | `TestPumpUntilIdle` | Drain queue |
| **359** | `CloseForTest` | Teardown |
| **360** | `TestSendKey` | Enqueue `KeyEvent` |
| **361** | `TestSendMouse` | Enqueue full `Std.Console.Event` |
| **362** | `TestMoveMouse` | Enqueue `Move` |
| **363** | `TestClickMouse` | Enqueue `Down` + `Up` |
| **364** | `TestResize` | Terminal resize |
| **365** | `TestPaste` | Bracketed paste |
| **366** | `TestFocus` | Focus gained/lost |
| **367** | `QueryScreenSize` | |
| **368** | `QueryScreenLine` | `Y` one-based |
| **369** | `QueryScreenCell` | `X`/`Y` one-based |
| **370** | `QueryRootViews` | |
| **371** | `QueryViewRect` | |
| **372** | `QueryViewParent` | |
| **373** | `QueryViewChildren` | |
| **374** | `QueryMenuBarState` | Menu bar widget only |
| **382** | `QueryViewState` | Resolved retained state |
| **383** | `QueryViewOptions` | Retained behavior options |
| **384** | `QueryResolvedView` | Geometry, clip, state, options |
| **385** | `QueryViewKind` | Native widget kind |
| **386** | `QuerySceneGraph` | Consistent full-tree snapshot |
| **387** | `HostSetViewVisible` | Retained visibility flag |
| **388** | `HostSetViewEnabled` | Retained input/focus flag |
| **389..=393** | `HostCreate*` controls | Label, button, input, checkbox, radio |
| **394..=396** | `HostSet*` control state | Input text, checked, selection |
| **397..=399** | `Query*State` controls | Input, checkbox, radio state |
| **400..=403** | List-box API | Create, replace, select, query |

### Headless test flow (example)

```pascal
program MenuHoverTest;
uses Std.Console, Std.Tui, Std.Test;

begin
  var App: Application := Application.OpenForTest(80, 25);
  var Bar: ViewId := Application.HostCreateMenuBarView(App, 0, 0, 80, 1, MenuItems(), MenuStyle());
  Application.Configure(App, Handlers);
  Application.RequestRedraw(App);
  Application.TestPump(App);
  Application.TestMoveMouse(App, 2, 1);
  Application.TestPump(App);
  AssertScreenCell(2, 1, 'F', LightGray, Black);
  AssertEquals(0, Application.QueryMenuBarState(App, Bar).hoveredIndex);
  Application.TestClickMouse(App, 2, 1);
  Application.TestPumpUntilIdle(App);
  Application.TestMoveMouse(App, 2, 4);
  Application.TestPumpUntilIdle(App);
  AssertEquals(1, Application.QueryMenuBarState(App, Bar).selectedEntry);
  Application.CloseForTest(App)
end.
```

`Std.Test` helpers `AssertScreenLine`, `AssertScreenCell`, and `AssertViewRect` (when `uses Std.Tui` is present) wrap the query intrinsics; see [`test.md`](../../testing/test.md).

### Where to test what

| Layer | Test where | Why |
| ----- | ---------- | --- |
| Pure widget routing (hit-testing, geometry) | Rust unit tests in `fpas-std` | Fast, no VM |
| App flows, dispatch, hover-to-screen, modal scope | FPAS `*_test.fpas` | Integrated host + dispatch path |
| Real terminal during `Run` | [Terminal checklist](../terminal-checklist.md) | Alternate screen, cursor, flicker, live resize |

### Example tests

| Path | Topic |
| ---- | ----- |
| [`tui_pump_test.fpas`](../../../../tests/tui/host/tui_pump_test.fpas) | Open/pump/close smoke |
| [`tui_inject_key_test.fpas`](../../../../tests/tui/host/tui_inject_key_test.fpas) | `TestSendKey` + `OnKeyPressed` |
| [`tui_escape_test.fpas`](../../../../tests/tui/host/tui_escape_test.fpas) | Escape + `AssertScreenLine` |
| [`tui_mouse_test.fpas`](../../../../tests/tui/host/tui_mouse_test.fpas) | `TestSendMouse` + `OnMouse` |
| [`tui_screen_query_test.fpas`](../../../../tests/tui/host/tui_screen_query_test.fpas) | Screen queries after paint |
| [`tui_view_query_test.fpas`](../../../../tests/tui/scene/tui_view_query_test.fpas) | View rect + initial menu state |
| [`tui_menu_bar_hover_test.fpas`](../../../../tests/tui/menu/tui_menu_bar_hover_test.fpas) | Bar hover colors |
| [`tui_menu_hover_test.fpas`](../../../../tests/tui/menu/tui_menu_hover_test.fpas) | Capstone: bar hover + submenu selection |
| [`tui_show_dialog_test.fpas`](../../../../tests/tui/modals/tui_show_dialog_test.fpas) | `ShowDialog`, modal Escape, `HostSetActiveModalResult`, owned-root cleanup |
| [`tui_framed_dialog_test.fpas`](../../../../tests/tui/modals/tui_framed_dialog_test.fpas) | Painted owned frame, modal depth, and automatic subtree cleanup |
| [`tui_framed_dialog_controls_test.fpas`](../../../../tests/tui/modals/tui_framed_dialog_controls_test.fpas) | Painted dialog frame with host label/button children and command routing |
| [`tui_frame_occlusion_test.fpas`](../../../../tests/tui/frames/tui_frame_occlusion_test.fpas) | Overlapping painted windows: close front frame repairs occluded back cells |
| [`tui_frame_scroll_clip_test.fpas`](../../../../tests/tui/frames/tui_frame_scroll_clip_test.fpas) | Frame scroll offset clips child paint to the inner viewport |
| [`tui_frame_occlusion_move_test.fpas`](../../../../tests/tui/frames/tui_frame_occlusion_move_test.fpas) | Overlapping windows: moving the front frame repairs previously occluded cells |
| [`tui_frame_occlusion_zoom_test.fpas`](../../../../tests/tui/frames/tui_frame_occlusion_zoom_test.fpas) | Overlapping windows: zoom/restore repairs cells outside the normal frame bounds |
| [`tui_frame_occlusion_resize_test.fpas`](../../../../tests/tui/frames/tui_frame_occlusion_resize_test.fpas) | Overlapping windows: shrinking a frame repairs newly exposed back cells |
| [`tui_frame_reserved_commands_test.fpas`](../../../../tests/tui/frames/tui_frame_reserved_commands_test.fpas) | Reserved command ids `-1`..=`-4` via keyboard shortcuts |
| [`tui_view_clip_test.fpas`](../../../../tests/tui/scene/tui_view_clip_test.fpas) | Effective clip during view-local paint |
| [`tui_scene_graph_query_test.fpas`](../../../../tests/tui/scene/tui_scene_graph_query_test.fpas) | Scene structure, state, options, clip, kind, and paint order |

### ScreenCell type (decided)

`QueryScreenCell` returns a **`ScreenCell`** record. Colors use the **packed CRT model** (`0..=15`), same as `TextColor` / `TextBackground` and host widgets such as `MenuBarStyle`.

```pascal
type ScreenCell = record
  ch: string;
  fg: integer;   { CRT foreground 0..15 — use Std.Console color constants }
  bg: integer;    { CRT background 0..15 }
end;
```

**Rationale**

- The TUI hosted back buffer stores **packed 16-color attributes** per cell (`ConsoleState`), not truecolor or 256-color palette state.
- Menu hover tests assert the same colors `MenuBarStyle` already uses (`LightGray`, `Black`, …).
- Assertions stay readable: `AssertEquals(LightGray, Cell.fg)` with `uses Std.Console`.

**Rules**

| Rule | Detail |
| ---- | ------ |
| Valid range | `fg` and `bg` are always `0..=15` for cells returned by `QueryScreenCell`. |
| Constants | Prefer `Std.Console` names (`Black`, `LightGray`, `Red`, …) over raw integers in tests. |
| Truecolor / 256-color | **Out of scope for v1.** Extended terminal colors from `TextColorRGB` / `TextColor256` are not represented in the logical screen model; do not add a richer `ScreenCell` shape until the back buffer supports it. |
| Errors | Out-of-bounds coordinates are a runtime error with a concrete hint (row/column and screen size). |

### MenuBarState type

`Application.QueryMenuBarState` returns a snapshot of menu bar hover, keyboard activation, and pull-down state for a `HostCreateMenuBarView` widget.

```pascal
type MenuBarState = record
  menuActive: boolean;       { keyboard menu navigation mode }
  hoveredIndex: integer;     { top-level bar index, or -1 }
  submenuOpen: boolean;
  submenuBarIndex: integer;  { bar item owning the open popup, or -1 }
  selectedEntry: integer;    { highlighted popup row, or -1 }
end;
```

Sentinel **`-1`** means “none” for index fields until `Option of integer` migration.

### Sidecar deprecation (TUI removed in Phase 8.1)

Runner sidecars **overlap** with the native FPAS test API and are **deprecated** for TUI work:

| Sidecar | Status | Replacement |
| ------- | ------ | ----------- |
| `<test>.script.toml` (console/TUI events) | **Removed** (Phase 8.1) | `TestSendKey`, `TestMoveMouse`, … + `TestPump` |
| `<test>.expect.screen` | **Removed** (Phase 8.1) | `QueryScreenLine` / `QueryScreenCell` + `Std.Test` assertions |
| `<test>.script.toml` (`readln` / graph) | **Removed** (legacy sidecars) | `Std.Test.PushReadLn`, `Application.OpenForTest` + `TestSendKey` |
| `<test>.expect.stdout` | **Keep** | Non-TUI output tests |
| `<test>.expect.pixels` | **Keep** | Headless graph |

Affected implementation paths (TUI sidecars removed in Phase 8.1):

- ~~`crates/fpas-cli/src/test_script/console.rs`~~ — removed (console/TUI script events)
- ~~`crates/fpas-cli/src/cli_test/expect_screen.rs`~~ — removed (golden screen compare)
- ~~`tests/tui/tui_escape_test.script.toml`~~, ~~`tui_mouse_test.script.toml`~~, ~~`tui_escape_test.expect.screen`~~ — migrated to native FPAS tests
- ~~`tests/console/readln_test.script.toml`~~, ~~`readln_order_test.script.toml`~~, ~~`tests/graph/graph_smoke_test.script.toml`~~ — migrated to `PushReadLn` / graph test APIs

`*.script.toml` remains available for `[test.overrides]` and `--script` only; new tests should use native FPAS injectors.

## See also

- [`Std.Test`](../../testing/test.md)
- [Terminal checklist](../terminal-checklist.md)
- [Hosted dispatch overview](README.md)
