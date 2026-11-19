// [Documentation](../../../docs/ui/gui/developer-tools/trace-viewer.md)
using Fpas.Ui.Gui.Cui.Widgets.Developer;

namespace Fpas.Ui.Gui.Cui.Tests.Display;

/// <summary>Checks native trace link hit testing, geometry and retained input.</summary>
public sealed class CuiTraceLinkTests
{
    /// <summary>Reads span rectangles and reports the original link index after caller inputs change.</summary>
    /// <param name="renderer">Native renderer.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void LinksAndSpanRectanglesUseNativeGeometry(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Trace links", 700, 360, renderer, continuous: true);
        window.SetReducedMotion(true);
        CuiDevSpan[] spans = [new("Send", 0, 1, 0, 0, 0), new("Nested", .1, .7, 0, 1, 0), new("Receive", 3, 4, 1, 0, 0)];
        Draw(); Draw();
        var first = window.TraceSpanRectangle("trace", spans[0]);
        var last = window.TraceSpanRectangle("trace", spans[2]);
        Assert.True(first.Width > 0 && first.Height > 0 && last.Width > 0);
        Assert.True(window.TraceSpanRectangle("trace", spans[1]).Width > 0);
        float x = (first.X + first.Width + last.X) / 2;
        float y = (first.Y + last.Y + first.Height) / 2;
        Assert.Equal(0, Draw(x, y).HoveredLink);
        Draw(collapsed: true);
        Assert.Equal(0, window.TraceSpanRectangle("trace", spans[1]).Width);
        Assert.Equal(0, window.TraceSpanRectangle("trace", spans[0] with { Start = 8, End = 9 }).Width);
        Assert.True(window.NextFrame()); window.Text("Removed"); window.Present();
        Assert.Equal(0, window.TraceSpanRectangle("trace", spans[0]).Width);

        CuiTraceResult Draw(float x = -1, float y = -1, bool collapsed = false)
        {
            Assert.True(window.NextFrame()); window.MovePointer(x, y); window.BeginColumn(8, 0);
            CuiTraceLink[] links = [new(0, 2, 8, 1.5f, true)];
            var result = window.TraceViewer("trace", [new("Main", collapsed), new("Worker", false)], spans, [],
                0, 5, 0, 0, 0, -1, 210, 26, 1, 0, true, links);
            window.TrackLastControl("trace");
            Array.Clear(links); GC.Collect();
            window.EndColumn(); window.Present();
            return result;
        }
    }

    /// <summary>Rejects invalid dependencies before CUI reads indices or dimensions.</summary>
    [GuiFact]
    public void InvalidLinksLeaveTheFrameUsable()
    {
        using var window = new CuiWindow("Trace validation", renderer: CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        foreach (var link in new CuiTraceLink[] { new(-1, 0), new(0, 1), new(0, 0, Width: float.NaN) })
            Assert.Throws<ArgumentException>(() => window.TraceViewer("trace", [new("Main", false)],
                [new("Span", 0, 1, 0, 0, 0)], [], 0, 2, 0, 0, 0, -1, 100, 0, 1, 0, true, [link]));
        window.Text("Usable"); window.Present();
    }
}
