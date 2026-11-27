# Expandable card

Use an expandable card for a summary with details that can open beneath its header. Available in **GUI** and [TUI](../../tui/disclosure/expandable-card.md).

![Expandable card in the FPAS gallery](images/expandable-card.png)

## Example

```pascal
unit Docs.Ui.Disclosure.ExpandableCard;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Containers.ExpandableCards as W;

// Builds the expandable card example.
// ## Parameters
// - `Expanded`: Current expansion state.
// ## Returns
// The widget for the application view.
public function View(Expanded: bool): E.Element of bool;
begin
  return W.ExpandableCard of bool('billing', 'Billing', Expanded, function(Value: bool): bool => Value,
    E.Text of bool('7 of 10 seats in use'),
    new W.Options(Description := 'Pro plan, renews on December 1', Badge := 'Active'));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Id` | Nonempty stable header ID. |
| `Title` | Nonempty literal header caption. |
| `Open` | Pass the current expansion flag from the model. |
| `OnChange` | Maps the proposed expansion flag to a message. |
| `Content` | The body under the header. |
| `Description` | Empty. Pass supporting text to show it under the title. |
| `Icon` | Empty. Pass a CUI glyph to show it before the title in the GUI. |
| `Badge` | Empty. Pass a caption to show a small badge before the chevron. |
| `BadgeTone` | Neutral. Selects the semantic color of the badge. |
| `Enabled` | `true`. Setting `false` disables input for the header and body. |

## Interaction

Click the header or activate it with Enter or Space to propose the opposite
expansion flag. Apply the proposal in the update function and pass the new
flag to `Open`. CUI draws the exit animation while the collapsing body stops
accepting input.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_expandable_card)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Containers/ExpandableCards.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Disclosure/View.fpas)
- [Managed behavior tests](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Overlays/CuiDisclosureTests.cs)
- [Native pixel comparison](../../../../src/Fpas.Ui.Gui.Cui/native/tests/disclosure.c)
- [Expandable text](expandable-text.md)
- [Disclosure](README.md)

## Source files

- [Shared Security example](../../../../examples/ui/gui/gallery/Shared/SecurityCard.fpas)
