# Skeleton

Use a skeleton to reserve the shape of content while it loads. Available in **GUI**.

![Skeleton in the FPAS gallery](images/skeleton.png)

## Example

```pascal
unit Docs.Ui.Feedback.Skeleton;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Layout as L;
uses Fpas.Ui.Widgets.Feedback.Skeletons as W;

// Builds the skeleton example.
// ## Returns
// The widget for the application view.
public function View(): E.Element of string;
begin
  return W.Skeleton of string('loading-title', L.Extent.Percent(70.0F), Height := 12.0F);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Width` | Omitted width fills the parent. Fixed logical widths and percentages from 0 through 100 are also valid. |
| `Height` | 0 uses the native default of 14 logical units. Supply a finite nonnegative height. |

## Interaction

CUI draws a passive placeholder with an animated shine. Reduced motion displays the static shape. Apply `Elements.Size` with a positive `Grow` to weight the width in a row. Replace the placeholder with the loaded content when the model is ready.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_skeleton)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Feedback/Skeletons.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Feedback/View.fpas)
- [Skeleton group](skeleton-group.md)
- [Feedback](README.md)

## Source files

- [Skeleton.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Feedback/Skeleton.fpas)
- [skeleton.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/skeleton.c)
- [SkeletonContract.fpas](../../../../tests/Fixtures/gui-host/SkeletonContract.fpas)
- [CuiSkeletonTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Layout/CuiSkeletonTests.cs)
