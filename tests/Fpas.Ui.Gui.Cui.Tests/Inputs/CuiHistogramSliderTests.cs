// [Documentation](../../../docs/ui/gui/pickers/histogram-slider.md)
using Fpas.Tests.Support;
using Fpas.Ui.Gui.Cui.Widgets.Charts;

namespace Fpas.Ui.Gui.Cui.Tests.Inputs;

/// <summary>Checks histogram formatting and input without bin data.</summary>
public sealed class CuiHistogramSliderTests
{
    /// <summary>Applies both endpoint formats and leaves the range operable when the bins are empty.</summary>
    [GuiFact]
    public void EmptyHistogramRetainsTheRangeControl()
    {
        const int width = 360, height = 240;
        using var window = new CuiWindow("Empty histogram", width, height, CuiRenderer.Software, continuous: true);
        using var directory = new TemporaryDirectory();
        window.SetReducedMotion(true);
        var format = new CuiChartFormat(1, Prefix: "$", Suffix: "%", Sign: CuiValueSign.Always);
        Assert.False(Draw(null, true, false));
        uint[] defaultPixels = Capture("default");
        Assert.False(Draw(format, false, false));
        Assert.False(defaultPixels.SequenceEqual(Capture("formatted")));
        Assert.True(window.ControlFocused("histogram"));
        Assert.True(Draw(format, false, true));

        bool Draw(CuiChartFormat? numberFormat, bool focus, bool enterRight)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(-1, -1);
            if (enterRight)
            {
                window.SetKey(8, true);
                window.SetKey(8, false);
            }
            bool changed = window.HistogramSlider("histogram", "Price", [], 40, 60, 0, 100,
                32, numberFormat, enabled: true, focus: focus, track: true, out float lower, out float upper);
            window.Present();
            Assert.Equal(60f, upper);
            if (enterRight) Assert.InRange(lower, MathF.BitIncrement(40), 60);
            else Assert.Equal(40f, lower);
            return changed;
        }

        uint[] Capture(string name)
        {
            string capture = directory.File(name + ".png");
            window.CapturePng(capture);
            return PngAssertions.ReadPixels(capture, width, height);
        }
    }
}
