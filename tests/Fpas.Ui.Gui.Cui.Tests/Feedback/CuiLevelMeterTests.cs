// [Documentation](../../../docs/ui/gui/audio/level-meter.md)
using Fpas.Tests.Support;

namespace Fpas.Ui.Gui.Cui.Tests.Feedback;

/// <summary>Exercises CUI's latching audio clip indicator through the managed adapter.</summary>
public sealed class CuiLevelMeterTests
{
    /// <summary>A click clears a latched clip once, after the signal drops below clipping.</summary>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void ClipLightReportsOneReset(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Audio clip", 320, 160, renderer, continuous: true);
        window.SetReducedMotion(true);
        Assert.False(Draw(1.2f));
        var meter = window.GetControlRectangle("levels");
        float x = meter.X + meter.Width - 2, y = meter.Y + 3;
        Assert.False(Draw(0.4f, x, y, pressed: true));
        Assert.True(Draw(0.4f, x, y, pressed: false));
        Assert.False(Draw(0.4f, x, y, pressed: true));
        Assert.False(Draw(0.4f, x, y, pressed: false));

        bool Draw(float peak, float x = -1, float y = -1, bool pressed = false)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(x, y);
            window.SetLeftButton(pressed);
            window.BeginColumn(12);
            bool cleared = window.LevelMeter("levels", [peak, 0.3f], -60, 6, 160, 6,
                horizontal: true, scale: true, segmented: true, hold: 1.5f, decay: 20,
                enabled: true, track: true);
            window.EndColumn();
            window.Present();
            return cleared;
        }
    }

    /// <summary>The horizontal clip light stays at the loud end, even in a right-to-left window.</summary>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void ClipResetUsesCurrentChannelsInRightToLeftLayout(CuiRenderer renderer)
    {
        const int rightToLeft = 2;
        using var window = new CuiWindow("RTL audio clip", 320, 160, renderer, continuous: true,
            readingDirection: rightToLeft);
        window.SetReducedMotion(true);
        Assert.False(Draw([1.2f, 0.2f]));
        var meter = window.GetControlRectangle("levels");
        Assert.True(meter.X > 100);
        float x = meter.X + meter.Width - 2, y = meter.Y + 3;

        Assert.False(Draw([0.2f, 0.2f], x, y, pressed: true));
        Assert.True(Draw([0.2f, 0.2f], x, y, pressed: false));

        Assert.False(Draw([0.2f, 1.2f], x, y));
        Assert.False(Draw([0.2f, 0.2f], x, y, pressed: true));
        Assert.False(Draw([0.2f], x, y, pressed: false));
        Assert.False(Draw([0.2f, 0.2f], x, y));
        Assert.False(Draw([0.2f, 0.2f], x, y, pressed: true));
        Assert.False(Draw([0.2f, 0.2f], x, y, pressed: false));

        bool Draw(float[] peaks, float pointerX = -1, float pointerY = -1, bool pressed = false)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(pointerX, pointerY);
            window.SetLeftButton(pressed);
            window.BeginColumn(12);
            bool cleared = window.LevelMeter("levels", peaks, -60, 6, 160, 6,
                horizontal: true, scale: true, segmented: true, hold: 1.5f, decay: 20,
                enabled: true, track: true);
            window.EndColumn();
            window.Present();
            return cleared;
        }
    }

    /// <summary>Zero selects the same meter geometry and drawing as the explicit native defaults.</summary>
    [GuiFact]
    public void ZeroSelectsNativeDefaults()
    {
        const int width = 320, height = 160;
        using var directory = new TemporaryDirectory();
        using var window = new CuiWindow("Native meter defaults", width, height,
            CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        string defaults = directory.File("defaults.png"), explicitOptions = directory.File("explicit.png");
        Draw(true, defaults);
        var bounds = window.GetControlRectangle("levels");
        Draw(false, explicitOptions);
        Assert.Equal(bounds, window.GetControlRectangle("levels"));
        Assert.Equal(PngAssertions.ReadPixels(explicitOptions, width, height),
            PngAssertions.ReadPixels(defaults, width, height));

        void Draw(bool zero, string path)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(-1, -1);
            window.BeginColumn(12);
            Assert.False(window.LevelMeter("levels", [0.65f, 0.2f], zero ? 0 : -60, zero ? 0 : 6,
                zero ? 0 : 160, zero ? 0 : 6, horizontal: true, scale: true, segmented: true,
                hold: zero ? 0 : 1.5f, decay: zero ? 0 : 20, enabled: true, track: true));
            window.EndColumn();
            window.Present();
            window.CapturePng(path);
        }
    }
}
