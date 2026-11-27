# Skeleton group

Group placeholders that should share a loading animation. Available in **GUI**.

![Skeleton group in the FPAS gallery](images/skeleton-group.png)

## Example

```pascal
unit Docs.Ui.Feedback.SkeletonGroup;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Layout as L;
uses Fpas.Ui.Widgets.Feedback.SkeletonGroups as W;
uses Fpas.Ui.Widgets.Feedback.Skeletons as S;

// Builds the skeleton group example.
// ## Returns
// The widget for the application view.
public function View(): E.Element of string;
begin
  return W.SkeletonGroup of string('loading-dither', [
    S.Skeleton of string('loading-title', L.Extent.Percent(60.0F), 8.0F),
    S.Skeleton of string('loading-line', Height := 8.0F)
  ], new W.Options(Gap := 10.0F, Dither := true));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Children` | Placeholder elements in layout order. The gallery nests rows inside the group. |
| `Width` | Fill. Accepts a fixed logical width or a percentage from 0 through 100. |
| `Gap` | 0 uses the native default of 8 logical units. Supply a finite nonnegative gap. |
| `Dither` | false. Enable it for the shared ordered-dot sweep. |

## Interaction

The skeletons in a group share one shine that sweeps across the group. A nested group owns the shine for its own skeletons. Apply `Elements.Size` with a positive `Grow` to weight the width in a row.

With `Dither`, CUI paints ordered dots and updates the sweep 20 times per second. Reduced motion holds the dots still. Replace the group with the loaded content when the model is ready.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_skeleton_group)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Feedback/SkeletonGroups.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Feedback/Widgets/Loading.fpas)
- [Skeleton](skeleton.md)
- [Feedback](README.md)

## Source files

- [Skeleton geometry tests](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Layout/CuiSkeletonTests.cs)
- `src/Fpas.Ui.Gui/Internal/Widgets/Feedback/`
  - [SkeletonGroup.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Feedback/SkeletonGroup.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Feedback/`
  - [NativeMethods.SkeletonGroup.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Feedback/NativeMethods.SkeletonGroup.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Feedback/`
  - [CuiWindow.SkeletonGroup.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Feedback/CuiWindow.SkeletonGroup.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/feedback/`
  - [skeleton_group.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/feedback/skeleton_group.c)
  - [skeleton_group.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/feedback/skeleton_group.h)
