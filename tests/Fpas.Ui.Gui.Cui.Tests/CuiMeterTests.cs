using Fpas.Tests.Support;

namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Exercises the native capacity meter through its managed ABI.</summary>
public sealed class CuiMeterTests
{
    /// <summary>Literal captions, zero/exceeded capacity and narrow-wide-narrow geometry remain usable.</summary>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void MeterUsesNativeGeometryForAllCapacityStates(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Capacity meter", 500, 240, renderer, continuous: true);
        window.SetReducedMotion(true);
        foreach (int width in new[] { 500, 280, 700, 280 })
        {
            window.Resize(width, 240);
            foreach (var (used, total) in new[] { (18.4f, 20f), (24f, 20f), (0f, 0f), (4f, 0f) })
            {
                Draw(used, total); Draw(used, total);
                var box = window.GetControlRectangle("meter");
                Assert.Equal(window.Size.Width - 24, box.Width);
                Assert.True(box.Height > 25);
                Assert.False(window.ControlFocused("meter"));
            }
        }

        void Draw(float used, float total)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(-1, -1);
            window.BeginColumn(12, 10);
            window.Meter("meter", "Storage ## literal – λ", used, total, "GB ## unit");
            window.TrackLastControl("meter");
            window.EndColumn();
            window.Present();
        }
    }

    /// <summary>Removing feedback releases its native progress interpolation before reinsertion.</summary>
    [GuiTheory]
    [InlineData(CuiRenderer.Software, true)]
    [InlineData(CuiRenderer.OpenGl, true)]
    [InlineData(CuiRenderer.Software, false)]
    [InlineData(CuiRenderer.OpenGl, false)]
    public void RemovingFeedbackReleasesInterpolation(CuiRenderer renderer, bool meter)
    {
        using var directory = new TemporaryDirectory();
        using var window = new CuiWindow("Meter lifetime", 500, 240, renderer, continuous: true);
        Draw(0);
        Draw(10);
        var box = window.GetControlRectangle("meter");
        int x = (int)(box.X + box.Width * .8f), y = (int)(box.Y + box.Height - 3);
        string before = directory.File("before.png"), after = directory.File("after.png");
        window.CapturePng(before);
        Draw(null);
        Draw(10);
        window.CapturePng(after);
        Assert.NotEqual(PngAssertions.ReadPixels(before, 500, 240)[y * 500 + x],
            PngAssertions.ReadPixels(after, 500, 240)[y * 500 + x]);

        void Draw(float? used)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(-1, -1);
            window.BeginColumn(12);
            if (used is { } amount)
            {
                if (meter) window.Meter("meter", "Storage", amount, 10, "GB");
                else window.Progress("meter", amount / 10, label: "Storage", showValue: true);
                window.TrackLastControl("meter");
            }
            window.EndColumn();
            window.Present();
        }
    }

    /// <summary>Invalid values and strings are rejected before constructing native nodes.</summary>
    [GuiFact]
    public void InvalidMeterOptionsLeaveTheFrameUsable()
    {
        using var window = new CuiWindow("Meter validation", renderer: CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        foreach (float invalid in new[] { -1f, float.NaN, float.PositiveInfinity })
        {
            Assert.Throws<ArgumentOutOfRangeException>(() => window.Meter("bad", "Storage", invalid, 20));
            Assert.Throws<ArgumentOutOfRangeException>(() => window.Meter("bad", "Storage", 1, invalid));
        }
        Assert.Throws<ArgumentException>(() => window.Meter("bad", "zero\0", 1, 20));
        Assert.Throws<ArgumentException>(() => window.Meter("bad", "Storage", 1, 20, "zero\0"));
        window.Meter("valid", "Storage", 18.4f, 20, "GB");
        window.Present();
    }
}
