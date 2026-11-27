# Banner

Use a banner for an announcement that appears in the page layout. Available in **GUI**.

![Banner in the FPAS gallery](images/banner.png)

## Example

```pascal
unit Docs.Ui.Disclosure.Banner;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Feedback.Banners as W;

// Builds the banner example.
// ## Parameters
// - `Open`: Current visibility from the application model.
// ## Returns
// The widget for the application view.
public function View(Open: bool): E.Element of string;
begin
  return W.Banner of string('release-banner', 'Version 1.1 is out.', Open,
    function(): string => 'dismiss-banner', function(): string => 'read-notes',
    new W.Options(ActionLabel := 'Read the notes'));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Tone` | `Accent`. Sets the color of the banner and its action. CUI treats `Neutral` as `Accent`. |
| `Icon` | Null omits the icon slot. A CUI glyph draws an icon and an empty string keeps the slot blank. |
| `ActionLabel` | Null omits the action. An empty caption shows the arrow. Pair the caption value with `OnAction`. |
| `OnDismiss` | Null. Supply a message factory to add a close button. |
| `Enabled` | True. Allows input on both buttons. |

## Interaction

Keep visibility in the model. When the close button sends its message, set
`Open` to false. The action sends its own message and leaves the banner open.
CUI places the action below the message in a narrow layout.
Keyboard focus draws an outline around the action.
Use `ActionId` and `DismissId` to identify the buttons.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_banner)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Feedback/Banners.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Disclosure/Widgets/Banner.fpas)
- [Disclosure](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Feedback/`
  - [Banner.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Feedback/Banner.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Feedback/`
  - [NativeMethods.Banner.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Feedback/NativeMethods.Banner.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Feedback/`
  - [CuiWindow.Banner.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Feedback/CuiWindow.Banner.Types.cs)
  - [CuiWindow.Banner.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Feedback/CuiWindow.Banner.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/feedback/`
  - [banner.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/feedback/banner.c)
  - [banner.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/feedback/banner.h)
