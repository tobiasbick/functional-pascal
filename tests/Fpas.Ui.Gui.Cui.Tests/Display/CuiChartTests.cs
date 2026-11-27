// [Documentation](../../../docs/ui/gui/charts/README.md)
using Fpas.Tests.Support;
using Fpas.Ui.Gui.Cui.Widgets.Charts;

namespace Fpas.Ui.Gui.Cui.Tests.Display;

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
        DeferredPaint.AssertInputsCopied(window, Draw);
        foreach (string id in new[] { "line", "bar", "donut", "gauge", "meter" })
        {
            var bounds = window.GetControlRectangle(id);
            Assert.True(bounds.Width > 0 && bounds.Height > 0, id);
        }

        void Draw(bool mutate)
        {
            Assert.True(window.NextFrame()); window.MovePointer(-1, -1); window.BeginColumn();
            float[] values = [18, 22, 21, 27];
            string[] labels = ["Jan", "Feb", "Mar", "Apr"];
            CuiChartSeries[] series = [new("Revenue", values, new(4)), new("Costs", [12, 13, 15])];
            var options = new CuiChartOptions(Height: 130, Labels: labels, Area: true, Smooth: true,
                Format: new(0, "$", "k"), XMinimum: 100, XMaximum: 400,
                Annotations: [new(2, 200, 22, "Target", "Threshold", Color: CuiChartColor.Theme(15))]);
            window.LineChart("line", series, options);
            window.BarChart("bar", series, options with { Stacked = true, Area = false });
            window.DonutChart("donut", values, labels, height: 120, format: new(0, Suffix: "%"));
            window.Gauge("gauge", "CPU ### literal", .42f, 90);
            window.PatternMeter("meter", [new("Photos", 81), new("Video", 58)],
                new(Total: 256, Label: "Storage", Unit: "GB"));
            if (mutate) { Array.Fill(values, 999); Array.Fill(labels, "Changed"); Array.Clear(series); }
            DeferredPaint.PresentOverFreedBuffers(window);
        }
    }

    /// <summary>An omitted caption and out-of-range readings use native gauge display rules.</summary>
    [GuiFact]
    public void GaugeClampsAndUsesDefaultDiameter()
    {
        const int width = 240, height = 360;
        using var window = new CuiWindow("Gauge options", width, height, CuiRenderer.Software, continuous: true);
        using var directory = new TemporaryDirectory();
        window.SetReducedMotion(true);
        uint[] expected = Draw("endpoints", "", 0, 1, 0);
        uint[] actual = Draw("clamped", null, -0.25f, 1.25f, -1);
        Assert.True(expected.SequenceEqual(actual));

        uint[] Draw(string name, string? label, float low, float high, float diameter)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(-1, -1);
            window.BeginColumn();
            window.Gauge("low", label, low, diameter);
            window.Gauge("high", label, high, diameter);
            window.EndColumn();
            window.Present();
            Assert.Equal(140, window.GetControlRectangle("low").Width);
            Assert.Equal(140, window.GetControlRectangle("high").Width);
            string path = directory.File(name + ".png");
            window.CapturePng(path);
            return PngAssertions.ReadPixels(path, width, height);
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
        Assert.Equal(-1, Draw(count: 0));
        Assert.Equal(-1, Draw());
        Assert.Equal(0, Draw(key: 8));
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

    /// <summary>The title and both kinds of legend entry preserve literal hash suffixes.</summary>
    [GuiFact]
    public void PatternMeterPreservesLiteralCaptions()
    {
        using var directory = new TemporaryDirectory();
        const int width = 600, height = 300;
        using var window = new CuiWindow("Pattern meter captions", width, height, CuiRenderer.Software,
            continuous: true);
        window.SetReducedMotion(true);
        var baseline = Draw("Capacity", "Photos", "Unused");
        Assert.False(baseline.SequenceEqual(Draw("Capacity##title", "Photos", "Unused")));
        Assert.False(baseline.SequenceEqual(Draw("Capacity", "Photos##legend", "Unused")));
        Assert.False(baseline.SequenceEqual(Draw("Capacity", "Photos", "Unused##legend")));

        uint[] Draw(string title, string part, string free)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(-1, -1);
            window.PatternMeter("meter", [new(part, 81, new(4), .35f)],
                new(Total: 100, Label: title, Unit: "GB", Format: new(1), FreeLabel: free,
                    Height: 24, WidthKind: 2, Width: 440));
            window.Present();
            Assert.Equal(440f, window.GetControlRectangle("meter").Width);
            string path = directory.File("meter.png");
            window.CapturePng(path);
            return PngAssertions.ReadPixels(path, width, height);
        }
    }

    /// <summary>Renders a signed, zero-padded scientific value with the requested precision and exponent case.</summary>
    [GuiFact]
    public void DonutShowsTypedScientificFormat()
    {
        const int width = 320, height = 260;
        using var window = new CuiWindow("Chart number format", width, height, CuiRenderer.Software, continuous: true);
        using var directory = new TemporaryDirectory();
        window.SetReducedMotion(true);
        uint[] expected = Draw("literal", 2, new(Prefix: "+01.20E+0"));
        uint[] actual = Draw("scientific", 120, new(2, Notation: CuiValueNotation.Scientific,
            FieldWidth: 10, Padding: CuiValuePadding.LeadingZeros, Sign: CuiValueSign.Always, Uppercase: true));
        Assert.Equal(expected, actual);

        uint[] Draw(string name, float value, CuiChartFormat format)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(-1, -1);
            window.DonutChart("donut", [value], [], height: 150, format: format);
            window.Present();
            string path = directory.File(name + ".png");
            window.CapturePng(path);
            return PngAssertions.ReadPixels(path, width, height);
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
        Assert.Throws<ArgumentOutOfRangeException>(() => window.BarChart("bad", data, options with { XMinimum = 5, XMaximum = 1 }));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.LineChart("bad", data, options with { Format = new(1000) }));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.LineChart("bad", [new("Palette", [1], new(7))], options));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.LineChart("bad", [new("Overflow", [float.MaxValue, -float.MaxValue])], options));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.LineChart("bad", [new("Constant", [1e10f, 1e10f, 1e10f])], options));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.BarChart("bad", [new("A", [float.MaxValue]), new("B", [float.MaxValue])],
            options with { Stacked = true, Minimum = 0, Maximum = 100 }));
        Assert.Throws<ArgumentException>(() => window.DonutChart("bad", [1, 2], ["Only one"]));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.DonutChart("bad", [-1], []));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.DonutChart("bad", [1], [], format: new(FieldWidth: -1)));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.DonutChart("bad", [1], [], format: new(Notation: (CuiValueNotation)99)));
        Assert.Throws<ArgumentOutOfRangeException>("value", () => window.Gauge("bad", "Gauge", float.NaN));
        Assert.Throws<ArgumentOutOfRangeException>("size", () => window.Gauge("bad", "Gauge", 1, float.PositiveInfinity));
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
