# Native TUI testing in FPAS (proposal)

**Status:** design only — not implemented.

Goal: **test the hosted `Std.Tui` surface completely from inside FPAS**, using `fpas test` as the only runner. A `*_test.fpas` program should be able to build a UI, drive input, and assert on the resulting host state (screen, views, focus, modal stack, widget state) without Rust integration tests, golden sidecar files, or an external control server.

This is the in-process counterpart to the out-of-process [TUI control server](../tui-test/README.md). Both can coexist; this document covers the **language / standard-library** path.

**Implementation tracking:** [`implementation-plan.md`](implementation-plan.md) — resumable, checkbox-driven plan with file anchors and verification steps. Start there to begin or continue the work.

## Mandate

This is a hobby project with **no legacy or backward-compatibility constraints**. Any existing API, intrinsic, sidecar format, or test convention may be **redesigned or removed** if it makes native TUI testing cleaner. Prefer the best end state over preserving current shapes. Existing surfaces referenced here (`HostQueryFocusedViewId`, `*.script.toml`, `*.expect.screen`) are starting points, not fixed contracts.

## Why this is currently impossible

A FPAS test today can only observe the hosted TUI through:

- Behavior callbacks (`OnCommand`, `OnKeyPressed`, …) setting `mutable var` flags.
- Two scalar host queries: `Application.HostQueryFocusedViewId`, `Application.HostModalDepth`.
- Runner-side golden files (`*.expect.screen`) that compare **characters only** — no colors, no per-cell attributes.

Consequences:

- **Hover is untestable in FPAS.** Menu-bar hover changes only colors and internal widget state; no FPAS-visible callback fires, and `*.expect.screen` ignores color. Submenu hover-by-mouse is not even implemented in the host widget.
- **Layout/geometry is untestable in FPAS.** View rectangles, z-order, and child trees live only in Rust (`ViewRegistry`).
- **Input is pre-queued, not interactive.** All scripted events are pushed before `vm.run()`, so a test cannot do "inject → observe → inject" within one program.
- **Widget internals are opaque.** Open submenu, hovered index, selected popup row are Rust-private (`MenuBarWidget`).

The missing piece is **observability and step control exposed to FPAS**, not the test runner itself.

## Target experience

A complete menu hover test, fully in FPAS, should read like this:

```pascal
program MenuHoverTest;

uses Std.Console, Std.Tui, Std.Test;

begin
  var App: Application := Application.OpenForTest(80, 25);

  var Bar: ViewId := Application.HostCreateMenuBarView(
    App, 0, 0, 80, 1, MenuItems(), MenuStyle());

  { Move the mouse over the "File" label. }
  Application.TestMoveMouse(App, 3, 1);
  Application.TestPump(App);

  { Hover is reflected in widget state and on screen. }
  var State: MenuBarState := Application.QueryMenuBarState(App, Bar);
  AssertEquals(0, State.hoveredIndex);

  var Cell: ScreenCell := Application.QueryScreenCell(App, 2, 0);
  AssertEquals(Color.LightGray, Cell.fg);   { highlight color, not bar color }

  { Open the submenu, hover the second entry, assert selection. }
  Application.TestClickMouse(App, 3, 1);
  Application.TestPump(App);
  Application.TestMoveMouse(App, 4, 3);
  Application.TestPump(App);

  var Sub: MenuBarState := Application.QueryMenuBarState(App, Bar);
  AssertTrue(Sub.submenuOpen);
  AssertEquals(1, Sub.selectedEntry);

  Application.CloseForTest(App)
end.
```

Everything above runs under `fpas test` with no real terminal, no Rust test, no golden file.

## Capability gaps to close

The work splits into four capabilities. Each is independently useful.

### 1. Headless deterministic host loop

A test must run the hosted loop **step by step**, not block in `Application.Run`.

- `Application.OpenForTest(Width, Height)` — open a TUI session bound to a fixed-size virtual screen, no terminal writer, no alt-screen.
- `Application.TestPump(App)` — process exactly one queued event plus the resulting redraw (the existing internal `HostProcessNext` / `DispatchRedraw` made first-class for tests).
- `Application.TestPumpUntilIdle(App)` — drain all queued events and settle redraws.
- `Application.CloseForTest(App)` — tear down deterministically.

Rationale: today input is pre-queued and `Application.Run` owns the loop. Tests need to interleave injection and observation.

### 2. First-class input injection from FPAS

Replace the pre-run-only `*.script.toml` path with FPAS-callable injectors (scripts can remain as sugar, or be removed):

- `Application.TestSendKey(App, Key)` where `Key: KeyEvent`.
- `Application.TestMoveMouse(App, X, Y)` — emits a `Move` mouse event.
- `Application.TestClickMouse(App, X, Y)` — `Down` then `Up`.
- `Application.TestSendMouse(App, Event)` — full control.
- `Application.TestResize(App, Width, Height)`, `TestPaste`, `TestFocus`.

Coordinates follow the documented console convention (one-based for mouse, matching `Std.Console.Event`).

### 3. Screen and cell introspection in FPAS

Promote the CRT back buffer to a FPAS-readable value, including colors:

- `Application.QueryScreenLine(App, Y): string` — characters of one row.
- `Application.QueryScreenCell(App, X, Y): ScreenCell` — `record ch: char; fg: Color; bg: Color end`.
- `Application.QueryScreenSize(App): Size`.

This is what makes **hover (color change)** and **paint correctness** assertable. It supersedes `*.expect.screen` for color-sensitive tests; the golden file can stay for coarse character snapshots or be dropped.

### 4. View tree and widget state introspection in FPAS

Expose the host view registry and widget internals as FPAS values:

- `Application.QueryViewRect(App, ViewId): Rect`.
- `Application.QueryViewParent(App, ViewId): Option of ViewId`.
- `Application.QueryViewChildren(App, ViewId): array of ViewId`.
- `Application.QueryRootViews(App): array of ViewId`.
- `Application.QueryMenuBarState(App, ViewId): MenuBarState` where

  ```pascal
  type MenuBarState = record
    menuActive: boolean;
    hoveredIndex: integer;     { -1 when none }
    submenuOpen: boolean;
    submenuBarIndex: integer;  { -1 when none }
    selectedEntry: integer;    { -1 when none }
  end;
  ```

These turn Rust-private state (`ViewRegistry`, `MenuBarWidget.hovered`, `open_submenu`) into testable FPAS records.

## Prerequisite fixes (not just test plumbing)

Native tests will expose real behavior gaps that must be fixed in the host, not worked around in tests:

1. **Submenu mouse hover.** `MenuBarWidget::handle_mouse` ignores non-`Down` events while a popup is open (`crates/fpas-std/src/tui/widget/menu_bar/input.rs`). Mouse `Move` over popup rows must update the selected entry, mirroring keyboard `Up`/`Down`. Without this, "hover in submenu" has nothing to assert.
2. **Bar hover via `Move`.** Confirm a `Move` over a bar item sets `hovered` and returns `HoverChanged` consistently, and that paint reflects highlight colors.
3. **Deterministic redraw boundary.** `TestPump` must guarantee that the back buffer reflects the dispatched event before the next FPAS line runs.

## Std.Test additions (optional sugar)

Convenience assertions layered on capability 3/4, so tests read clearly:

- `AssertScreenLine(Expected: string; Y: integer)`.
- `AssertScreenCell(X, Y: integer; Ch: char; Fg, Bg: Color)`.
- `AssertViewRect(App: Application; V: ViewId; X, Y, W, H: integer)`.

These are pure wrappers around the query APIs; they add no new host capability.

## What gets redesigned or removed

Per the mandate, candidate changes (decide during implementation):

| Current | Proposed direction |
| ------- | ------------------ |
| `*.script.toml` pre-run injection | Keep only as optional sugar, or remove in favor of FPAS injectors |
| `*.expect.screen` character golden | Keep for coarse snapshots, or replace with `AssertScreenLine` |
| Bare `integer` view handles + `-1` sentinels | Typed `ViewId` + `Option of ViewId`; see [ViewId type decision](../../pascal/std/tui-app.md#viewid-type-decided) |
| `Application.Run` as the only entry | Add `OpenForTest` + `TestPump`; keep `Run` for real apps |
| Rust-only `MenuBarWidget` state | Expose via `QueryMenuBarState` |

No compatibility shims. If a name or shape is wrong, change it.

## Boundaries: FPAS tests vs Rust tests

Even with full FPAS introspection, keep a clear split:

| Layer | Test where | Why |
| ----- | ---------- | --- |
| Pure widget routing (hit-testing math, geometry) | Rust unit tests in `fpas-std` | Fast, no VM, exhaustive edge cases |
| App flows, dispatch, hover-to-screen, modal scope | FPAS `*_test.fpas` | Tests the real integrated host + dispatch path |
| Live exploration / IDE debugging | [Control server](../tui-test/README.md) | Out-of-process, during real `Run` |

Native FPAS tests target the **integration** layer that Rust unit tests cannot reach cleanly and golden files describe too weakly.

## Implementation sketch (when started)

Intrinsics extend the existing `TuiIntrinsic` enum (`crates/fpas-bytecode/src/intrinsic/tui.rs`) and reuse current plumbing:

| Capability | Reuses |
| ---------- | ------ |
| `TestPump` / `TestPumpUntilIdle` | `TuiHostProcessNext`, `TuiHostDispatchRedraw` paths in `crates/fpas-vm/src/vm/execute/io/tui_run.rs` |
| `TestSendKey` / `TestSendMouse` / … | `Vm::push_console_event` (`crates/fpas-vm/src/vm/mod.rs`), mapping in `crates/fpas-cli/src/test_script/console.rs` |
| `QueryScreenCell` / `QueryScreenLine` | `ConsoleState` cell access already used by Rust test helpers (`console.test_cell`) |
| `Query*View*` | `ViewRegistry` in `TuiState` (`crates/fpas-vm/src/vm/shared.rs`) |
| `QueryMenuBarState` | `MenuBarWidget` fields (`crates/fpas-std/src/tui/widget/menu_bar/`) |

Sema registration follows `crates/fpas-sema/src/std_registry/loaded/tui/`; lowering follows `crates/fpas-compiler/src/compiler/std_calls/tui/`. New record types (`ScreenCell`, `MenuBarState`, `Rect`, `ViewId`) are declared in the `Std.Tui` registry — `ViewId` as an empty opaque record per the [ViewId decision](../../pascal/std/tui-app.md#viewid-type-decided).

## Open decisions

1. ~~**Is `ViewId` a real opaque type or still a bare `integer` in FPAS?**~~ **Decided** — real type `Std.Tui.ViewId` (empty opaque record, same pattern as `Application`). Host routines return `ViewId`; missing views use `Option of ViewId` instead of `-1`. Rationale: type safety, readable tests, removes magic sentinels. See [`tui-app.md` § ViewId type](../../pascal/std/tui-app.md#viewid-type-decided).
2. **Do injectors require headless mode, or also work during a live `Run` (for the control server to reuse)?** Sharing one injection path is cleaner.
3. **Keep `*.script.toml` and `*.expect.screen` at all?** They overlap with the FPAS-native path; decide whether to deprecate.
4. **Color representation in `ScreenCell`** — reuse the `Std.Console` CRT color enum (`0..=15`) or a richer color type for future truecolor support.
5. ~~**Naming:** `Test*` vs `Host*` vs `Query*` prefixes.~~ **Decided** — see [`docs/pascal/std/tui-app.md` § Native TUI testing API](../../pascal/std/tui-app.md#naming-convention-decided): `Test*` = pump/inject, `Query*` = read, `Host*` = mutators only; rename `HostQueryFocusedViewId` → `QueryFocusedViewId`, `HostModalDepth` → `QueryModalDepth`.

## Success criteria (when implemented)

1. A `*_test.fpas` opens a headless TUI, injects mouse `Move` over a menu item, pumps, and asserts the highlight color via `QueryScreenCell` — passing under `fpas test`.
2. The same test opens a submenu, hovers an entry by mouse, and asserts `selectedEntry` via `QueryMenuBarState` (requires the submenu-hover host fix).
3. View geometry (`QueryViewRect`) and tree (`QueryRootViews`) are assertable in FPAS.
4. No real terminal, no Rust integration test, and no golden sidecar are needed for the above.
5. `fpas test` remains the single runner; `Application.Run` still serves real apps unchanged.

## Related documentation

| Document | Relevance |
| -------- | --------- |
| [`docs/pascal/std/tui-app.md`](../../pascal/std/tui-app.md) | Hosted TUI API, host queries, widgets |
| [`docs/pascal/std/test.md`](../../pascal/std/test.md) | `fpas test`, sidecars, golden files |
| [`docs/future/tui-test/README.md`](../tui-test/README.md) | Out-of-process control server (sibling approach) |
| [`docs/future/tui-application-framework.md`](../tui-application-framework.md) | Phase 8 quality / scripted terminal work |
| [`docs/rust/tui-terminal-checklist.md`](../../rust/tui-terminal-checklist.md) | Manual real-terminal verification |
