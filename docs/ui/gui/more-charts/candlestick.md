# Candlestick

Use a candlestick chart for ordered open, high, low and close values with optional volume. The gallery starts with 2,000 periods.

Available in **GUI**.

![Candlestick in the FPAS gallery](images/candlestick.png)

## Example

```pascal
unit Docs.Ui.MoreCharts.Candlestick;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Charts.Candlesticks as W;

// Builds the candlestick example.
// ## Parameters
// - `Candles`: OHLC periods in chronological order.
// ## Returns
// The widget for the application view.
public function View(Candles: array of W.Candle): E.Element<int>;
begin
  return W.Candlestick<int>('price', Candles, new W.Options(Height := 280.0F),
    function(Index: int): int => Index);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Candles / Labels` | Pass finite values with High and Low enclosing Open and Close. Volume is nonnegative and defaults to 0. Labels is empty or has one caption per period. |
| `Width / Height / Format` | Fill / 0 / null. A null price format uses two decimal places. |
| `Overlays` | Empty. Supply up to four line series. NaN in an overlay marks a gap. |
| `Volume / Ohlc / Enabled` | true / false / true. Enable Ohlc for bars instead of candle bodies. |
| `Up / Down` | Automatic / Automatic. |

## Interaction

Use the wheel to zoom around the pointer. Drag or use Shift+wheel to pan. With focus, Left and Right move the crosshair, Shift moves ten periods, and Home or End goes to an endpoint. Plus and minus zoom and zero fits all periods. The chart follows newly appended periods when its viewport is at the end. `OnCursorChange` can report the original period index or -1 when inspection clears.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Charts/Candlesticks.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/MoreCharts/Widgets/Market.fpas)
- [Line chart](../charts/line-chart.md)
- [More charts](README.md)
