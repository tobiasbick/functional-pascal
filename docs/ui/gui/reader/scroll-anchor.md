# Scroll anchor

The Reader screenshot shows the article after activating Install in the
outline. The Install marker sits before that heading inside the scroll view.
Available in **GUI**.

![Reader showing Install after an outline jump](images/scroll-anchor.png)

## Example

```pascal
unit Docs.Ui.Reader.ScrollAnchor;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Layout as L;
uses Fpas.Ui.Widgets.Navigation.ScrollAnchors as W;

// Builds the viewport targeted by the outline example.
// ## Returns
// A scroll view with three anchored sections.
public function View(): E.Element of string;
begin
  return E.Size of string(
    E.ScrollView of string('article', E.Column of string(
      new L.Flow(Gap := L.Spacing.Normal),
      W.ScrollAnchor of string('intro'), E.Text of string('Introduction'),
      E.Gap of string(L.Measure.Logical(180)),
      W.ScrollAnchor of string('install'), E.Text of string('Installation'),
      E.Gap of string(L.Measure.Logical(180)),
      W.ScrollAnchor of string('cmake'), E.Text of string('With CMake'),
      E.Gap of string(L.Measure.Logical(180))
    )),
    new L.Flex(Height := new L.AxisSize(Basis := L.Extent.Fixed(L.Measure.Logical(160))))
  );
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Id` | A nonempty, unique key that matches the corresponding outline entry. |

## Usage

Insert the marker immediately before the target content in an uncontrolled
scroll view. The marker has zero height. Give the scroll view a finite
height so the outline can move through its content.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Navigation/ScrollAnchors.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Reader/View.fpas)
- [Table of contents](table-of-contents.md)
- [Scroll view](../layout/scroll-view.md)
- [Chapter index](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Navigation/`
  - [ScrollAnchor.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Navigation/ScrollAnchor.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Navigation/`
  - [NativeMethods.ScrollAnchor.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Navigation/NativeMethods.ScrollAnchor.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Navigation/`
  - [CuiWindow.ScrollAnchor.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Navigation/CuiWindow.ScrollAnchor.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/navigation/`
  - [scroll_anchor.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/navigation/scroll_anchor.c)
  - [scroll_anchor.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/navigation/scroll_anchor.h)
