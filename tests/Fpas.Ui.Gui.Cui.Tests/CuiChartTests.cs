namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Focused chart binding, copied-data and native keyboard contracts.</summary>
public sealed class CuiChartTests
{
    /// <summary>Copies all caller data before presentation, including labels and annotations.</summary>
    /// <param name="renderer">Native renderer.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void ChartsCopyInputsBeforePresentation(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Charts", 720, 900, renderer, continuous: true);
        window.SetReducedMotion(true);
        string first = Path.Combine(Path.GetTempPath(), Guid.NewGuid() + ".png");
        string second = Path.Combine(Path.GetTempPath(), Guid.NewGuid() + ".png");
        try
        {
            for (int frame = 0; frame < 5; frame++) Draw(false);
            window.CapturePng(first);
            Draw(true);
            window.CapturePng(second);
            Assert.Equal(File.ReadAllBytes(first), File.ReadAllBytes(second));
            foreach (string id in new[] { "line", "bar", "donut", "gauge", "meter" })
            {
                var bounds = window.GetControlRectangle(id);
                Assert.True(bounds.Width > 0 && bounds.Height > 0, id);
            }
        }
        finally { File.Delete(first); File.Delete(second); }

        void Draw(bool mutate)
        {
            Assert.True(window.NextFrame()); window.MovePointer(-1, -1); window.BeginColumn();
            float[] values = [18, 22, 21, 27];
            string[] labels = ["Jan", "Feb", "Mar", "Apr"];
            CuiChartSeries[] series = [new("Revenue", values, new(4)), new("Costs", [12, 13, 15])];
            var options = new CuiChartOptions(Height: 130, Labels: labels, Area: true, Smooth: true,
                Format: new(0, "$", "k"), Annotations: [new(1, 0, 20, "Target", "Threshold", true)]);
            window.LineChart("line", series, options);
            window.BarChart("bar", series, options with { Stacked = true, Area = false });
            window.DonutChart("donut", values, labels, height: 120, format: new(0, Suffix: "%"));
            window.Gauge("gauge", "CPU ### literal", .42f, 90);
            window.PatternMeter("meter", [new("Photos", 81), new("Video", 58)],
                new(Total: 256, Label: "Storage", Unit: "GB"));
            if (mutate) { Array.Fill(values, 999); Array.Fill(labels, "Changed"); Array.Clear(series); }
            window.EndColumn(); GC.Collect(); window.Present();
        }
    }

    /// <summary>Keeps native keyboard selection, clamps it after shrinking, and retires it on removal.</summary>
    /// <param name="renderer">Native renderer.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void PatternMeterOwnsKeyboardSelection(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Pattern meter", 600, 350, renderer, continuous: true);
        window.SetReducedMotion(true);
        Draw(focus: true);
        Draw(key: 1); // Tab establishes keyboard focus rather than pointer focus.
        Draw(focus: true);
        Assert.Equal(0, Draw(key: 8));
        Assert.True(window.ControlFocused("meter"));
        Assert.Equal(1, Draw(key: 8));
        Assert.Equal(2, Draw(key: 12));
        Assert.Equal(0, Draw(count: 1));
        Assert.Equal(-1, Draw(enabled: false));
        window.ForgetControl("meter");
        Assert.Equal(-1, Draw());
        Assert.False(window.ControlFocused("meter"));

        int Draw(int key = 0, bool focus = false, int count = 3, bool enabled = true)
        {
            Assert.True(window.NextFrame()); window.MovePointer(-1, -1);
            if (key != 0) { window.SetKey(key, true); window.SetKey(key, false); }
            window.BeginColumn();
            CuiMeterPart[] parts = [new("Photos", 81), new("Video", 58), new("Apps", 41)];
            int selected = window.PatternMeter("meter", parts[..count], new(Total: 256, Label: "Storage"),
                enabled, focus);
            window.EndColumn(); window.Present(); return selected;
        }
    }

    /// <summary>Rejects invalid data before native mutation and keeps literal percent formatting safe.</summary>
    [GuiFact]
    public void InvalidChartsDoNotReachNativeCode()
    {
        using var window = new CuiWindow("Chart validation", 600, 500, CuiRenderer.Software);
        Assert.True(window.NextFrame()); window.BeginColumn();
        CuiChartSeries[] data = [new("Series", [1, 2, 3])];
        var options = new CuiChartOptions();
        Assert.Throws<ArgumentOutOfRangeException>(() => window.LineChart("bad", [new("NaN", [float.NaN])], options));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.BarChart("bad", data, options with { Minimum = 5, Maximum = 1 }));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.LineChart("bad", data, options with { Format = new(7) }));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.LineChart("bad", [new("Palette", [1], new(7))], options));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.LineChart("bad", [new("Overflow", [float.MaxValue, -float.MaxValue])], options));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.LineChart("bad", [new("Constant", [1e10f, 1e10f, 1e10f])], options));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.BarChart("bad", [new("A", [float.MaxValue]), new("B", [float.MaxValue])],
            options with { Stacked = true, Minimum = 0, Maximum = 100 }));
        Assert.Throws<ArgumentException>(() => window.DonutChart("bad", [1, 2], ["Only one"]));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.DonutChart("bad", [-1], []));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.Gauge("bad", "Gauge", 2));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.PatternMeter("bad", [new("Bad", 1, Density: 2)], new()));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.PatternMeter("bad", Enumerable.Repeat(new CuiMeterPart("Part", 1), 33).ToArray(), new()));
        window.LineChart("safe", data, options with { Format = new(2, "%n %s ", " %"), Annotations = [new(2, 1, 2, "%n", "%s")] });
        window.BarChart("empty", [], options);
        window.LineChart("large", [new("Constant", [1e10f, 1e10f, 1e10f])],
            options with { Minimum = 0, Maximum = 2e10f, Height = 60 });
        window.DonutChart("zero", [0, 0], ["A", "B"]);
        window.PatternMeter("empty-meter", [], new());
        window.EndColumn(); window.Present();
    }
}
