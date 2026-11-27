# Container options

Use `Layout.Flow` with `Elements.Row`, `Elements.Column` and `Stacks.Stack`.
Pass the same record in `Grids.Options.Flow` to configure a grid.
All four containers use native CUI layout.

## Placement and overflow

| Option | Default | Behavior |
| --- | --- | --- |
| `Align` | `Alignment.Default` | Align children on the cross axis. Rows center children vertically. Columns stretch children horizontally. Intrinsic controls keep their natural size. |
| `Justify` | `Justification.Start` | Distribute free space on the main axis with `Start`, `Center`, `End`, `Between` or `Around`. |
| `Wrap` | `false` | Start a new line when a row runs out of width. |
| `Clip` | `null` | Enable or disable clipping of child paint at the container bounds. |
| `Scrolling` | `ScrollMode.None` | Use `Vertical` to scroll up and down. Use `Both` to add horizontal scrolling. |
| `Id` | `null` | Name the container in the frame snapshot. CUI also uses this ID to retain its native scroll position. |

With `Clip := null`, columns and non-wrapping rows clip their children.
Wrapping rows, layered stacks and grids let child paint extend past their bounds.
A container with scrolling always clips to its viewport.
CUI enables both axes when the native box sets horizontal scrolling.
Constrain the container on the scroll axis so the contents can overflow.
Supply a stable ID to preserve the native position as the view changes.
CUI generates an ID for an unnamed scrolling container.
Use [Scroll view](scroll-view.md) to store the position in the model or reveal
a named child.

In a layered stack, `Align` places children horizontally and `Justify` places
them vertically. See [Layered stack](stack.md) for the supported placements.
The grid positions children at cell origins. Keep `Align` and `Justify` at their
defaults in the grid itself.

Use [sizing](../../layout/sizing.md) to set native width and height.
Each axis accepts its own bounds and growth weight.
Set `Gap` and the `Padding` fields to control [spacing](spacing.md).
Set `ReadingDirection` to override the [inherited direction](reading-direction.md).
Set `Scope` to add a [focus scope](focus-scopes.md).

## Paint

| Option | Default | Behavior |
| --- | --- | --- |
| `Background / BackgroundTheme` | `null` | Choose a straight-alpha RGBA fill or a semantic theme color. Supply at most one. |
| `Border` | `null` | Set the stroke width in logical units. Choose `Color` or `ThemeColor`. CUI uses the theme border color if both are null. |
| `CornerRadius` | `0.0F` | Set a uniform radius in logical units. |
| `Corners` | `null` | Override the physical corner radii. A null corner uses `CornerRadius`. A zero corner stays square. |
| `Elevation` | `Elevation.None` | Choose a native theme shadow with `Resting`, `Raised` or `Floating`. |
| `Opacity` | `1.0F` | Set the paint opacity of the container and its descendants. Accepts finite values from zero through one. |

`new Layout.Border()` draws a stroke one logical unit wide.
The GUI host requires finite, nonnegative stroke widths and radii.
Corner positions stay physical when the reading direction changes.
Use `Elements.Border(Child, Title)` for a framed section with an optional
caption. In GUI, this creates a vertical container around the child.

## Disabled subtrees

Wrap any graphical element with `Elements.DisabledScope of TMsg(Child, Disabled)`.
The default is true. The child keeps its layout and uses native disabled input.
A false flag inherits the enclosing state, so it cannot enable a control
inside a disabled scope. Adjacent elements keep their own input state.
The scope also applies to children created by cached subviews and mapped views.

## See also

- [CUI API coverage](../../../cui-api-coverage.md#application-capabilities)
- [Spacing and padding](spacing.md)
- [Reading direction](reading-direction.md)
- [Focus scopes](focus-scopes.md)
- [Layout inspection](layout-inspection.md)
- [Shared sizing options](../../layout/sizing.md)
- [GUI layout](README.md)

## Source files

- [Layout.fpas](../../../../src/Fpas.Ui/Layout.fpas)
- [Elements.fpas](../../../../src/Fpas.Ui/Elements.fpas)
- [GUI layout values and paint](../../../../src/Fpas.Ui.Gui/Internal/Layout/Sizing.fpas)
- [Container validation](../../../../src/Fpas.Ui/Internal/Validation/Core/ContainerRules.fpas)
- [Container presentation](../../../../src/Fpas.Ui.Gui/Internal/Presentation/Presentation.fpas)
