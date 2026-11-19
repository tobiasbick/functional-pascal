# Hover card

Use a hover card for a preview anchored to an interactive control. The gallery opens a profile preview over the @jonas link. Available in **GUI**.

![Hover card in the FPAS gallery](images/hover-card.png)

## Example

```pascal
unit Docs.Ui.Disclosure.HoverCard;

uses Fpas.Ui.Elements as E,
  Fpas.Ui.Widgets.Overlays.HoverCards as W,
  Fpas.Ui.Widgets.Display.Links as L;

// Builds the hover card example.
// ## Parameters
// - `Content`: Current value from the application model.
// ## Returns
// The widget for the application view.
public function View(Content: E.Element<string>): E.Element<string>;
begin
  return W.HoverCard<string>('jonas-card', L.Link<string>('jonas-link', '@jonas', 'open-profile'), Content);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Trigger` | Supply a single interactive control or a card with `Hoverable` enabled. |
| `Position / Width` | BottomStart / 300 logical points. Zero width uses the default. |
| `Delay` | 0.45 seconds. Zero uses the default. |
| `Enabled` | true. |

## Interaction

Rest the pointer on the link to open the card after the delay. It remains open while the pointer is over the trigger or the card. Escape dismisses it until the pointer leaves the trigger. The link still sends its own activation message.

Declare dialogs and sheets outside the hover card body. If the body has scroll
anchors, put them in a viewport inside that body.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Overlays/HoverCards.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Disclosure/Widgets/HoverCard.fpas)
- [Link](link.md)
- [Avatar](avatar.md)
- [Disclosure](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Overlays/`
  - [HoverCard.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Overlays/HoverCard.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Overlays/`
  - [NativeMethods.HoverCard.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Overlays/NativeMethods.HoverCard.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Overlays/`
  - [CuiWindow.HoverCard.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Overlays/CuiWindow.HoverCard.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/overlays/`
  - [hover_card.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/overlays/hover_card.c)
  - [hover_card.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/overlays/hover_card.h)
