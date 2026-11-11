# Fpas.Ui.Gui

FPAS desktop adapter for CUI. Applications use [Fpas.Ui.Mvu](../Fpas.Ui.Mvu/README.md)
transitions and [Fpas.Ui](../Fpas.Ui/README.md) declarative views. Start with the
[GUI gallery](../../examples/ui/gui/gallery/README.md).

## Application lifecycle

Use Run for the owner loop or Start with using const for explicit Step calls.
Create, step, inspect and dispose sessions on their creating UI thread. Post
admits typed messages from workers and reports Queued, Full or Closed.
Options select title, logical width/height, renderer, mailbox capacity and theme.

Update and View run on the owner. Shared MVU commands start/cancel background
work, request focus and exit. Close stops admission, cancels work and keeps
pumping the frozen view until cleanup finishes. Dispose joins cleanup. Step,
Snapshot and disposal must not be reentered from callbacks.

## Widgets and input

The [shared reference](../Fpas.Ui/README.md) owns options and interaction contracts:

- Buttons, icon buttons and connected button groups.
- Copy, hold, confirmation, status and split buttons.
- Theme switcher and sun/moon toggle with model-owned palette updates.
- Segmented controls with labels/icons, native sizes and full-width allocation.
- Dropdown menus with native triggers, command states, hints, labels and separators.
- Context menus attached to existing target components, using the same menu entries.
- Sidebar navigation and edge sheets/drawers.
- Text, separators, rows/columns, sizing, borders, semantic styles and scrolling
  as the foundations needed to compose these widgets.

Visible labels and stable IDs are separate, including literal ## text. Disabled
and covered controls accept no input or focus. Tab/Shift+Tab traverse enabled
controls within the active modal. Native CUI supplies rendering and interaction;
the host maps proposals to messages without applying application updates itself.

Prepare validates the active view before drawing. GUI distances use logical
units; terminal-specific elements and cell distances are rejected. View records,
arrays and callbacks remain unchanged while retained by the host.

## Invalidation and inspection

Transitions and size changes rebuild the view. Ordinary repaints reuse its
prepared tree and never restart effects. Typed caches advance after successful
presentation. Width classes are Compact below 600, Regular below 1200, then
Expanded; height thresholds are 400 and 900 logical units. Include relevant
theme dependencies in cached subviews.

Platform live-resize callbacks draw the cached tree without Update, View,
commands or input proposals. Callback state stays rooted until detachment;
managed failures surface after the native pump returns.

Snapshot returns the last successful frame: sequence, logical size, text, theme
and detached control descriptors with ID, role, item key, value, enabled state,
focus, hover and bounds. Reading before the first presentation throws.
IsSoftware reports the actual renderer, including fallback.
CapturePng saves the presented frame outside Step on the owner thread. It needs
a live window, a successful presentation and an existing destination directory.

## Build and verification

See [development](../../docs/build/development.md), the
[native prerequisites](../Fpas.Ui.Gui.Cui/native/README.md#prerequisites) and the
[binding verification](../Fpas.Ui.Gui.Cui/README.md#source-build-and-verification).
The GUI gallery contract exercises both renderers and all five publish modes,
including single-file and NativeAOT. Linux window tests require a display.

Only widgets checked in the [CUI inventory](../../docs/future/cui-gaps.md) are
implemented. Future widgets are added through that inventory.
