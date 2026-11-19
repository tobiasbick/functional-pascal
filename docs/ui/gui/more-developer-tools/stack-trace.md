# Stack trace

Use a stack trace to inspect where an exception occurred. The gallery
shows a JavaScript error with source around the failing line.
Available in **GUI**.

![Stack frames and highlighted source in the FPAS gallery](images/stack-trace.png)

## Example

```pascal
unit Docs.Ui.MoreDeveloperTools.StackTrace;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Developer.StackTraces as W;

// Displays a recorded runtime error.
// ## Parameters
// - `Frames`: Call stack in display order.
// - `Message`: Exception message from the application.
// ## Returns
// The trace panel with clicked frames as messages.
public function View(Frames: array of W.Frame; Message: string): E.Element<W.Event>;
begin
  return W.StackTrace<W.Event>('error-trace', Frames,
    function(Change: W.Event): W.Event => Change,
    new W.Options(Title := 'Runtime error', Message := Message, Language := 'js'));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Frame.File / Line / Column` | File can be null. Lines and columns are one-based. Zero omits a location component. |
| `Frame.External / Source / SourceFirst` | External groups library frames. Source is an array of lines starting at SourceFirst. |
| `Title / Message / Language` | All empty. Language controls source highlighting. |
| `Width / Enabled` | Fill / true. |

## Interaction

Toggle library groups or frames with source. The copy button copies the
trace as text. `Event.Clicked` identifies a frame in the input array or
returns minus one. Use that index to open the source in your application.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Developer/StackTraces.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/MoreDeveloper/View.fpas)
- [More developer tools](README.md)
