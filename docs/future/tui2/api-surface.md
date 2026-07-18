# Std.Tui2 API surface

This document inventories the remaining planned surface and is not a frozen specification.
Implemented symbols are documented in the current
[`Std.Tui2` reference](../../pascal/std/tui2/README.md).

## Type categories

Std.Tui2 distinguishes immutable or copyable values from identities for live application state.

### Value records

| Type | Purpose |
| --- | --- |
| `TuiPaintContext` | Local bounds, clip, state, and palette for one paint callback. |
| `TuiMeasureSpec` | Bounded or unbounded measurement constraints per axis. |
| `TuiMeasureResult` | Minimum, preferred, and maximum sizes returned by measurement. |
| `TuiLayoutFit` | Minimum, available, and per-axis overflow extents for a container. |
| `TuiKey` | A normalized key and modifiers when a TUI-specific form is needed. |
| `TuiEvent` | An event routed through the TUI. |
| `TuiChangeOrigin` | `User` or `Programmatic` origin for a typed value change. |
| `TuiMenuItem` | A menu entry description. |
| `TuiStatusItem` | A status-line entry description. |
| `TuiSizePolicyKind` | One resizing policy for one axis. |
| `TuiSizePolicy` | Independent horizontal and vertical resizing policies. |
| `TuiAlignment` | Alignment inside an allocated layout rectangle. |
| `TuiMargins` | Outer layout margins in cells. |
| `TuiLayoutItem` | A view, nested layout, or spacer entry. |
| `TuiSpacer` | Fixed or expanding empty layout space. |
| `TuiLayoutDirection` | Horizontal or vertical layout main axis. |

These records describe values only. They do not own live views or contain copies of mutable widget state.

### Handler types

| Type | Purpose |
| --- | --- |
| `TuiTextChangedHandler` | Observe a text value change. |
| `TuiCheckedChangedHandler` | Observe a boolean checked-state change. |
| `TuiListSelectionChangedHandler` | Observe a list selection change. |
| `TuiRadioSelectionChangedHandler` | Observe a radio selection change. |
| `TuiKeyHandler` | Handle routed or fallback key input. |
| `TuiMouseHandler` | Handle routed or fallback mouse input. |
| `TuiAttachHandler` | Observe attachment to a live parent. |
| `TuiDetachHandler` | Observe removal from a live parent. |
| `TuiMeasureHandler` | Calculate custom-view size information. |
| `TuiResizeHandler` | Observe resolved bounds changing. |
| `TuiPaintHandler` | Draw a custom view into a transient canvas. |
| `TuiFocusHandler` | Observe focus acquisition. |
| `TuiBlurHandler` | Observe focus loss. |
| `TuiCloseRequestHandler` | Allow or reject closing. |
| `TuiClosedHandler` | Observe completed closing. |

### Live handles

| Type | Purpose |
| --- | --- |
| `TuiContainer` | Common identity for a view that can own children and a layout. |
| `TuiCustomView` | Application-defined view implemented through lifecycle hooks. |
| `TuiDesktop` | Root container for windows and application chrome. |
| `TuiWindow` | Movable or framed top-level content. |
| `TuiDialog` | Modal or modeless dialog content. |
| `TuiMenuBar` | Application menu bar. |
| `TuiStatusLine` | Application status and shortcut line. |
| `TuiLabel` | Static or associated text. |
| `TuiInputLine` | Single-line text input. |
| `TuiListBox` | Selectable item list. |
| `TuiScrollBar` | Scroll position and range control. |
| `TuiCheckBox` | Boolean option control. |
| `TuiRadioGroup` | Mutually exclusive option control. |
| `TuiMemo` | Multi-line editable text. |
| `TuiTextViewer` | Scrollable read-only text. |
| `TuiScrollView` | Explicit viewport for content larger than its allocated rectangle. |
| `TuiLayout` | Common identity for a live layout. |
| `TuiHorizontalLayout` | Horizontal box layout. |
| `TuiVerticalLayout` | Vertical box layout. |
| `TuiGridLayout` | Row and column layout. |
| `TuiFormLayout` | Label and field layout. |
| `TuiStackedLayout` | Shared-area layout with one visible item. |

Each live handle is a small opaque record backed by an internal registry identity. Handles do not expose Rust ownership or implementation objects.

### Transient handles

| Type | Purpose |
| --- | --- |
| `TuiCanvas` | Clipped drawing target valid only during one `OnPaint` callback. |

Transient handles cannot be stored for later use. Operations reject them after their callback returns.

## Core operations

The following signatures illustrate ownership and naming. They are expected to evolve.

### Application

```pascal
TuiApplication.Open(): TuiApplication
TuiApplication.Run(App: TuiApplication)
TuiApplication.Quit(App: TuiApplication)
TuiApplication.Invalidate(App: TuiApplication)
App.Desktop: TuiDesktop                          { read-only property }
TuiApplication.Post(App: TuiApplication; Handler: procedure()): boolean
```

The implemented headless application additionally exposes `Size`, `ResizeForTest`,
`RunIterations`, and `Quit`. `RunIterations` performs posted-callback, desktop-layout, and tick
phases with a deterministic iteration budget; the terminal-backed `Run` above remains the intended
interactive entry point.

### Common views

```pascal
View.Bounds: TuiRect                             { read-write property }
View.Visible: boolean                            { read-write property }
View.Enabled: boolean                            { read-write property }
View.Tag: integer                                { read-write property }
TuiView.Show(View: TuiView)
TuiView.Hide(View: TuiView)
TuiView.Enable(View: TuiView)
TuiView.Disable(View: TuiView)
TuiView.Focus(View: TuiView)
TuiView.Invalidate(View: TuiView)
TuiView.Destroy(View: TuiView)
TuiView.Measure(View: TuiView; Spec: TuiMeasureSpec): TuiMeasureResult

TuiContainer.Add(Container: TuiContainer; View: TuiView)
TuiContainer.Remove(Container: TuiContainer; View: TuiView)
Container.Layout: TuiLayout                      { read-write property }
```

Typed view handles use explicit `AsView` operations so containers can accept different controls without discarding type-specific operations. Layout handles use matching `AsLayout` operations. Downcasts are not public.

Views expose minimum, preferred, and maximum sizes plus independent horizontal and vertical size policies. Containers may attach nested layouts that assign the resolved bounds. See [layout.md](layout.md).

### Layouts

```pascal
TuiHorizontalLayout.Create(App: TuiApplication): TuiHorizontalLayout
TuiVerticalLayout.Create(App: TuiApplication): TuiVerticalLayout
TuiGridLayout.Create(App: TuiApplication): TuiGridLayout
TuiFormLayout.Create(App: TuiApplication): TuiFormLayout
TuiStackedLayout.Create(App: TuiApplication): TuiStackedLayout
Horizontal.AsLayout(): TuiLayout
Vertical.AsLayout(): TuiLayout
Grid.AsLayout(): TuiLayout
Form.AsLayout(): TuiLayout
Stacked.AsLayout(): TuiLayout
TuiGridPlacement.Create(Row: integer; Column: integer; RowSpan: integer; ColumnSpan: integer): TuiGridPlacement
TuiGridItems.Add(Grid: TuiGridLayout; Item: TuiLayoutItem; Placement: TuiGridPlacement): boolean
TuiFormItems.AddRow(Form: TuiFormLayout; LabelView: TuiView; FieldView: TuiView): boolean
TuiStackedItems.Add(Stacked: TuiStackedLayout; Item: TuiLayoutItem): boolean
Stacked.CurrentIndex: integer                    { read-write property }
TuiLayoutSettings.SetMargins(Layout: TuiLayout; Margins: TuiMargins): boolean
TuiLayoutSettings.SetSpacing(Layout: TuiLayout; Spacing: integer): boolean
TuiLayoutMeasure.Measure(Layout: TuiLayout; Spec: TuiMeasureSpec): TuiMeasureResult
TuiLayoutArrange.Arrange(Layout: TuiLayout; Bounds: TuiRect): boolean
Container.NeedsLayout(): boolean
Container.PerformLayout(): boolean
Container.LayoutFit: TuiLayoutFit                 { read-only property }
Desktop.LayoutFit: TuiLayoutFit                   { read-only property }
TuiScrollView.Create(App: TuiApplication): TuiScrollView
TuiScrollView.AsView(ScrollView: TuiScrollView): TuiView
TuiScrollView.AsContainer(ScrollView: TuiScrollView): TuiContainer
ScrollView.Layout: option of TuiLayout             { read-write property }
ScrollView.Offset: TuiPoint                        { read-write property }
ScrollView.ViewportSize: TuiSize                   { read-only property }
ScrollView.ContentSize: TuiSize                    { read-only property }
ScrollView.MaximumOffset: TuiPoint                 { read-only property }
ScrollView.NeedsLayout(): boolean
ScrollView.PerformLayout(): boolean
```

Grid construction, form rows, stacked pages, recursive measurement/allocation, and explicit
container invalidation passes are implemented. `LayoutFit` detects terminal-too-small state without
compressing minimum geometry. `TuiScrollView` exposes oversized preferred content through a clamped
two-axis offset and lays it out at a negative local origin. Control-specific measurement remains
planned.

### Containers and top-level views

```pascal
TuiDesktop.Add(Desktop: TuiDesktop; View: TuiView)

TuiWindow.Create(App: TuiApplication; Title: string): TuiWindow
TuiWindow.Add(Window: TuiWindow; View: TuiView)

TuiDialog.Create(App: TuiApplication; Title: string): TuiDialog
TuiDialog.Add(Dialog: TuiDialog; View: TuiView)
TuiDialog.Execute(Dialog: TuiDialog): TuiCommand
```

### Basic controls

```pascal
TuiLabel.Create(App: TuiApplication; Text: string): TuiLabel
LabelView.Text: string                            { read-write property }

TuiInputLine.Create(App: TuiApplication; MaxLength: integer): TuiInputLine
Input.Text: string                                { read-write property }
```

### Selection and text controls

```pascal
TuiListBox.Create(App: TuiApplication): TuiListBox
List.Items: array of string                       { read-write property }
List.Selected: integer                            { read-write property }

TuiCheckBox.Create(App: TuiApplication; Text: string): TuiCheckBox
CheckBox.Checked: boolean                         { read-write property }

TuiRadioGroup.Create(App: TuiApplication; Items: array of string): TuiRadioGroup
Group.Selected: integer                           { read-write property }

TuiMemo.Create(App: TuiApplication): TuiMemo
Memo.Text: string                                 { read-write property }

TuiTextViewer.Create(App: TuiApplication; Text: string): TuiTextViewer
Viewer.Text: string                               { read-write property }
```

The property notation above is the planned public syntax. Accessors remain ordinary focused FPAS
methods that validate handles and update the registry. Imperative operations such as `Focus`,
`Invalidate`, `Destroy`, `Add`, `Remove`, and `Execute` remain functions or procedures.

## Actions and handlers

`TuiCommand` is a distinct public type even if its initial runtime representation is an integer.

The remaining action work binds menus, status items, and shortcuts to the existing `TuiAction`.
Controls with values expose one typed event per semantic change.

Raw key and mouse handlers return `boolean` to participate in input propagation. Action handlers and typed change notifications do not return `boolean`.

`TuiAttachHandler`, `TuiDetachHandler`, and `TuiResizeHandler` are implemented on `TuiCustomView`.
The resize handler receives both old and new bounds and is delivered after layout. The remaining
custom-view handler types in this inventory are still planned.

The core does not provide general multicast publish/subscribe. See
[events-and-actions.md](events-and-actions.md) for the full contract.

Application state and exact initial event shapes are defined in
[events-and-actions.md](events-and-actions.md).

## View lifecycle

Applications use `OnStart`, `OnStop`, and optional `OnTick` boundaries. Views have attach, detach, measure, resize, paint, focus, blur, close-request, and closed phases.

Built-in controls implement these phases internally. `TuiCustomView` exposes one typed handler for each supported hook. See [view-lifecycle.md](view-lifecycle.md) for timing and mutation rules.

## Lifetime direction

- `TuiApplication` owns the live registry and desktop.
- A container owns the child handles attached to it.
- Closing the application invalidates every handle from that application.
- Modal execution does not create a second terminal session.
- Mutable widget state lives in the registry and is read through typed operations.

Removal destroys the removed subtree, reparenting is initially unsupported, and explicit destruction follows [handles-and-ownership.md](handles-and-ownership.md).

## Later extensions

The following topics are not prerequisites for the first usable controls:

- menu and status item construction;
- custom layout callbacks;
- checked action groups.
