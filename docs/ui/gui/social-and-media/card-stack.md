# Card stack

The gallery shows a destination deck. A completed swipe reports Saved
or Skipped and advances the model to the next card.
Available in **GUI**.

![Top destination card and stacked cards in the FPAS gallery](images/card-stack.png)

## Example

```pascal
unit Docs.Ui.SocialAndMedia.CardStack;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Containers.CardStacks as W;

// Shows a deck with the top card first.
// ## Parameters
// - `Cards`: Composed card views, front to back.
// ## Returns
// The deck with swipe directions as messages.
public function View(Cards: array of E.Element of W.Direction): E.Element of W.Direction;
begin
  return W.CardStack of W.Direction('destinations', Cards,
    function(Swipe: W.Direction): W.Direction => Swipe, new W.Options(Height := 200.0F));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Width` | Content. CUI prefers 320 logical units and can shrink the deck to fit. Use Fill, Fixed, or Percent to set the width. |
| `Height` | Zero uses 320 logical units for each card. Use a finite nonnegative height to set the card height. |
| `Visible` | Zero shows up to three cards. A positive count includes the top card. |
| `Enabled` | True. Allows dismissing the top card and interacting with its contents. |

## Interaction

Build the cards from front to back. The deck reserves 12 logical units of
additional height for each visible position below the top card.
Only the top card accepts input. An empty deck has no focus target.

Flick the top card to dismiss it. Tab moves focus to the top card and shows
its focus ring. Press Left or Right to dismiss it in that physical direction.
During a pointer drag, the pointer owns the swipe until release.

Handle the reported direction and remove or rotate the top card in the
model. The message arrives once the dismissed card leaves the deck.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_card_stack_begin)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Containers/CardStacks.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Social/Media.fpas)
- [Social and media](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Containers/`
  - [CardStack.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Containers/CardStack.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Containers/`
  - [NativeMethods.CardStack.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Containers/NativeMethods.CardStack.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Containers/`
  - [CuiWindow.CardStack.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Containers/CuiWindow.CardStack.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/containers/`
  - [card_stack.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/containers/card_stack.c)
  - [card_stack.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/containers/card_stack.h)
