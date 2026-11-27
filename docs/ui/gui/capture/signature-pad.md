# Signature pad

Use a signature pad to record handwritten strokes. The gallery uses a
520-unit-wide pad with a native Clear button. Available in **GUI**.

![Signature pad with a sample stroke in the FPAS gallery](images/signature-pad.png)

## Example

```pascal
unit Docs.Ui.Capture.SignaturePad;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Files.SignaturePads as W;

// Builds a pad that reports a snapshot of the ink.
// ## Returns
// The pad with each signature change as an application message.
public function View(): E.Element of W.Signature;
begin
  return W.SignaturePad of W.Signature('signature',
    function(Value: W.Signature): W.Signature => Value,
    new W.Options(Height := 180.0F));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Id` | Stable pad ID. Retain it while the pad holds ink. |
| `Width` | `Fill`. A fixed logical width or a percentage sets the native width. `Content` also fills the available width. |
| `Height` | Nonpositive uses 180 logical units. |
| `Placeholder` | Null uses `Sign here`. An empty string hides the instruction. |
| `Ink` | `Charts.Color.Automatic` uses the foreground. Choose a theme token or explicit RGBA with nonzero alpha to override it. |
| `Enabled` | true. Set to false to keep the ink and disable drawing and clearing. |
| `OnChange` | null. Supply a mapper to read the detached signature on changes. |

## Interaction

Draw with the mouse or a pen. CUI varies the width with stroke speed and
keeps the ink for the widget's stable ID. A narrower pad scales older ink
toward the signature line. Clear removes the ink. Use
`ClearId(Id)` to address the button while it is visible.

The change message contains `Points` in logical drawing coordinates and
`StrokeEnds` as exclusive indices into that array. Clear reports an empty
snapshot. Save the detached data in the model if you need it after the pad is
removed.

When drawing directly with `CuiWindow`, call `ReadSignature(Id)` to copy the
points on any frame. Call `ClearSignature(Id)` before drawing the pad to
remove the ink and cancel any stroke in progress. Use the same ID and drawing
scope for both calls.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_signature_pad)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Files/SignaturePads.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Capture/View.fpas)
- [Capture](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Files/`
  - [SignaturePad.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Files/SignaturePad.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Files/`
  - [NativeMethods.SignaturePad.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Files/NativeMethods.SignaturePad.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Files/`
  - [CuiWindow.SignaturePad.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Files/CuiWindow.SignaturePad.Types.cs)
  - [CuiWindow.SignaturePad.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Files/CuiWindow.SignaturePad.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/files/`
  - [signature_pad.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/files/signature_pad.c)
  - [signature_pad.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/files/signature_pad.h)
