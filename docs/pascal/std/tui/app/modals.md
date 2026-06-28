# Modals and dialogs

Hosted modal scopes, owned dialog roots, validated results, and focus restore on close.

| API | Role |
| --- | ---- |
| `Application.ShowModal` | Scope an existing root view subtree |
| `Application.ShowDialog` | Register a new owned root view and show it modally |
| `Application.ShowFramedDialog` | Atomically create an owned painted dialog frame and show it modally |
| `Application.CloseModal` | Pop the active modal frame |
| `Application.HostEnterModal` / `HostLeaveModal` | Low-level stack push/pop without view ownership |
| `Application.HostSetActiveModalResult` | Store Accept, Cancel, or an application-defined result |
| `Application.HostBindCommandToActiveModal` | Shortcuts active only for the top modal frame |
| `Application.HostAttachViewToActiveModal` | Extend modal scope beyond the root subtree |
| `Application.QueryModalDepth` | Current modal stack depth |

Full intrinsic mapping: [VM bridge](vm-bridge.md). View tree and focus rules: [Views and focus](views.md).

## Modal stack

The host keeps a stack of modal frames. Each frame stores:

- an application-defined **`ModalId`** (`integer`, chosen by the app);
- the **root view** whose subtree defines the default modal scope (`ShowModal`, `ShowDialog`, or `ShowFramedDialog`);
- optional **extra scoped views** attached with `HostAttachViewToActiveModal`;
- modal-local **command bindings** from `HostBindCommandToActiveModal`;
- the **previous active window root** and **previous focused leaf** captured on entry;
- an optional **result** set through `HostSetActiveModalResult` before close.

Nested modals push another frame. Each inner frame saves its own return context. Closing the inner dialog restores focus and the active window root saved for that frame before the outer modal resumes.

## High-level surfaces

### `Application.ShowModal(App, ModalId, RootViewId)`

Use when the modal content already exists as a registered root view (or will be built under an existing root).

On entry the host:

1. raises `RootViewId` to the front of the root z-order;
2. pushes a modal frame scoped to that view's subtree;
3. saves the previous active window root and focused leaf;
4. moves focus into the modal scope when an eligible descendant exists.

Background views remain registered but lose focus, mouse, and command routing until the modal closes.

Example: [`examples/pascal/tui/show_modal_existing_view.fpas`](../../../../../examples/pascal/tui/show_modal_existing_view.fpas)

### `Application.ShowDialog(App, ModalId, X, Y, Width, Height): ViewId`

Use for dialogs whose entire subtree should be owned by the modal frame.

On entry the host:

1. registers a new root view at `(X, Y)` with `Width` × `Height` (absolute terminal coordinates);
2. pushes a modal frame that **owns** that root;
3. saves return context and moves focus into the new subtree when possible;
4. returns the new root `ViewId` so the app can attach children, widgets, and paint handlers.

After `Application.CloseModal`, an owned dialog root and its entire subtree are **unregistered automatically**. The app must not keep using handles from that subtree.

Typical setup after `ShowDialog`:

1. create child views or `HostCreateSolidFillView` for chrome;
2. `HostSetViewParent` / `HostSetViewRect` for layout;
3. `HostPushChildView` on custom generic views that should be focusable;
4. `HostRegisterOnViewPaint` for titles and button captions;
5. `HostBindCommandToActiveModal` for Enter, Escape, and button shortcuts;
6. `RequestRedraw`.

Examples:

- [`examples/pascal/tui/show_dialog.fpas`](../../../../../examples/pascal/tui/show_dialog.fpas) — owned framed dialog with OK/Cancel and modal results
- [`apps/ide/src/dialog.fpas`](../../../../../apps/ide/src/dialog.fpas) — IDE Help → About dialog

### `Application.ShowFramedDialog(...)`

`ShowFramedDialog(App, ModalId, X, Y, Width, Height, Title, Movable, Resizable, Zoomable,
Scrollable, Closable)` performs the same owned-modal lifecycle as `ShowDialog`, but the owned root is a
native gray `FrameWidget`. Geometry is validated before either the view tree or modal stack is
changed. Invalid geometry therefore leaves both unchanged. Children attach to the returned
`ViewId` and are clipped to its inner viewport. Closing the modal unregisters the complete frame
subtree.

Create child views at placeholder `(0, 0)`, call `HostSetViewParent`, then set frame-local layout
with `HostSetViewRect` (see [`tui_show_framed_dialog_controls_test.fpas`](../../../../../tests/tui/modals/tui_show_framed_dialog_controls_test.fpas)).
When the active modal has no focused child yet, the first focusable child that is parented into the
modal and positioned inside the visible dialog viewport becomes the focused leaf. Native controls
such as buttons and inputs are focusable when created; custom generic views still need
`HostPushChildView`.

Example: [`examples/pascal/tui/framed_dialog.fpas`](../../../../../examples/pascal/tui/framed_dialog.fpas)

### `Application.CloseModal(App)`

Pops the **topmost** modal frame. Empty stack: no-op.

On close the host:

1. reads the stored modal result, if any;
2. unregisters an **owned** dialog root subtree when the frame was created by `ShowDialog` or `ShowFramedDialog`;
3. restores the saved focused leaf when that view still exists;
4. otherwise re-activates the saved window root and moves focus into its subtree;
5. otherwise focuses the first eligible view in the remaining modal scope, or the next global focus candidate;
6. requests redraw for affected views.

Call `HostSetActiveModalResult` **before** `CloseModal` when the dialog outcome matters to application logic.

## Modal results

`Application.HostSetActiveModalResult(App, ResultCode)` validates and stores the result for the active frame:

| `ResultCode` | Meaning |
| --- | --- |
| `1` | Accept — default confirmation |
| `2` | Cancel — dismiss without accepting |
| `>= 1000` | Application-defined command result |

Other codes, or calls with no active modal frame, are runtime errors.

The host does not invoke Pascal automatically on close. Application code reads its own state updated in `OnCommand` or button handlers that call `HostSetActiveModalResult` then `CloseModal`.

Constants used in examples:

```pascal
const
  ModalResultAccept: integer := 1;
  ModalResultCancel: integer := 2;
```

Reserve custom dialog outcomes at **`1000` and above** so they do not collide with Accept/Cancel.

## Enter, Escape, and modal-local commands

Default (Enter) and cancel (Escape) actions are regular modal-local command bindings. No separate
public helper is needed for the current API: applications give OK/Cancel buttons command ids, bind
Enter/Escape with **`HostBindCommandToActiveModal`**, and handle both button clicks and shortcuts in
the same **`OnCommand`** routine, as in `show_dialog.fpas` and `Ide.Dialog`:

```pascal
Application.HostBindCommandToActiveModal(App, EnterKey, CmdDialogOk);
Application.HostBindCommandToActiveModal(App, EscapeKey, CmdDialogCancel);

procedure OnCommand(App: Application; CommandId: integer);
begin
  if CommandId = CmdDialogOk then
  begin
    Application.HostSetActiveModalResult(App, ModalResultAccept);
    Application.CloseModal(App)
  end
  else if CommandId = CmdDialogCancel then
  begin
    Application.HostSetActiveModalResult(App, ModalResultCancel);
    Application.CloseModal(App)
  end
end;
```

Bind modal shortcuts **after** `ShowDialog` (or `ShowModal`) so they attach to the active frame. Re-bind when opening a nested dialog; bindings disappear when that frame is closed.

## Modal scope rules

While a modal frame is active and has one or more scoped views:

| Input | Behavior |
| --- | --- |
| Tab / Shift+Tab | Traversal limited to views inside the modal scope |
| Mouse | Events outside scoped rectangles are suppressed |
| Keys / commands | View-local and global bindings are blocked when focus is outside the modal scope; `HostBindCommandToActiveModal` bindings always dispatch |
| Menu bar | Only menu bars inside the active modal scope receive keyboard/mouse routing |

`Application.HostAttachViewToActiveModal(App, ViewId)` adds views outside the root subtree to the scope (for example a shared menu bar that must stay reachable).

## Low-level stack API

`Application.HostEnterModal(App, ModalId)` pushes a modal id **without** attaching a root view or changing scope geometry. Use only when the app manages scope manually through attached views and paint handlers.

`Application.HostLeaveModal(App)` pops one frame with the same restore semantics as `CloseModal` but is intended for manual stack management. Prefer `ShowModal` / `ShowDialog` + `CloseModal` in application code.

## Testing

Headless coverage:

| Path | Topic |
| ---- | ----- |
| [`tui_show_dialog_test.fpas`](../../../../../tests/tui/modals/tui_show_dialog_test.fpas) | `ShowDialog`, modal Escape via `HostBindCommandToActiveModal`, `HostSetActiveModalResult`, owned-root cleanup |
| [`tui_framed_dialog_default_cancel_test.fpas`](../../../../../tests/tui/modals/tui_framed_dialog_default_cancel_test.fpas) | `ShowFramedDialog`, Enter as Accept, Escape as Cancel, and owned-root cleanup |

See [Native testing](testing.md) for pump and assertion patterns.

## Implementation (contributors)

| Concern | Location |
| --- | ---- |
| Modal stack and return context | [`fpas-std/src/tui/modal/`](../../../../../crates/fpas-std/src/tui/modal/) |
| VM show/close bridge | [`fpas-vm/.../views/modal.rs`](../../../../../crates/fpas-vm/src/vm/execute/io/tui/views/modal.rs) |
| Native dialog controls | [`fpas-std/src/tui/widget/control/`](../../../../../crates/fpas-std/src/tui/widget/control/) |

Frame geometry, painted chrome, desktop work area, active-window activation, and owned framed
dialogs are exposed through the APIs documented in [Frame roots](frames.md).

## See also

- [Views and focus](views.md)
- [VM bridge](vm-bridge.md)
- [Handlers](handlers.md)
- [Hosted dispatch overview](README.md)
