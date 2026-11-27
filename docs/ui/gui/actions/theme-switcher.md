# Theme switcher

Select Light, Dark or an application-provided System appearance. Available in **GUI**.

![Theme switcher in the FPAS gallery](images/theme-switcher.png)

## Example

```pascal
unit Docs.Ui.Actions.ThemeSwitcher;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Theming.ThemeSwitchers as T;
uses Fpas.Ui.Widgets.Theming.Themes as Palette;

// Builds the example view.
// ## Parameters
// - `Selected`: Current application value.
// ## Returns
// The widget with its message type.
public function View(Selected: T.Selection): E.Element of T.Selection;
begin
  return T.ThemeSwitcher of T.Selection('theme', Selected,
    function(Choice: T.Selection): T.Selection => Choice, new T.Options(WithSystem := true));
end function;

// Resolves the selected palette for the host's ThemeFor callback.
// ## Parameters
// - `Selected`: The appearance selected in the model.
// - `SystemAppearance`: The concrete light or dark system appearance.
// - `Accent`: The current accent.
// ## Returns
// The theme drawn by the host.
public function ThemeFor(
  Selected: T.Selection; SystemAppearance: Palette.Mode; Accent: Palette.Accent
): Palette.Theme;
begin
  return if T.Resolve(Selected, SystemAppearance) = Palette.Mode.Dark then
    Palette.Dark(Accent)
  else
    Palette.Light(Accent)
  end if;
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Id` | Nonempty stable group ID. |
| `Selected` | Light or Dark. System requires `WithSystem`. The model supplies the current selection. |
| `OnChange` | Maps a proposed choice to a message. |
| `WithSystem` | False. True adds the System choice with a monitor icon. |
| `Enabled` | True. False disables focus and input for all segments. |

## Interaction

Tab moves focus without selecting. Left and Right focus and propose the
adjacent choice. Click, Enter or Space proposes a different selection.
Use `ItemId` to address a segment in a focus request.

Apply the accepted choice through `ThemeFor`. For System, pass the concrete
appearance from the application provider to `Resolve`.

The FPAS gallery shares its model-owned selection with the sidebar and toggle.
The CUI gallery stores the switcher's selection independently. Changing the
palette with the CUI toggle leaves the CUI switcher's selected segment in place.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_theme_switcher)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Theming/ThemeSwitchers.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Actions/Widgets/ThemeSwitcher.fpas)
- [Gallery theme callback](../../../../examples/ui/gui/gallery/App/Update.fpas)
- [Sidebar appearance controls](../navigation/sidebar.md#appearance-controls)
- [Actions](README.md)

## Source files

- [ThemeSwitcher.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Theming/ThemeSwitcher.fpas)
- [CuiWindow.ThemeSwitcher.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Theming/CuiWindow.ThemeSwitcher.cs)
- [CuiThemeSwitcherTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Actions/CuiThemeSwitcherTests.cs)
