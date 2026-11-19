# Markdown

Use Markdown to display formatted notes with links and task lists. The
gallery can replay its release notes as a stream. Available in **GUI**.

![Interactive Markdown release notes in the FPAS gallery](images/markdown.png)

## Example

```pascal
unit Docs.Ui.Code.Markdown;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Display.Markdowns as W;

// Renders release notes or a partial update to them.
// ## Parameters
// - `Text`: Markdown source received so far.
// - `Streaming`: Whether the document is still arriving.
// ## Returns
// Markdown with link and task events as messages.
public function View(Text: string; Streaming: bool): E.Element<W.Event>;
begin
  return W.Markdown<W.Event>('notes', Text, function(Change: W.Event): W.Event => Change,
    new W.Options(Streaming := Streaming));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Width / Size` | Fill / 0. Zero uses the theme body size. |
| `Compact / Streaming` | Both false. Compact reduces heading sizes and spacing. Set Streaming while supplying incomplete text. |

## Interaction

`Event.Link` reports the activated URL. `Event.Hover` reports a URL or null
when the pointer leaves the link. `Event.Task` reports the new checked state,
a one-based line and a UTF-8 byte offset. Convert that offset before indexing
a CLR string by UTF-16 position, then render the updated Markdown source.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Display/Markdowns.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Code/Tools.fpas)
- [Gallery update function](../../../../examples/ui/gui/gallery/Pages/Code/Update.fpas)
- [Code](README.md)
