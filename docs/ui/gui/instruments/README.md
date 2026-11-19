# Instruments

The Instruments page models caffeine levels over a day. The event track edits
cup times. The chart, readouts and dot-matrix display present values calculated
from those times. These widgets require the GUI host.

| Widget | When to use it |
| --- | --- |
| [Readout](readout.md) | Show a labeled value, unit and context. |
| [Dot-matrix chart](dot-chart.md) | Plot stacked sample bands with native annotations. |
| [Event track](event-track.md) | Edit discrete events on a numeric or time-of-day axis. |
| [Dot-matrix display](dot-display.md) | Render a short status or a scrolling line in a 5×7 dot font. |

To align a chart with an event track, give them the same width and X range.
The gallery lets the event track draw the shared axis.

## See also

- [UI handbook](../README.md)
- [Charts](../charts/README.md)
- [Instruments in the gallery](../../../../examples/ui/gui/gallery/Pages/Instruments/View.fpas)

## Source files

- [State.fpas](../../../../examples/ui/gui/gallery/Pages/Instruments/State.fpas)
- [code_instruments.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/code_instruments.c)
