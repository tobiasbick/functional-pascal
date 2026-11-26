# Terminal

Use Terminal for an ANSI output panel with command input. The gallery
implements demo commands for help, colors and progress. Available in **GUI**.

![Terminal showing help and ANSI colors in the FPAS gallery](images/terminal.png)

## Example

```pascal
unit Docs.Ui.Files.Terminal;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Developer.Terminals as W;

// Builds a terminal for commands handled by the application.
// ## Parameters
// - `Output`: Cumulative ANSI output.
// - `Revision`: Current scrollback reset revision.
// ## Returns
// The terminal with submitted commands and interrupts as messages.
public function View(Output: string; Revision: int): E.Element<W.Event>;
begin
  return W.Terminal<W.Event>('commands', Output, Revision,
    function(Input: W.Event): W.Event => Input,
    new W.Options(Height := 300.0F, Prompt := '> '));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Width / Height / Size` | Fill / 0 / 0. Zero uses native height or font size. A height of minus one fills the viewport. |
| `Prompt / Input` | `'$ '` / true. Set Input to false for an output panel. |
| `Output / Revision` | Append to cumulative output for live scrollback. Change Revision to clear the scrollback and replay the supplied output. |

## Interaction

CUI echoes submitted lines and handles Up and Down command history, pointer
selection and Ctrl+L. A submitted line sends `Event.Command`. Ctrl+C without
a selection sends `Event.Interrupt`. Handle those actions in the application
update and append any resulting output to the model.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Developer/Terminals.fpas)
- [Gallery widget](../../../../examples/ui/gui/gallery/Pages/Files/View.fpas)
- [Gallery command handling](../../../../examples/ui/gui/gallery/Pages/Files/Terminal.fpas)
- [Files](README.md)

## Source files

- `src/Fpas.Ui/Widgets/Developer/`
  - [Terminal.fpas](../../../../src/Fpas.Ui/Widgets/Developer/Terminal.fpas)
- `src/Fpas.Ui.Gui/Internal/Widgets/Developer/`
  - [Terminal.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Developer/Terminal.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Developer/`
  - [NativeMethods.Terminal.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Developer/NativeMethods.Terminal.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Developer/`
  - [CuiWindow.Terminal.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Developer/CuiWindow.Terminal.Types.cs)
  - [CuiWindow.Terminal.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Developer/CuiWindow.Terminal.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/developer/`
  - [terminal.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/developer/terminal.c)
  - [terminal.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/developer/terminal.h)
