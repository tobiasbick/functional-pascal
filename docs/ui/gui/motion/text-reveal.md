# Text reveal

The gallery reveals a release-build message and replays it on demand.
Available in **GUI**.

![Revealed build completion message in the FPAS gallery](images/text-reveal.png)

## Example

```pascal
unit Docs.Ui.Motion.TextReveal;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Motion.Motion as M;
uses Fpas.Ui.Widgets.Motion.TextReveals as W;

// Reveals the build report.
// ## Parameters
// - `Summary`: Build completion text.
// - `Revision`: Replay revision.
// ## Returns
// The native text reveal.
public function View(Summary: string; Revision: int): E.Element<string>;
begin
  return W.TextReveal<string>('build-report', Summary, Revision,
    new M.TextOptions(Font := M.Font.Display, Size := 20.0F));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Replay / Delay` | Both zero. Change Replay to restart. Delay is in seconds. |
| `Font / Size / Color` | Body / 0 / Automatic. |
| `Width / NoWrap` | Content / false. Use NoWrap to keep the message on a single line. |

## Interaction

CUI reveals words when first shown or when the text changes. Change the
revision to replay the same message. Reduced motion shows the full text.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Motion/TextReveals.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Motion/View.fpas)
- [Typewriter](typewriter.md)
- [Motion](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Motion/`
  - [TextReveal.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Motion/TextReveal.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Motion/`
  - [NativeMethods.TextReveal.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Motion/NativeMethods.TextReveal.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Motion/`
  - [CuiWindow.TextReveal.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Motion/CuiWindow.TextReveal.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/motion/`
  - [text_reveal.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/motion/text_reveal.c)
  - [text_reveal.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/motion/text_reveal.h)
