using Fpas.Tests.Support;

namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Checks rendering at a ruler bound between ticks.</summary>
public sealed class CuiRulerSliderTests
{
    /// <summary>Scrolling at the bound keeps both the displayed value and the proposal in range.</summary>
    [GuiFact]
    public void RulerKeepsTheDisplayedValueInsideItsBounds()
    {
        const int width = 320, height = 240;
        const float maximum = 1.6f;
        using var directory = new TemporaryDirectory();
        using var window = new CuiWindow("Ruler bound", width, height, CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        Draw();
        var tape = window.GetControlRectangle("ruler");
        string baseline = directory.File("baseline.png"), scrolled = directory.File("scrolled.png");
        Draw(tape);
        window.CapturePng(baseline);
        Assert.Equal(maximum, Draw(tape, scroll: -1));
        window.CapturePng(scrolled);

        uint[] before = PngAssertions.ReadPixels(baseline, width, height);
        uint[] after = PngAssertions.ReadPixels(scrolled, width, height);
        int aboveTape = (int)tape.Y * width;
        Assert.True(before.AsSpan(0, aboveTape).SequenceEqual(after.AsSpan(0, aboveTape)),
            "The ruler must still display 1.6 after scrolling at its upper bound.");

        float Draw(CuiRectangle? area = null, float scroll = 0)
        {
            Assert.True(window.NextFrame());
            if (area is { } rect)
                window.MovePointer(rect.X + rect.Width / 2, rect.Y + rect.Height / 2);
            else
                window.MovePointer(-1, -1);
            if (scroll != 0) window.Wheel(scroll);
            window.BeginColumn(12);
            float value = window.RulerSlider("ruler", "Value", maximum, 0, maximum, 1, 10, 9, 0,
                1, "", enabled: true, focus: false, track: true);
            window.EndColumn();
            window.Present();
            return value;
        }
    }
}
