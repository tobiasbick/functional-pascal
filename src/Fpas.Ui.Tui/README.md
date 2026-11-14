# Fpas.Ui.Tui

Terminal adapter for the shared [MVU runtime](../Fpas.Ui.Mvu/README.md) and
[declarative widgets](../Fpas.Ui/README.md). It depends on managed terminal
services and does not load CUI or require a graphical display.

## Application lifecycle

Run owns the interactive terminal loop. StartHeadless provides explicit Send,
Step, Snapshot and Close operations. Update and View run on the owner;
background work posts typed messages through shared MVU commands. Close cancels
work and joins cleanup. Do not reenter owner operations from callbacks.
The interactive host restores terminal modes on normal exit and failure.

## Views, layout and validation

Views use the shared Element tree. Supported widgets are buttons, icon buttons,
button groups, segmented controls, dropdown menus, copy/hold/confirm/status/split buttons, theme selectors/toggles,
sidebar navigation and sheets. Text, rows/columns, sizing, borders, styles,
cached subviews and scrolling provide their composition foundations.

Preflight validates the view before rendering. Logical distances map to cells:
8 units per column and 16 per row. Measure.Cells explicitly requests cell
distances. Content/fill allocation, weighted growth, shrinkage, size bounds and
cross-axis alignment follow the shared CUI layout subset.

The host retains prepared subviews, allocation data and unchanged painted rows.
An equivalent frame keeps its presentation sequence while installing current
input bindings. Resize resolves the new viewport without invalidating unrelated
cached subviews.

## Input and controlled state

Tab/Shift+Tab traverse enabled controls in the active scope. Enter and typed
Space activate the focused action. Theme segments use Left/Right without
wrapping. Split menus support arrows, Enter, Escape, wheel and page keys.
The model owns accepted state changes; rendering never calls change mappers.

Dropdown and split buttons share the session's popup foundation. Menu sections
and separators are not focus stops; disabled commands reject input. TUI opens
on the first enabled command and skips disabled rows during navigation.
Icons, display-only shortcut hints, check marks and danger colors have cell
equivalents. Preferred placement and logical width fit to the viewport;
an icon-only trigger keeps its caption when no terminal glyph is supplied.

Pointer input uses cell coordinates. Disabled and covered controls reject input.
Disabling cancels pending pointer capture. Hold-to-confirm requires a continuous
pointer press and release over its control; terminal input supplies no key-release
events. Copy uses OSC 52 when the terminal permits clipboard writes.

Sheets dock to any edge with a close button, dimmed backdrop and scrolling body.
Escape or a backdrop click proposes dismissal; closing restores available focus.

## Terminal extensions

- Elements.CellStyled and Terminal.Style supply optional RGB overrides.
- TerminalExtensions.Views.Viewport resolves a view from exact columns/rows.
- TerminalExtensions.Views.Screen and typed Input shortcuts bind application keys.
- Application.Input/Frame support controlled input injection and cell snapshots.

Snapshots expose visible control IDs, roles, values, bounds, focus and hover,
plus cell text and colors. The gallery's headless-control mode supports the
[private terminal debugger](../../docs/pascal/tools/debugging.md#private-terminal-and-tui-control).

## Themes and examples

Use Fpas.Ui.Widgets.Theming.Themes.Light/Dark and the same ThemeFor callback as GUI. Terminal
styles derive from the shared palette, with alpha composited over cell
backgrounds. Include relevant theme values in cached-subview dependencies.
See [shared themes](../Fpas.Ui/README.md#shared-themes) and the
[TUI gallery](../../examples/ui/tui/gallery/README.md).

Only widgets checked in the [CUI inventory](../../docs/future/cui-gaps.md) and
selected for terminal use are implemented.
