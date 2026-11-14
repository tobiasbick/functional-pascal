# Avatar

Use an avatar to identify a person with initials or an image. Available in **GUI**.

![Avatar in the FPAS gallery](images/avatar.png)

## Example

```pascal
unit Docs.Ui.Disclosure.Avatar;

uses Fpas.Ui.Elements as E,
  Fpas.Ui.Widgets.Media.Avatars as W;

// Builds the avatar example.
// ## Returns
// The widget for the application view.
public function View(): E.Element<string>;
begin
  return W.Avatar<string>('jonas-avatar', 'Jonas Berg', new W.Options(Size := 44.0F, Status := 'online'));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Name` | Used for the initials and control label. |
| `Size / Square` | 32 logical points / false. Square uses a rounded-square frame. |
| `ImagePath` | null. Supply a model-owned path for a profile image. |
| `Status` | Empty hides the dot. Use online, busy or away for colored presence. |

## Interaction

The avatar is a display element. Use a link or another interactive control when the user needs to open a profile. The gallery places the avatar in a hover card.

## See also

- [Widget API and types](../../../src/Fpas.Ui/Widgets/Media/Avatars.fpas)
- [Gallery example](../../../examples/ui/gui/gallery/Pages/Disclosure/Widgets/HoverCard.fpas)
- [Hover card](hover-card.md)
- [Disclosure](README.md)
