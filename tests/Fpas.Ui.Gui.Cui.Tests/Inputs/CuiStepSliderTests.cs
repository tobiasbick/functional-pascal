// [Documentation](../../../docs/ui/gui/creative-controls/step-slider.md)
using Fpas.Tests.Support;

namespace Fpas.Ui.Gui.Cui.Tests.Inputs;

/// <summary>Verifies the step slider's displayed value and format validation.</summary>
public sealed class CuiStepSliderTests
{
    /// <summary>Formats an integer safely and applies the native text precedence.</summary>
    [GuiFact]
    public void StepSliderFormatsTheStop()
    {
        const int width = 360, height = 240;
        using var window = new CuiWindow("Step slider", width, height, CuiRenderer.Software, continuous: true);
        using var directory = new TemporaryDirectory();
        window.SetReducedMotion(true);

        const string formatted = "2 variants (100%)";
        uint[] expected = Capture("display", display: formatted);
        Assert.Equal(expected, Capture("format", format: "%d variants (100%%)"));
        Assert.Equal(expected, Capture("labels", format: "%d unused", labels: ["One", formatted, "Three", "Four"]));
        Assert.Equal(expected, Capture("override", format: "%d unused", labels: ["One", "Two", "Three", "Four"], display: formatted));

        Assert.True(window.NextFrame());
        int value = 2;
        foreach (string invalid in new[] { "%s", "%n", "%d %d", "%", "%1$d" })
            Assert.Throws<ArgumentException>(() => window.StepSlider(
                "step", "Variations", ref value, 1, 4, [], invalid, "", "", "", 1, -1, null));
        window.Present();

        uint[] Capture(string name, string format = "", string[]? labels = null, string display = "")
        {
            Assert.True(window.NextFrame());
            window.MovePointer(-1, -1);
            int current = 2;
            Assert.Equal(0, window.StepSlider("step", "Variations", ref current, 1, 4,
                labels ?? [], format, display, "", "", 1, -1, null));
            Assert.Equal(2, current);
            window.Present();
            string path = directory.File(name + ".png");
            window.CapturePng(path);
            return PngAssertions.ReadPixels(path, width, height);
        }
    }
}
