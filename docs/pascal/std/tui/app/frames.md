# Frame roots (`Std.Tui`)

Host-managed frame views provide validated window/dialog geometry, painted Turbo Vision-style
chrome, desktop constraints, and title-bar move, border resize, zoom/restore, and next-window
activation.

**See also:** [TUI application hub](README.md), [VM bridge](vm-bridge.md), [Controls](controls.md).

## Quick reference

| Call | Role |
| ---- | ---- |
| `Application.HostSetDesktopWorkArea(App, X, Y, Width, Height)` | Configure the desktop rectangle that constrains frame roots. Returns `false` when the rectangle is empty. |
| `Application.HostCreateFrameView(App, X, Y, Width, Height, Title, Kind, Movable, Resizable, Zoomable, Scrollable)` | Create and paint one frame root (`Kind`: `0` = Window, `1` = Dialog). Returns `ViewId`. |
| `Application.HostActivateNextWindow(App)` | Raise and focus the next eligible root in z-order. Returns whether a root was activated. |
| `Application.HostZoomFrameRoot(App, ViewId)` | Zoom a zoomable root to the desktop work area. |
| `Application.HostRestoreFrameRoot(App, ViewId)` | Restore a zoomed root to its saved rectangle. |
| `Application.QueryFrameRootState(App, ViewId)` | Query outer geometry, capability flags, and zoom state. |
| `Application.HostCascadeFrameRoots(App, StepX, StepY)` | Cascade window roots diagonally from the work-area origin; each root keeps its size. Returns the number of roots repositioned. Typical steps are `2` and `1`. |
| `Application.HostTileFrameRoots(App)` | Resize and arrange window roots in a grid that fills the desktop work area. Returns the number of roots repositioned. |

## `FrameRootState`

| Field | Type | Meaning |
| ----- | ---- | ------- |
| `x`, `y`, `width`, `height` | `integer` | Current outer frame rectangle in terminal cells. |
| `kind` | `integer` | `0` = Window, `1` = Dialog. |
| `movable`, `resizable`, `zoomable`, `scrollable` | `boolean` | Implemented capability flags. |
| `zoomed` | `boolean` | `true` when a pre-zoom rectangle is stored. |

## Rendering

`HostCreateFrameView` attaches a native `FrameWidget` (`ViewKind.Frame`). Window frames use an
active light-blue title/border and inactive blue title/border; dialog frames use a gray palette.
The client area is painted before the frame's local handler and descendants. The double-line
border, title, and enabled `▲` / `▼` zoom cells are painted afterward, so child views cannot
overwrite frame chrome. Titles that exceed their slot end with `…`.

Frame-integrated scrolling and close chrome are not implemented yet. `Scrollable` currently
participates in validated geometry only; use standalone `ScrollView` or `ScrollBar` controls for
interactive scrolling.

## Pointer interaction

When a frame root is movable or resizable, the host routes title-bar and border hits before ordinary widget mouse dispatch:

- **Title-bar drag** moves the root with pointer capture until mouse up.
- **Border/corner drag** resizes the root with pointer capture until mouse up.
- **Title-bar click** on a non-draggable frame still activates (raises) the root.

Use `Application.TestSendMouse`, `Application.TestMoveMouse`, and `Application.TestPumpUntilIdle` in headless tests.

## Window layout helpers

`HostCascadeFrameRoots` and `HostTileFrameRoots` operate on registered **window-kind** frame roots only. Dialog roots, zoomed roots, and the active modal root (when present) are skipped. Both calls require a configured desktop work area and return `0` when nothing was repositioned.

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
| Geometry + painting + interaction | `crates/fpas-std/src/tui/widget/frame/` |
| VM bridge | `crates/fpas-vm/src/vm/execute/io/tui/frame_model/` |
| Intrinsics **410..=415** | `crates/fpas-bytecode/src/intrinsic/tui.rs` |
| FPAS tests | `tests/tui/tui_frame_chrome_test.fpas`, `tests/tui/tui_frame_window_test.fpas`, `tests/tui/tui_frame_layout_test.fpas` |
