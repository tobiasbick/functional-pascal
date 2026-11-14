using Fpas.Ui.Gui.Cui.Widgets.Charts;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Focused native visualization binding, ownership and activation contracts.</summary>
public sealed class CuiVisualizationTests
{
    /// <summary>Keeps all caller data valid through deferred paint after inputs and temporary buffers change.</summary>
    /// <param name="renderer">Native renderer.</param>
    /// <param name="kind">Visualization index.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software, 0)]
    [InlineData(CuiRenderer.Software, 1)]
    [InlineData(CuiRenderer.Software, 2)]
    [InlineData(CuiRenderer.Software, 3)]
    [InlineData(CuiRenderer.Software, 4)]
    [InlineData(CuiRenderer.Software, 5)]
    [InlineData(CuiRenderer.Software, 6)]
    [InlineData(CuiRenderer.OpenGl, 0)]
    [InlineData(CuiRenderer.OpenGl, 1)]
    [InlineData(CuiRenderer.OpenGl, 2)]
    [InlineData(CuiRenderer.OpenGl, 3)]
    [InlineData(CuiRenderer.OpenGl, 4)]
    [InlineData(CuiRenderer.OpenGl, 5)]
    [InlineData(CuiRenderer.OpenGl, 6)]
    public void VisualizationsCopyDataBeforePainting(CuiRenderer renderer, int kind)
    {
        using var window = new CuiWindow("Visualization ownership", 640, 620, renderer, continuous: true);
        window.SetReducedMotion(true);
        string first = Path.Combine(Path.GetTempPath(), Guid.NewGuid() + ".png");
        string second = Path.Combine(Path.GetTempPath(), Guid.NewGuid() + ".png");
        try
        {
            for (int frame = 0; frame < 5; frame++) Draw(false);
            window.CapturePng(first); Draw(true); window.CapturePng(second);
            Assert.Equal(File.ReadAllBytes(first), File.ReadAllBytes(second));
            var bounds = window.GetControlRectangle("widget");
            Assert.True(bounds.Width > 0 && bounds.Height > 0);
        }
        finally { File.Delete(first); File.Delete(second); }

        void Draw(bool mutate)
        {
            Assert.True(window.NextFrame()); window.MovePointer(-1, -1); window.BeginColumn();
            float[] values = [1, 2, 3, 4, 5, 6];
            string[] rows = ["Monday", "Tuesday"], columns = ["Jan", "", "Mar"];
            CuiVizItem[] items = [new("Layout ### literal", 55), new("Text 日本語", 42), new("Draw", 35)];
            CuiChartSeries[] series = [new("1.0", values), new("1.1", [2, 4, 6, 3, 5, 1], new(4))];
            string[] axes = ["Speed", "Size", "Polish", "Docs", "Platforms", "Tests"];
            switch (kind)
            {
                case 0: window.Heatmap("widget", values, 2, 3, new(rows, columns, Format: new(0, Suffix: " req"))); break;
                case 1: window.CalendarHeatmap("widget", new(2026, 1, 5), values, new(Unit: "contributions", Format: new(0))); break;
                case 2: window.Treemap("widget", items, new(Format: new(0, Suffix: " KB"))); break;
                case 3: window.RadarChart("widget", series, axes, new(Maximum: 10, Format: new(0))); break;
                case 4: window.Funnel("widget", items, new(Format: new(0))); break;
                case 5: window.SplitBar("widget", items, new(Format: new(0, Suffix: " ms"))); break;
                case 6: window.Waffle("widget", items, new(Format: new(0))); break;
            }
            if (mutate) { Array.Fill(values, 999); Array.Fill(rows, "Changed"); Array.Fill(columns, "Changed");
                Array.Fill(axes, "Changed"); Array.Clear(items); Array.Clear(series); }
            // Reuse and overwrite the allocator's freed call-scoped buffers before native painting.
            var blocks = new List<nint>();
            try
            {
                for (int bytes = 8; bytes <= 256; bytes += 8)
                    for (int count = 0; count < 4; count++)
                    {
                        nint block = Marshal.AllocHGlobal(bytes); blocks.Add(block);
                        Marshal.Copy(Enumerable.Repeat((byte)'Z', bytes).ToArray(), 0, block, bytes);
                    }
                window.EndColumn(); GC.Collect(); window.Present();
            }
            finally { foreach (nint block in blocks) Marshal.FreeHGlobal(block); }
        }
    }

    /// <summary>Reports the original day index even when native CUI drops older weeks.</summary>
    /// <param name="renderer">Native renderer.</param>
    /// <param name="width">Window width.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software, 280)]
    [InlineData(CuiRenderer.OpenGl, 280)]
    [InlineData(CuiRenderer.Software, 900)]
    [InlineData(CuiRenderer.OpenGl, 900)]
    public void CalendarClicksResolveToOriginalDates(CuiRenderer renderer, int width)
    {
        using var window = new CuiWindow("Calendar clicks", width, 420, renderer, continuous: true);
        window.SetReducedMotion(true);
        for (int i = 0; i < 5; i++) Draw(-1, -1);
        var bounds = window.GetControlRectangle("calendar");
        int fit = Math.Max(1, (int)MathF.Floor((bounds.Width - 32 + 3) / 14));
        int expected = Math.Max(0, (52 - fit) * 7);
        float x = bounds.X + 34, y = bounds.Y + 22;
        Draw(x, y, true);
        Assert.Equal(expected, Draw(x, y, false));
        Draw(x, y, true, false);
        Assert.Equal(-1, Draw(x, y, false, false));

        int Draw(float x, float y, bool? down = null, bool enabled = true)
        {
            Assert.True(window.NextFrame()); window.MovePointer(x, y);
            if (down is { } pressed) window.SetLeftButton(pressed);
            window.BeginColumn();
            int clicked = window.CalendarHeatmap("calendar", new(2026, 1, 5),
                Enumerable.Repeat(1f, 364).ToArray(), new(Unit: "contributions"), enabled);
            window.EndColumn(); window.Present(); return clicked;
        }
    }

    /// <summary>Keeps treemap selection model-owned, toggles it natively and rejects disabled activation.</summary>
    /// <param name="renderer">Native renderer.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void TreemapSelectionUsesNativeClickProposals(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Treemap selection", 600, 420, renderer, continuous: true);
        window.SetReducedMotion(true);
        int model = -1;
        for (int i = 0; i < 4; i++) Draw(-1, -1);
        var r = window.GetControlRectangle("tree");
        float x = r.X + 12, y = r.Y + 12;
        Draw(x, y, true);
        var proposal = Draw(x, y, false);
        Assert.Equal(0, proposal.Activated); Assert.Equal(0, proposal.Selected);
        Assert.Equal(-1, Draw(-1, -1).Selected); // A rejected proposal does not replace the model.
        model = proposal.Selected;
        Draw(x, y, true);
        Assert.Equal(-1, Draw(x, y, false).Selected);
        Draw(x, y, true, false);
        Assert.Equal(-1, Draw(x, y, false, false).Activated);
        window.ForgetControl("tree");
        Assert.Equal(model, Draw(-1, -1).Selected);

        CuiTreemapResult Draw(float mx, float my, bool? down = null, bool enabled = true)
        {
            Assert.True(window.NextFrame()); window.MovePointer(mx, my);
            if (down is { } pressed) window.SetLeftButton(pressed);
            window.BeginColumn();
            var result = window.Treemap("tree", [new("Layout", 55), new("Text", 42), new("Draw", 35)],
                new(Height: 240), model, enabled);
            window.EndColumn(); window.Present(); return result;
        }
    }

    /// <summary>Rejects malformed dimensions, dates, values and scales before native mutation.</summary>
    [GuiFact]
    public void InvalidVisualizationDataIsRejected()
    {
        using var window = new CuiWindow("Visualization validation", 640, 620, CuiRenderer.Software);
        Assert.True(window.NextFrame()); window.BeginColumn();
        Assert.Throws<ArgumentOutOfRangeException>(() => window.Heatmap("bad", [1, 2], 2, 2, new()));
        Assert.Throws<ArgumentException>(() => window.Heatmap("bad", [1, 2], 1, 2, new(["A", "B"])));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.Heatmap("bad", [float.NaN], 1, 1, new()));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.Heatmap("bad", [1e10f, 1e10f], 1, 2, new()));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.CalendarHeatmap("bad", DateOnly.MaxValue, [1, 2], new()));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.Treemap("bad", [new("A", -1)], new()));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.Treemap("bad", [new("A", 1)], new(), 1));
        Assert.Throws<ArgumentException>(() => window.RadarChart("bad", [new("A", [1, 2])], ["X", "Y", "Z"], new()));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.RadarChart("bad", [], ["X", "Y"], new()));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.Funnel("bad", [new("A", float.PositiveInfinity)], new()));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.SplitBar("bad", [new("A", 1)], new(Height: -1)));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.Waffle("bad", [new("A", 1)], new(Columns: 61)));
        window.Heatmap("heat", [], 0, 0, new());
        window.CalendarHeatmap("calendar", new(2026, 1, 5), [], new());
        window.Treemap("tree", [], new(Height: 30));
        window.Funnel("funnel", [], new());
        window.SplitBar("split", [], new(Legend: false));
        window.Waffle("waffle", [], new(Cell: 4));
        window.EndColumn(); window.Present();
    }
}
