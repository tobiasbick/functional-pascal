# Card

Use a card to place related content on a native surface. Available in **GUI**.

![Card in the FPAS gallery](images/card.png)

## Example

```pascal
unit Docs.Ui.Feedback.Card;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Containers.Cards as W;

// Builds the card example.
// ## Parameters
// - `Content`: Child view drawn by the card.
// ## Returns
// The widget for the application view.
public function View(Content: E.Element<string>): E.Element<string>;
begin
  return W.Card<string>('badges-and-avatars-card', [Content], new W.Options(Gap := 16.0F));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Width / Height` | Content / Content. |
| `MinimumWidth / MaximumWidth` | Both 0, which uses the native defaults. |
| `Padding / Gap` | 0 / 0 use native defaults. PaddingX and PaddingY can override the main inset. |
| `Horizontal / Align / Wrap` | false / Default / false. Wrap applies to a horizontal card. |
| `Muted / Selected / Enabled` | false / false / true. Muted uses a borderless surface, and Selected uses an accent border. |
| `OnActivate` | Optional callback that makes the card itself interactive. |
| `Hoverable` | false. Enable it to use a card without an activation callback as a hover card trigger. |

## Interaction

Without an activation callback the card groups its children. An enabled card with a callback can be focused and activated with the pointer or keyboard. Child controls retain their own actions.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Containers/Cards.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Feedback/Widgets/BadgesAndAvatars.fpas)
- [Grid](grid.md)
- [Expandable card](../disclosure/expandable-card.md)
- [Feedback](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Containers/`
  - [Card.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Containers/Card.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Containers/`
  - [NativeMethods.Card.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Containers/NativeMethods.Card.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Containers/`
  - [CuiWindow.Card.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Containers/CuiWindow.Card.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/containers/`
  - [card.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/containers/card.c)
  - [card.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/containers/card.h)
