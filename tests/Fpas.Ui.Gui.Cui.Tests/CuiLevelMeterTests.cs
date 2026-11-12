using Fpas.Tests.Support;

namespace Fpas.Ui.Gui.Cui.Tests;

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
}
