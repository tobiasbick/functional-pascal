# Views and focus

Retained view tree, coordinates, clipping, focus traversal, and paint order for hosted `Std.Tui` applications.

Intrinsic reference: [VM bridge](vm-bridge.md) — `HostRegisterView`, `HostSetViewParent`, `HostSetViewRect`, `HostSetViewVisible`, `HostSetViewEnabled`, `HostPushChildView`, `HostRegisterOnViewPaint`, and host widget constructors.

## View handles

`Std.Tui.ViewId` is an opaque host-owned handle. Only host routines construct values (`HostRegisterView`, `HostCreateMenuBarView`, `ShowDialog`, …). Compare handles with `=`; use `Option of ViewId` for “no view”.

See [ViewId rules](testing.md#viewid-type-decided) in the native testing page.

## View tree

Each view has:

- an absolute **screen rectangle** (after resolving parent chain);
- an optional **parent** (`HostSetViewParent`; `None` places the view in the root list);
- ordered **children** (sibling order is z-order within the parent);
- **state** (`Visible`, `Enabled`, focus-related flags) managed by the host;
- optional **native widget** paint/input (menu bar, status bar, solid fill);
- optional **Pascal paint handler** (`HostRegisterOnViewPaint`).

Root views use **absolute terminal coordinates**. After reparenting, `HostSetViewRect` interprets `X` and `Y` **relative to the parent**. Reparenting preserves the current absolute rectangle; only subsequent rect updates use parent-relative coords.

`Width` and `Height` must be greater than zero for registration, widget creation, and `HostSetViewRect`.

## View state mutation

`Application.HostSetViewVisible(App, ViewId, Visible)` changes the retained visibility flag. A hidden
view and all its descendants resolve as `visible = false`, have no effective clip, cannot receive
focus or pointer input, and remain present in `QuerySceneGraph`. Restoring visibility is still
subject to ancestor visibility and clipping.

`Application.HostSetViewEnabled(App, ViewId, Enabled)` changes whether that view accepts input and
focus. Disabling the focused view immediately clears focus. The flag applies to the specified node;
it does not rewrite descendant flags.

Both calls update retained state immediately and request redraws for the affected subtree. As with
the other `HostSetView*` mutators, an unknown or unregistered `ViewId` is ignored.

## Resolved geometry and clipping

The host resolves each node to:

- one absolute screen rectangle;
- a **content origin** for local painting;
- an **effective clip** from ancestor bounds; frame roots further restrict descendants to their
  inner content viewport.

Paint, hit-testing, focus eligibility, damage, and `QueryViewRect` consume the same resolved record. `QueryViewRect` returns the absolute rectangle **before** clipping.

During `OnViewPaint`, the `Bounds` argument is local (`x = 0`, `y = 0`, `width`, `height`). Console coordinates such as `GotoXY(1, 1)` address the view's top-left cell. CRT writes are hard-clipped to the effective ancestor clip for the duration of the callback.

## Focus and Tab traversal

Focus is derived from **tree order and view state**, not from a separate global focus list.

`Application.HostPushChildView(App, ViewId)` is a compatibility adapter: it marks a view **selectable** and a **Tab stop**. Call it for each control that should participate in keyboard focus.

Tab / Shift+Tab skip views that are hidden, disabled, fully clipped, non-selectable, or not Tab stops. **Pointer-down** on a selectable view moves focus to that view (and activates its containing window root in the retained engine).

`Application.QueryFocusedViewId(App): Option of ViewId` returns the focused **leaf**, or `None`.

`OnDeactivate` / `OnActivate` fire when the focused leaf changes through traversal or pointer-down. They currently receive only `Application` (no `ViewId` parameters).

Groups track a **current child** along the active focus path internally.

## Paint order

During hosted redraw:

1. **Global `OnPaint`** runs first when registered (full-screen logical frame; still used for backgrounds and status text in many apps).
2. Each **root view** is painted **depth-first**:
   - native widget **underlay**;
   - view-local **Pascal handler**;
   - **child subtrees**;
   - widget **overlays** (for example an open menu popup).
3. A final **menu overlay layer** paints topmost menu popups that must sit above sibling content.

When a view has both a native widget and `OnViewPaint`, the widget base paints first, then the Pascal handler. Widget overlays paint after local handlers so popups are not covered.

`OnPaint` may be an empty no-op when host widgets paint the entire chrome (see `apps/ide/src/shell.fpas`).

## Pointer routing and capture

Mouse dispatch:

1. resolve a target from **pointer capture** or the topmost enabled clipped view under the pointer;
2. suppress events outside the **active modal scope** when applicable;
3. move focus on pointer-down when the target is selectable;
4. route to **menu bar widgets** before other widgets under the pointer;
5. fall back to **`OnMouse`** when registered.

**Pointer capture** keeps delivering move/up events to the capturing view after the pointer leaves the original hit rectangle (used internally for drag-like interactions). Terminal focus loss and view removal release capture.

## Commands

Command resolution order (most local first):

1. command map on the **focused view**, then each **ancestor**;
2. **active modal frame** command map;
3. **global** map from `HostBindCommand`.

View-local bindings: `HostBindCommandToView`. Modal-local bindings: `HostBindCommandToActiveModal`.

## Query API

Read-only introspection (headless tests and debugging):

| Call | Returns |
| ---- | ------- |
| `QueryRootViews` | Root handles in z-order |
| `QueryViewRect` | Absolute `Rect` |
| `QueryViewParent` | `Option of ViewId` |
| `QueryViewChildren` | Direct children in sibling order |
| `QueryViewState` | Resolved visibility, enabled, focus, active, and exposure flags |
| `QueryViewOptions` | Selectability, Tab, routing, and child-clipping options |
| `QueryResolvedView` | Absolute rect, effective clip, state, and options |
| `QueryViewKind` | Native widget kind or `Generic` |
| `QuerySceneGraph` | Full `array of ViewSnapshot` in paint order |
| `QueryFocusedViewId` | Focused leaf or `None` |

The single-view queries require a live registered `ViewId`. `QuerySceneGraph` takes one consistent
read-only snapshot while holding the TUI state lock and includes hidden and fully clipped nodes.
See [Types](types.md#scene-graph-introspection-types) and
[Native testing](testing.md#screen-and-view-introspection-query--implemented).

## Active windows (retained engine)

The Rust retained registry tracks an **active window root** (raise on click-to-front, focus moves into that subtree). This policy is used internally for overlapping roots and modal return focus. There is **no** Pascal `HostActivateRoot` yet; overlapping-window behavior is validated in Rust headless tests under `fpas-std/src/tui/widget/frame/tests.rs`.

## Examples

| Path | Topic |
| ---- | ----- |
| [`local_view_paint.fpas`](../../../../examples/pascal/tui/local_view_paint.fpas) | Parent-relative layout and `OnViewPaint` |
| [`view_scoped_commands.fpas`](../../../../examples/pascal/tui/view_scoped_commands.fpas) | Focus-aware command maps |
| [`show_dialog.fpas`](../../../../examples/pascal/tui/show_dialog.fpas) | Owned dialog subtree under a modal root |

## Implementation (contributors)

| Concern | Location |
| --- | ---- |
| View registry and focus | [`fpas-std/src/tui/view/`](../../../../../crates/fpas-std/src/tui/view/) |
| Depth-first redraw | [`fpas-vm/.../host/redraw.rs`](../../../../../crates/fpas-vm/src/vm/execute/io/tui/host/redraw.rs) |
| Event routing | [`fpas-vm/.../host/process/`](../../../../../crates/fpas-vm/src/vm/execute/io/tui/host/process/) |

## See also

- [Modals and dialogs](modals.md)
- [VM bridge](vm-bridge.md)
- [Handlers](handlers.md)
- [Native testing](testing.md)
