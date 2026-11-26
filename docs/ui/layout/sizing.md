# Sizing and units

Wrap a child in `Elements.Size(Child, Settings)` to apply a `Layout.Flex` rule. In a row, the main-axis rule controls width. In a column, the rule controls height.

## Measures

| `Layout.Measure` | Capability profile | Meaning |
| --- | --- | --- |
| `Logical(Value)` | Portable, Terminal, Graphical | A whole number of logical units. |
| `Cells(Value)` | Terminal | A whole number of terminal cells. |
| `Points(Value)` | Graphical | Logical units with fractional precision. |

Distances must be finite and nonnegative. The TUI host rounds logical distances up. One column maps to eight logical units and one row maps to 16.

## Extents and bounds

| `Flex.Basis` | Allocation |
| --- | --- |
| `Extent.Content` | Use the child's preferred size. A GUI widget retains its own size hint. A positive `Grow` requests a share of free space. |
| `Extent.Fill` | Share free space with other growing children. |
| `Extent.Fixed(Measure)` | Use the specified distance subject to bounds. The child keeps its allocated size when space runs out. |
| `Extent.Percent(Value)` | Use a percentage of the parent's inner extent. The GUI host accepts values from zero through 100. |

`Minimum` and `Maximum` bound the allocated size. The minimum must not exceed the maximum. Use logical units for both bounds or cells for both bounds.

## Growth and shrinkage

Growing children divide the space that remains after padding, gaps and other children. The host allocates that space in proportion to `Grow`. When a child reaches a bound, the host redistributes the remaining space.

`Grow` must be finite and nonnegative. For `Fill`, the GUI host uses weight one when `Grow` is zero. The TUI host raises fill weights below one to one. Set `Grow` to zero for fixed and percentage extents.

When space runs out, the host shrinks flexible children toward their minimum sizes. Fixed sizes and minimums can leave overflow. The TUI host rounds shared boundaries to allocate whole cells.

## Independent GUI axes

Pass `Flex.Width` or `Flex.Height` as an `AxisSize` record to size that axis. Each axis has its own basis, bounds and growth weight. An omitted axis keeps its size and bounds. Leave the main-axis `Flex` fields at their defaults when you set either independent axis.

## Example

```pascal
unit Docs.Ui.Layout.Sizing;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Layout as L;

// Places a fixed-width sidebar beside a growing body.
// ## Parameters
// - `Sidebar`: The left pane in reading order.
// - `Body`: The main pane.
// ## Returns
// A row that uses a portable size rule for each pane.
public function View(Sidebar: E.Element of string; Body: E.Element of string): E.Element of string;
begin
  return E.Row of string(
    new L.Flow(Gap := L.Spacing.Normal),
    E.Size of string(Sidebar, new L.Flex(Basis := L.Extent.Fixed(L.Measure.Logical(192)))),
    E.Size of string(Body, new L.Flex(Basis := L.Extent.Fill))
  );
end function;

end unit;
```

## See also

- [UI layout](README.md)
- [Responsive views](responsive.md)
- [GUI rows](../gui/layout/row.md)
- [TUI layout](../tui/layout/README.md)

## Source files

- [Layout.fpas](../../../src/Fpas.Ui/Layout.fpas)
- [Elements.fpas](../../../src/Fpas.Ui/Elements.fpas)
- [LayoutRules.fpas](../../../src/Fpas.Ui/Internal/Validation/Core/LayoutRules.fpas)
- [GUI sizing](../../../src/Fpas.Ui.Gui/Internal/Layout/Sizing.fpas)
- [TUI allocation](../../../src/Fpas.Ui.Tui/Internal/Layout/Flexible.fpas)
