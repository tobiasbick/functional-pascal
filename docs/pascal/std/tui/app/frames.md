# Frame roots (`Std.Tui`)

Host-managed frame roots provide validated window/dialog geometry, desktop constraints, and chrome interaction (title-bar move, border resize, zoom/restore, next-window activation) before the full `FrameWidget` painter lands.

**See also:** [TUI application hub](README.md), [VM bridge](vm-bridge.md), [Controls](controls.md).

## Quick reference

| Call | Role |
| ---- | ---- |
| `Application.HostSetDesktopWorkArea(App, X, Y, Width, Height)` | Configure the desktop rectangle that constrains frame roots. Returns `false` when the rectangle is empty. |
| `Application.HostCreateFrameRootView(App, X, Y, Width, Height, Kind, Movable, Resizable, Zoomable, Scrollable)` | Register one frame root (`Kind`: `0` = Window, `1` = Dialog). Returns `ViewId`. |
| `Application.HostActivateNextWindow(App)` | Raise and focus the next eligible root in z-order. Returns whether a root was activated. |
| `Application.HostZoomFrameRoot(App, ViewId)` | Zoom a zoomable root to the desktop work area. |
| `Application.HostRestoreFrameRoot(App, ViewId)` | Restore a zoomed root to its saved rectangle. |
| `Application.QueryFrameRootState(App, ViewId)` | Query outer geometry, capability flags, and zoom state. |

## `FrameRootState`

| Field | Type | Meaning |
| ----- | ---- | ------- |
| `x`, `y`, `width`, `height` | `integer` | Current outer frame rectangle in terminal cells. |
| `kind` | `integer` | `0` = Window, `1` = Dialog. |
| `movable`, `resizable`, `zoomable`, `scrollable` | `boolean` | Implemented capability flags. |
| `zoomed` | `boolean` | `true` when a pre-zoom rectangle is stored. |

## Pointer interaction

When a frame root is movable or resizable, the host routes title-bar and border hits before ordinary widget mouse dispatch:

- **Title-bar drag** moves the root with pointer capture until mouse up.
- **Border/corner drag** resizes the root with pointer capture until mouse up.
- **Title-bar click** on a non-draggable frame still activates (raises) the root.

Use `Application.TestSendMouse`, `Application.TestMoveMouse`, and `Application.TestPumpUntilIdle` in headless tests.

## Reserved command ids

Bind these through `Application.HostBindCommand` to dispatch built-in window actions before `OnCommand`:

| `CommandId` | Action |
| ----------- | ------ |
| `-1` | Activate the next window root (`NextWindow`). |
| `-2` | Zoom the source or active frame root (`Zoom`). |
| `-3` | Restore the source or active zoomed frame root (`ZoomBack`). |

Application-defined command ids remain non-negative and still flow through `OnCommand`.

## Implementation (contributors)

| Layer | Location |
| ----- | -------- |
| Geometry + interaction | `crates/fpas-std/src/tui/widget/frame/` |
| VM bridge | `crates/fpas-vm/src/vm/execute/io/tui/frame_model/` |
| Intrinsics **410..=415** | `crates/fpas-bytecode/src/intrinsic/tui.rs` |
| FPAS tests | `tests/tui/tui_frame_window_test.fpas` |
