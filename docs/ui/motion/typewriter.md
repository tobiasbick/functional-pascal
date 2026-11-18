# Typewriter

The gallery types a line and offers Replay. The AI page also uses the
typewriter for its sample answer.
Available in **GUI**.

![Typewriter text in the FPAS gallery](images/typewriter.png)

## Example

```pascal
unit Docs.Ui.Motion.Typewriter;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Motion.Typewriters as W, Fpas.Ui.Widgets.Motion.Motion as M;

// Reveals the current answer text.
// ## Parameters
// - `Answer`: Text received so far.
// - `Revision`: Replay revision.
// ## Returns
// The typewriter with a completion message.
public function View(Answer: string; Revision: int): E.Element<string>;
begin
  return W.Typewriter<string>('answer', Answer, Revision,
    new M.TextOptions(Size := 15.0F, Caret := true), function(): string => 'answer-visible');
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Replay / OnComplete` | 0 / null. Change Replay to restart. Supply a callback for completion. |
| `Speed / Delay / Caret` | 0 / 0 / false. Positive Speed sets characters per second. Delay is in seconds. |
| `Font / Size / Color / NoWrap` | Body / 0 / Automatic / false. |

## Interaction

Append to the current string to let typing continue from its previous
position. Replace the text or change Replay to start again. Completion
applies to the supplied text. Reduced motion reveals it immediately.

`OnComplete` runs when the current text first becomes fully visible for
the current replay revision. A change to the text or revision allows
another notification, including when the text returns to an earlier value.
Removing the widget clears its completion state. The next application
frame can report a completion reached during cached resize painting.

## See also

- [Widget API and types](../../../src/Fpas.Ui/Widgets/Motion/Typewriters.fpas)
- [Gallery example](../../../examples/ui/gui/gallery/Pages/Motion/View.fpas)
- [Text reveal](text-reveal.md)
- [AI](../ai/README.md)
- [Motion](README.md)
