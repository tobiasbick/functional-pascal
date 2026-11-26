# Divider

The Notifications sheet separates preference switches from recent activity
with a divider. Available in **GUI**.

![Divider above Recent in the FPAS gallery](images/divider.png)

## Example

```pascal
unit Docs.Ui.Layout.Divider;

uses Fpas.Ui.Elements as E;

// Separates notification preferences from recent activity.
// ## Parameters
// - `Preferences`: The controls above the divider.
// - `Recent`: The view below the divider.
// ## Returns
// The two sections with a divider between them.
public function View(Preferences: E.Element of string; Recent: E.Element of string): E.Element of string;
begin
  return E.Column of string(Preferences, E.Divider of string(), Recent);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Title` | Empty. Pass a string to add a caption. |

## Usage

Place the divider between adjacent groups. A plain divider is vertical in a
row and horizontal elsewhere. CUI draws the line one logical unit thick in
the theme's border color. In a row or column, the line spans the available
space on the cross axis. Pass a title to place a caption above a horizontal
line.

The Notifications example adds its Recent heading below the line.

## See also

- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Overlays/Widgets/Notifications.fpas)
- [Spacing](spacing.md)
- [Sheet and drawer](../overlays/sheet.md)
- [GUI layout](README.md)

## Source files

- [Elements.fpas](../../../../src/Fpas.Ui/Elements.fpas)
- [Divider presentation](../../../../src/Fpas.Ui.Gui/Internal/Presentation/Presentation.fpas)
