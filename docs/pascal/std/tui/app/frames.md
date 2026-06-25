# Frame roots (`Std.Tui`)

Host-managed frame views provide validated window/dialog geometry, painted Turbo Vision-style
chrome, desktop constraints, and title-bar move, border resize, zoom/restore, and next-window
activation.

**See also:** [TUI application hub](README.md), [VM bridge](vm-bridge.md), [Controls](controls.md).

## Quick reference

| Call | Role |
| ---- | ---- |
| `Application.HostSetDesktopWorkArea(App, X, Y, Width, Height)` | Configure the desktop rectangle that constrains frame roots. Returns `false` when the rectangle is empty. |
| `Application.HostCreateFrameView(App, X, Y, Width, Height, Title, Kind, Movable, Resizable, Zoomable, Scrollable, Closable)` | Create and paint one frame root (`Kind`: `0` = Window, `1` = Dialog). Returns `ViewId`. |
| `Application.ShowFramedDialog(App, ModalId, X, Y, Width, Height, Title, Movable, Resizable, Zoomable, Scrollable, Closable)` | Atomically create an owned painted dialog frame and enter it modally. Returns `ViewId`. |
| `Application.HostActivateNextWindow(App)` | Raise and focus the next eligible root in z-order. Returns whether a root was activated. |
| `Application.HostZoomFrameRoot(App, ViewId)` | Zoom a zoomable root to the desktop work area. |
| `Application.HostRestoreFrameRoot(App, ViewId)` | Restore a zoomed root to its saved rectangle. |
| `Application.QueryFrameRootState(App, ViewId)` | Query outer geometry, capability flags, and zoom state. |
| `Application.HostCascadeFrameRoots(App, StepX, StepY)` | Cascade window roots diagonally from the work-area origin; each root keeps its size. Returns the number of roots repositioned. Typical steps are `2` and `1`. |
| `Application.HostTileFrameRoots(App)` | Resize and arrange window roots in a grid that fills the desktop work area. Returns the number of roots repositioned. |
| `Application.QueryFrameWindowList(App)` | List open window-kind frame roots in back-to-front z-order with titles and active flags. |
| `Application.HostActivateFrameWindow(App, ViewId)` | Raise and focus one frame root. Returns whether activation succeeded. |
| `Application.HostSetFrameContentSize(App, FrameView, ContentWidth, ContentHeight)` | Replace logical content size for a scrollable frame root and refresh scroll-bar geometry. |
| `Application.HostScrollFrame(App, FrameView, DeltaX, DeltaY)` | Scroll a scrollable frame root by signed cell deltas. |
| `Application.HostSetFrameScrollOffset(App, FrameView, OffsetX, OffsetY)` | Set absolute scroll offsets for a scrollable frame root. |
| `Application.QueryFrameScrollState(App, FrameView)` | Query scroll offsets and logical content size. |

## `FrameScrollState`

| Field | Type | Meaning |
| ----- | ---- | ------- |
| `offsetX`, `offsetY` | `integer` | Current scroll offsets in terminal cells. |
| `contentWidth`, `contentHeight` | `integer` | Logical content size used for scroll-bar visibility. |

## `FrameRootState`

| Field | Type | Meaning |
| ----- | ---- | ------- |
| `x`, `y`, `width`, `height` | `integer` | Current outer frame rectangle in terminal cells. |
| `kind` | `integer` | `0` = Window, `1` = Dialog. |
| `movable`, `resizable`, `zoomable`, `scrollable`, `closable` | `boolean` | Implemented capability flags. |
| `zoomed` | `boolean` | `true` when a pre-zoom rectangle is stored. |

## Rendering

`HostCreateFrameView` attaches a native `FrameWidget` (`ViewKind.Frame`). Window frames use an
active light-blue title/border and inactive blue title/border; dialog frames use a gray palette.
The client area is painted before the frame's local handler and descendants. The double-line
border, title, and enabled `■` / `▲` / `▼` chrome cells are painted afterward, so child views cannot
overwrite frame chrome. Titles that exceed their slot end with `…`.

When `Closable` is `true`, clicking the title-bar `■` dispatches the built-in close command for that
frame root. Owned modal frames unregister on close; non-modal frames are removed from the scene
graph. When `Zoomable` is `true`, clicking `▲` zooms the frame to the desktop work area and `▼`
restores the saved rectangle.

Children are clipped for paint, hit-testing, focus eligibility, and scene queries to the resolved
inner frame viewport. Direct children use **view coordinates** (origin at the top-left of the
viewport). The host applies scroll offsets when resolving descendant geometry.

When `Scrollable` is `true`, the frame paints integrated `▲█▼` / `◄█►` scroll chrome on its border
columns and rows. Scroll bars appear when logical content exceeds the viewport. Use
`HostSetFrameContentSize` to set content size explicitly; when content size remains `(0, 0)`, the
host measures direct-child bounds before resolving geometry. Mouse wheel over the viewport scrolls
vertically after child controls decline the event; arrow, page, home, and end keys scroll the
containing frame after focused descendant handling. Scroll-bar arrows, track clicks, and thumb drags
target frame chrome directly.

## Pointer interaction

When a frame root is movable or resizable, the host routes title-bar and border hits before ordinary widget mouse dispatch:

- **Title-bar drag** moves the root with pointer capture until mouse up.
- **Border/corner drag** resizes the root with pointer capture until mouse up.
- **Title-bar click** on a non-draggable frame still activates (raises) the root.
- **Close (`■`)** removes a non-modal frame or closes an owned modal frame when `Closable` is `true`.
- **Zoom (`▲`) / restore (`▼`)** toggle zoom state when `Zoomable` is `true`.

Use `Application.TestSendMouse`, `Application.TestMoveMouse`, and `Application.TestPumpUntilIdle` in headless tests.

## Window layout helpers

`HostCascadeFrameRoots` and `HostTileFrameRoots` operate on registered **window-kind** frame roots only. Dialog roots, zoomed roots, and the active modal root (when present) are skipped. Both calls require a configured desktop work area and return `0` when nothing was repositioned.

## MDI window list

`QueryFrameWindowList` returns `array of FrameWindowEntry` for every **window-kind** frame root in back-to-front z-order. The active modal root is excluded when present. Use `HostActivateFrameWindow` to raise and focus a listed root (for example after the user picks one from a popup menu).

### `FrameWindowEntry`

| Field | Type | Meaning |
| ----- | ---- | ------- |
| `id` | `ViewId` | Frame root handle |
| `title` | `string` | Title-bar text |
| `kind` | `integer` | `0` = Window, `1` = Dialog (always `0` in this query today) |
| `active` | `boolean` | Whether this root is the active window root, or the frontmost root when no view is focused |
| `zIndex` | `integer` | Back-to-front position among roots (`0` = back) |

## Reserved command ids

Bind these through `Application.HostBindCommand` to dispatch built-in window actions before `OnCommand`:

| `CommandId` | Action |
| ----------- | ------ |
| `-1` | Activate the next window root (`NextWindow`). |
| `-2` | Zoom the source or active frame root (`Zoom`). |
| `-3` | Restore the source or active zoomed frame root (`ZoomBack`). |
| `-4` | Close the source frame root or owned modal frame (`Close`). |

Global `HostBindCommand` resolves `-1` (`NextWindow`) without a source view. Bind `-2`..=`-4` on the
frame root with `HostBindCommandToView` (or ensure keyboard focus is inside the frame) so the host
can determine the source root.

Application-defined command ids remain non-negative and still flow through `OnCommand`.

## Implementation (contributors)

| Layer | Location |
| ----- | -------- |
| Geometry + painting + interaction | `crates/fpas-std/src/tui/widget/frame/` |
| VM bridge | `crates/fpas-vm/src/vm/execute/io/tui/frame_model/`, `views/modal.rs` |
| Intrinsics **410..=422**, **428..=429** | `crates/fpas-bytecode/src/intrinsic/tui.rs` |
| FPAS tests | `tests/tui/tui_frame_chrome_test.fpas`, `tests/tui/tui_frame_chrome_actions_test.fpas`, `tests/tui/tui_framed_dialog_test.fpas`, `tests/tui/tui_frame_window_test.fpas`, `tests/tui/tui_frame_layout_test.fpas`, `tests/tui/tui_frame_scroll_test.fpas`, `tests/tui/tui_frame_window_list_test.fpas`, `tests/tui/tui_frame_reserved_commands_test.fpas`, `tests/tui/tui_frame_occlusion_move_test.fpas`, `tests/tui/tui_frame_occlusion_zoom_test.fpas`, `tests/tui/tui_frame_occlusion_resize_test.fpas` |
| Examples | [`examples/pascal/tui/framed_window.fpas`](../../../../examples/pascal/tui/framed_window.fpas), [`framed_dialog.fpas`](../../../../examples/pascal/tui/framed_dialog.fpas) |
