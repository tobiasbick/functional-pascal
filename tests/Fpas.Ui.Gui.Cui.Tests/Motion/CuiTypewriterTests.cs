// [Documentation](../../../docs/ui/gui/motion/typewriter.md)
using Fpas.Ui.Gui.Cui.Widgets.Motion;

namespace Fpas.Ui.Gui.Cui.Tests.Motion;

/// <summary>Checks completion transitions and the native typewriter's retained lifetime.</summary>
public sealed class CuiTypewriterTests
{
    /// <summary>Reports completion once and restarts after replay, replacement, forgetting and removal.</summary>
    /// <param name="renderer">Native renderer used for presentation.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void CompletionAndRestartRespectNativeState(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Typewriter state", 480, 240, renderer, continuous: true);
        Assert.Equal(new(false, false), Draw());
        Complete();
        Assert.Equal(new(false, false), Draw(replay: 1));
        Complete(replay: 1);
        Assert.Equal(new(false, false), Draw(replay: 1, text: "Replacement 日本語"));
        Complete(replay: 1, text: "Replacement 日本語");
        window.ForgetControl("typing");
        Assert.Equal(new(false, false), Draw(replay: 1, text: "Replacement 日本語"));
        Complete(replay: 1, text: "Replacement 日本語");
        Assert.True(window.NextFrame());
        window.Text("Removed");
        window.Present();
        Assert.Equal(new(false, false), Draw(replay: 1, text: "Replacement 日本語"));
        Complete(replay: 1, text: "Replacement 日本語");

        void Complete(int replay = 0, string text = "Hello 😀")
        {
            // Reduced motion finishes the pending text without depending on wall-clock timing.
            Assert.Equal(new(true, true), Draw(replay, text, reduced: true));
            Assert.Equal(new(true, false), Draw(replay, text, reduced: true));
        }

        CuiTypewriterResult Draw(int replay = 0, string text = "Hello 😀", bool reduced = false)
        {
            window.SetReducedMotion(reduced);
            Assert.True(window.NextFrame());
            window.MovePointer(-1, -1);
            var result = window.Typewriter("typing", text, replay,
                new CuiMotionText(0, 0, 0, default, 16, Delay: 600));
            window.TrackLastControl("typing");
            window.Present();
            return result;
        }
    }
}
