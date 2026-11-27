# Timeline item

Use timeline items in an activity feed to show an event, its time and optional details. Available in **GUI**.

![Timeline item in the FPAS gallery](images/timeline-item.png)

## Example

```pascal
unit Docs.Ui.Feedback.TimelineItem;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Styles as S;
uses Fpas.Ui.Widgets.Planning.TimelineItems as W;

// Builds the timeline item example.
// ## Returns
// The widget for the application view.
public function View(): E.Element of string;
begin
  return W.TimelineItem of string('deployed', 'Deployed to production',
    new W.Options(Time := '2m ago', Description := 'Release 1.0.3 is live.',
      Tone := S.Tone.Success, Last := true));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Title` | A nonempty literal heading. |
| `Time` | Empty omits the caption. Format timestamps before passing the caption. |
| `Description` | Empty omits the supporting text. CUI wraps the description to the available width. |
| `Icon` | Empty uses CUI's dot. Pass a glyph from `IconGlyphs` to replace it. |
| `Tone` | Neutral. The tone sets the icon and its surrounding fill and border. |
| `Last` | false. Set true on the final item to omit its connector and trailing space. |

## Interaction

Place items in the order the reader should see them. CUI wraps the text and aligns the title and time. A timeline item has no activation action.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_timeline_item)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Planning/TimelineItems.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Feedback/View.fpas)
- [Feedback](README.md)

## Source files

- [TimelineItem.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Planning/TimelineItem.fpas)
- [CuiTimelineTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Display/CuiTimelineTests.cs)
- [timeline.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/timeline.c)
