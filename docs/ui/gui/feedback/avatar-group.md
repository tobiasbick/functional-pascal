# Avatar group

Use an avatar group to show several people in a compact space. The gallery shows four initials and a +3 overflow count. Available in **GUI**.

![Avatar group in the FPAS gallery](images/avatar-group.png)

## Example

```pascal
unit Docs.Ui.Feedback.AvatarGroup;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Media.AvatarGroups as W;

// Builds the avatar group example.
// ## Parameters
// - `Names`: Names in display order.
// ## Returns
// The widget for the application view.
public function View(Names: array of string): E.Element<string>;
begin
  return W.AvatarGroup<string>('feedback-team', Names, new W.Options(MaximumVisible := 4));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Names` | At least one name, in display order. |
| `MaximumVisible / Size` | 4 / 32 logical points. The visible limit excludes the overflow count. |

## Interaction

CUI draws the visible initials as overlapping avatars and shows the number of remaining people. Pass the new ordered list when membership changes.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Media/AvatarGroups.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Feedback/Widgets/BadgesAndAvatars.fpas)
- [Avatar](../disclosure/avatar.md)
- [Feedback](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Media/`
  - [AvatarGroup.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Media/AvatarGroup.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Media/`
  - [NativeMethods.AvatarGroup.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Media/NativeMethods.AvatarGroup.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Media/`
  - [CuiWindow.AvatarGroup.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Media/CuiWindow.AvatarGroup.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/media/`
  - [avatar_group.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/media/avatar_group.c)
  - [avatar_group.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/media/avatar_group.h)
