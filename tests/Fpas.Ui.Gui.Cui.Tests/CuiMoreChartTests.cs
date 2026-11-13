using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Focused bindings for native flows, ranks, rings, scatter and market instruments.</summary>
public sealed class CuiMoreChartTests
{
    /// <summary>Keeps call-scoped input alive through deferred paint for each new widget.</summary>
    /// <param name="renderer">Native renderer.</param>
    /// <param name="kind">Widget index.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software, 0)]
    [InlineData(CuiRenderer.Software, 1)]
    [InlineData(CuiRenderer.Software, 2)]
    [InlineData(CuiRenderer.Software, 3)]
    [InlineData(CuiRenderer.Software, 4)]
    [InlineData(CuiRenderer.Software, 5)]
    [InlineData(CuiRenderer.OpenGl, 0)]
    [InlineData(CuiRenderer.OpenGl, 1)]
    [InlineData(CuiRenderer.OpenGl, 2)]
    [InlineData(CuiRenderer.OpenGl, 3)]
    [InlineData(CuiRenderer.OpenGl, 4)]
    [InlineData(CuiRenderer.OpenGl, 5)]
    public void MoreChartsOwnDataThroughPainting(CuiRenderer renderer, int kind)
    {
        using var window = new CuiWindow("More chart ownership", 720, 500, renderer, continuous: true);
        window.SetReducedMotion(true);
        string a = Path.Combine(Path.GetTempPath(), Guid.NewGuid() + ".png");
        string b = Path.Combine(Path.GetTempPath(), Guid.NewGuid() + ".png");
        try
        {
            for (int i = 0; i < 5; i++) Draw(false);
            window.CapturePng(a); Draw(true); window.CapturePng(b);
            Assert.Equal(File.ReadAllBytes(a), File.ReadAllBytes(b));
            var bounds = window.GetControlRectangle("widget");
            Assert.True(bounds.Width > 0 && bounds.Height > 0);
        }
        finally { File.Delete(a); File.Delete(b); }

        void Draw(bool mutate)
        {
            Assert.True(window.NextFrame()); window.MovePointer(-1, -1); window.BeginColumn();
            CuiSankeyNode[] nodes = [new("Search ### literal"), new("Trial"), new("Paid")];
            CuiSankeyLink[] links = [new(0, 1, 42), new(1, 2, 24)];
            CuiVizItem[] items = [new("Move", .82f), new("Exercise", 1.24f), new("Stand", .58f)];
            CuiPlotPoint[] points = [new(1, 2), new(2, 3), new(3, 1), new(4, 4)];
            float[] sizes = [10, 20, 30, 40];
            CuiScatterSeries[] scatter = [new("Software 日本語", points, sizes), new("OpenGL", [new(2, 1), new(3, 3)])];
            CuiCandle[] candles = Candles(100);
            string[] labels = Enumerable.Range(0, 100).Select(i => "Day " + i).ToArray();
            float[] overlay = Enumerable.Range(0, 100).Select(i => i % 9 == 0 ? float.NaN : 64 + i * .2f).ToArray();
            switch (kind)
            {
                case 0: window.Sankey("widget", nodes, links, new(Height: 260, Format: new(0))); break;
                case 1: window.BarList("widget", items, new(Format: new(2))); break;
                case 2: window.ActivityRings("widget", items, new(Size: 170, Center: "82%", Caption: "of goal", Legend: true)); break;
                case 3: window.ScatterChart("widget", scatter, new(XLabel: "Nodes", YLabel: "Time", YFormat: new(2))); break;
                case 4: window.RadialGauge("widget", 64, 0, 100, new(Label: "CPU", Unit: "%", Bands:
                    [new(60, 2, "Normal"), new(85, 3, "Busy"), new(100, 4, "Overloaded")], Detail: "of goal")); break;
                case 5: window.Candlestick("widget", candles, new(Labels: labels, Overlays: [new("Average", overlay)])); break;
            }
            if (mutate)
            {
                Array.Clear(nodes); Array.Clear(links); Array.Clear(items); Array.Clear(scatter);
                Array.Fill(points, new(999, 999)); Array.Fill(sizes, 999); Array.Clear(candles);
                Array.Fill(labels, "Changed"); Array.Fill(overlay, 999);
            }
            var allocations = new List<nint>();
            try
            {
                for (int size = 8; size <= 256; size += 8)
                    for (int j = 0; j < 4; j++) { nint block = Marshal.AllocHGlobal(size); allocations.Add(block);
                        Marshal.Copy(Enumerable.Repeat((byte)'Z', size).ToArray(), 0, block, size); }
                window.EndColumn(); GC.Collect(); window.Present();
            }
            finally { foreach (nint block in allocations) Marshal.FreeHGlobal(block); }
        }
    }

    /// <summary>Keeps crosshair, zoom, appended data and retirement under native control.</summary>
    /// <param name="renderer">Native renderer.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void CandlestickUsesNativeKeysAndRetiresItsViewport(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Candle keys", 720, 440, renderer, continuous: true);
        window.SetReducedMotion(true);
        Draw(focus: true); Assert.Equal(0, Draw(key: 11));
        Assert.Equal(1, Draw(key: 8));
        Assert.Equal(99, Draw(key: 12));
        Draw(key: 53); Draw(key: 52); Draw(key: 42);
        Assert.Equal(99, Draw(count: 120));
        Assert.Equal(119, Draw(key: 12, count: 120));
        Assert.Equal(-1, Draw(enabled: false));
        window.ForgetControl("price");
        Assert.Equal(-1, Draw());
        Draw(focus: true);
        Assert.Equal(99, Draw(key: 12));
        Assert.True(window.ControlFocused("price"));

        int Draw(int key = 0, bool focus = false, bool enabled = true, int count = 100)
        {
            Assert.True(window.NextFrame()); window.MovePointer(-1, -1);
            if (key != 0) { window.SetKey(key, true); window.SetKey(key, false); }
            window.BeginColumn();
            int cursor = window.Candlestick("price", Candles(count), new(Height: 280), enabled, focus);
            window.EndColumn(); window.Present(); return cursor;
        }
    }

    /// <summary>Returns original array indices after native rank sorting and preserves label identities.</summary>
    /// <param name="renderer">Native renderer.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void BarListReturnsInputIndicesAfterRankChanges(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Ranks", 640, 420, renderer, continuous: true);
        window.SetReducedMotion(true);
        CuiVizItem[] items = [new("A", 10), new("B", 30), new("C", 20)];
        for (int i = 0; i < 4; i++) Draw(-1, -1);
        var r = window.GetControlRectangle("ranks");
        float x = r.X + 30, y = r.Y + 15;
        Draw(x, y, true); Assert.Equal(1, Draw(x, y, false));
        items = [new("A", 50), new("B", 30), new("C", 20)];
        Draw(-1, -1); Draw(x, y, true); Assert.Equal(0, Draw(x, y, false));
        Draw(x, y, true, false); Assert.Equal(-1, Draw(x, y, false, false));
        window.ForgetControl("ranks");
        Draw(-1, -1);

        int Draw(float mx, float my, bool? down = null, bool enabled = true)
        {
            Assert.True(window.NextFrame()); window.MovePointer(mx, my);
            if (down is { } pressed) window.SetLeftButton(pressed);
            window.BeginColumn();
            int clicked = window.BarList("ranks", items, new(), enabled);
            window.EndColumn(); window.Present(); return clicked;
        }
    }

    /// <summary>Maps a filtered zero-flow Sankey diagram back to its original link index.</summary>
    /// <param name="renderer">Native renderer.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void SankeyKeepsOriginalLinkIndices(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Flows", 640, 420, renderer, continuous: true);
        window.SetReducedMotion(true);
        for (int i = 0; i < 4; i++) Draw(-1, -1);
        var r = window.GetControlRectangle("flow");
        float x = r.X + r.Width / 2, y = r.Y + r.Height / 2;
        Draw(x, y, true); Assert.Equal(3, Draw(x, y, false));

        int Draw(float mx, float my, bool? down = null)
        {
            Assert.True(window.NextFrame()); window.MovePointer(mx, my);
            if (down is { } pressed) window.SetLeftButton(pressed);
            window.BeginColumn();
            int clicked = window.Sankey("flow", [new("Source"), new("Paid")], [new(0, 1, 0), new(0, 1, 42)], new(Height: 220));
            window.EndColumn(); window.Present(); return clicked;
        }
    }

    /// <summary>Rejects malformed public data before it reaches native rendering.</summary>
    [GuiFact]
    public void InvalidMoreChartDataIsRejected()
    {
        using var window = new CuiWindow("Chart validation", 640, 480, CuiRenderer.Software);
        Assert.True(window.NextFrame()); window.BeginColumn();
        Assert.Throws<ArgumentOutOfRangeException>(() => window.Sankey("bad", [new("A")], [new(0, 1, 1)], new()));
        Assert.Throws<ArgumentException>(() => window.BarList("bad", [new("A", 1), new("A", 2)], new()));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.ActivityRings("bad", Enumerable.Repeat(new CuiVizItem("A", 1), 9).ToArray(), new()));
        Assert.Throws<ArgumentException>(() => window.ScatterChart("bad", [new("A", [new(1, 1), new(2, 2)], [1])], new()));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.ScatterChart("bad", [new("A", [new(float.NaN, 1)])], new()));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.RadialGauge("bad", 10, 100, 0, new()));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.RadialGauge("bad", 10, 0, 100, new(Bands: [new(60, 2), new(40, 3)])));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.Candlestick("bad", [new(10, 9, 8, 11)], new()));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.Candlestick("bad", [new(10, 12, 8, 11, -1)], new()));
        window.Sankey("empty-flow", [], [], new(Height: 40));
        window.BarList("empty-ranks", [], new());
        window.ActivityRings("empty-rings", [], new(Size: 60));
        window.ScatterChart("empty-scatter", [], new(Height: 40));
        window.Candlestick("empty-price", [], new(Height: 40));
        window.EndColumn(); window.Present();
    }

    private static CuiCandle[] Candles(int count) => Enumerable.Range(0, count)
        .Select(i => new CuiCandle(64 + i * .2f, 65 + i * .2f, 63 + i * .2f, 64.5f + i * .2f, 2000 + i * 10)).ToArray();
}
