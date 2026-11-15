# Trace viewer

Use a trace viewer for nested time spans across threads or services.
The gallery shows several lanes and a measured interval.
Available in **GUI**.

![Trace lanes and time measurement in the FPAS gallery](images/trace-viewer.png)

## Example

```pascal
unit Docs.Ui.DeveloperTools.TraceViewer;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Developer.TraceViewers as W;

// Shows the current trace.
// ## Parameters
// - `Tracks`: Lanes and their collapsed state.
// - `Spans`: Recorded spans in track, depth and start order.
// - `Markers`: Named timestamps crossing all lanes.
// - `Viewport`: Time range, vertical offset and measurement.
// - `Selected`: Selected span index, or minus one.
// ## Returns
// The viewer with selection, lane and viewport proposals.
public function View(
  Tracks: array of W.Track;
  Spans: array of W.Span;
  Markers: array of W.Marker;
  Viewport: W.View;
  Selected: int
): E.Element<W.Event>;
begin
  return W.TraceViewer<W.Event>('trace', Tracks, Spans, Markers, Viewport, Selected,
    function(Change: W.Event): W.Event => Change, new W.Options(Height := 320.0F));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Span.Start / End / Track / Depth` | Times are in seconds. Lanes and depths are zero-based. Supply nonoverlapping spans at the same depth in a lane. |
| `View.Start / End / Scroll` | Pass the visible time range in seconds and the vertical offset in points. |
| `View.MarkStart / MarkEnd` | Pass the measured interval in seconds. |
| `Width / Height / RowHeight / Enabled` | Fill / 0 / 0 / true. Zero uses native dimensions. |

## Interaction

Zoom with the wheel, pan by dragging and hold Shift while dragging to
measure. Click a lane name to fold it. Save `Event.Viewport` and `Selection`.
When `TrackIndex` is nonnegative, also update the indicated lane's
`Collapsed` state.

## See also

- [Widget API and types](../../../src/Fpas.Ui/Widgets/Developer/TraceViewers.fpas)
- [Gallery example](../../../examples/ui/gui/gallery/Pages/Developer/View.fpas)
- [Flame graph](../editing-tools/flame-graph.md)
- [Developer tools](README.md)
