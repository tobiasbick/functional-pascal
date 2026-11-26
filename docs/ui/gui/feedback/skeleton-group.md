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
public function View(): E.Element<string>;
begin
  return W.SkeletonGroup<string>('loading-dither', [
    S.Skeleton<string>('loading-title', L.Extent.Percent(60.0F), 8.0F),
    S.Skeleton<string>('loading-line', Height := 8.0F)
  ], new W.Options(Gap := 10.0F, Dither := true));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Children` | Placeholder elements in layout order. The gallery nests rows inside the group. |
| `Width / Gap` | Fill / 0, which uses the native gap. |
| `Dither` | false. Enable it for the shared ordered-dot sweep. |

## Interaction

Place the group where the eventual content will appear. CUI shares an animation across the skeletons inside it. Replace the whole group when the data arrives.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Feedback/SkeletonGroups.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Feedback/Widgets/Loading.fpas)
- [Skeleton](skeleton.md)
- [Feedback](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Feedback/`
  - [SkeletonGroup.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Feedback/SkeletonGroup.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Feedback/`
  - [NativeMethods.SkeletonGroup.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Feedback/NativeMethods.SkeletonGroup.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Feedback/`
  - [CuiWindow.SkeletonGroup.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Feedback/CuiWindow.SkeletonGroup.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/feedback/`
  - [skeleton_group.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/feedback/skeleton_group.c)
  - [skeleton_group.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/feedback/skeleton_group.h)
