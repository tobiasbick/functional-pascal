# Accordion

Use an accordion header to show or hide a section of related content. Available in **GUI**.

![Accordion in the FPAS gallery](images/accordion.png)

## Example

```pascal
unit Docs.Ui.DataDisplay.Accordion;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Containers.Accordions as W;

// Builds the accordion example.
// ## Parameters
// - `Open`: Expansion flag from the application model.
// ## Returns
// The widget for the application view.
public function View(Open: bool): E.Element of bool;
begin
  return W.Accordion of bool('reduced-motion', 'Does it respect reduced motion?', Open,
    function(Expanded: bool): bool => Expanded,
    E.Text of bool('Every animation has a reduced motion path.'));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Id` | Nonempty stable ID. Keep it when the title changes. |
| `Title` | Nonempty literal header text. |
| `Open` | The current model flag. |
| `OnChange` | Maps the proposed expansion flag to a message. |
| `Content` | The body displayed under the header. CUI can draw it during a closing animation. |
| `Enabled` | true. Use false to disable input for the header and body. |

## Interaction

Click the header or focus it and press Enter or Space to propose a new flag.
Apply the proposal in the update function and pass the new flag to `Open`.
The closing body cannot receive input.

Compose adjacent accordion items in one [card](../feedback/card.md) or
[column](../layout/column.md). CUI draws a divider before each accordion item
that follows a sibling. Give each header its own stable ID. For exclusive
expansion, close the other items in the update function.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_accordion)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Containers/Accordions.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Lists/View.fpas)
- [GUI fixture](../../../../tests/Fixtures/gui-host/DisclosureContract.fpas)
- [Native pixel comparison](../../../../src/Fpas.Ui.Gui.Cui/native/tests/disclosure.c)
- [Expandable card](../disclosure/expandable-card.md)
- [Data display](README.md)

## Source files

- [Disclosure.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Containers/Disclosure.fpas)
