# Layered stack

Layout shows a blue badge over the bottom-right corner of a placeholder.
Available in **GUI**.

![Badge drawn over a placeholder in the FPAS gallery](images/stack.png)

## Example

```pascal
unit Docs.Ui.Layout.Stack;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Layout as L;
uses Fpas.Ui.Widgets.Containers.Stacks as W;
uses Fpas.Ui.Widgets.Feedback.Badges as Badge;
uses Fpas.Ui.Widgets.Feedback.Skeletons as Placeholder;

// Layers a badge over a placeholder.
// ## Returns
// A size-constrained stack with the badge painted last.
public function View(): E.Element of string;
begin
  return E.Size of string(
    W.Stack of string(
      new L.Flow(Align := L.Alignment.End, Justify := L.Justification.End,
        Padding := L.Spacing.Fixed(L.Measure.Logical(12))),
      Placeholder.Skeleton of string('preview-placeholder', Height := 140.0F),
      Badge.Badge of string('preview-badge', 'On top')
    ),
    new L.Flex(
      Width := new L.AxisSize(Basis := L.Extent.Fill, Maximum := L.Measure.Logical(420)),
      Height := new L.AxisSize(Basis := L.Extent.Fixed(L.Measure.Logical(160)))
    )
  );
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Settings` | Pass a `Layout.Flow` record. `Align` places children horizontally. `Justify` places children vertically. |
| `Children` | Specify children in paint order. The final child paints on top. |

## Usage

CUI defaults to the start edge horizontally and the top edge vertically.
Use `Align` to place or stretch children horizontally.
Use `Justify` with `Start`, `Center` or `End` to place children vertically.
Children use their content height by default. Set the height of a child
explicitly to stretch that child vertically.
Use `Elements.Size` on the stack to set its own dimensions.

## See also

- [CUI API coverage](../../../cui-api-coverage.md#application-capabilities)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Containers/Stacks.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Layout/View.fpas)
- [Container options](container-options.md)
- [Reading direction](reading-direction.md)
- [GUI layout](README.md)

## Source files

- [Layout.fpas](../../../../src/Fpas.Ui/Layout.fpas)
- [Stack presentation](../../../../src/Fpas.Ui.Gui/Internal/Presentation/Presentation.fpas)
- [CuiLayoutOptionsTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Layout/CuiLayoutOptionsTests.cs)
